use std::{
    borrow::Cow,
    collections::{HashMap, VecDeque},
    io::{Read, Write},
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
    time::Duration,
};

use portable_pty::{ChildKiller, CommandBuilder, MasterPty, PtySize, native_pty_system};
use relay_core::TerminalSessionId;

const DEVICE_STATUS_REPORT: &[u8] = b"\x1b[6n";
const CURSOR_POSITION_REPORT: &[u8] = b"\x1b[1;1R";

pub type TerminalResult<T> = Result<T, TerminalError>;

#[derive(Debug, thiserror::Error)]
pub enum TerminalError {
    #[error("pty error: {0}")]
    Pty(String),
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("terminal session not found: {0}")]
    MissingSession(TerminalSessionId),
    #[error("terminal session already exists: {0}")]
    SessionAlreadyExists(TerminalSessionId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalSpawn {
    pub cwd: PathBuf,
    pub program: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub cols: u16,
    pub rows: u16,
    pub scrollback_limit: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminalEvent {
    Output {
        session_id: TerminalSessionId,
        data: Vec<u8>,
    },
    Title {
        session_id: TerminalSessionId,
        title: String,
    },
    Cwd {
        session_id: TerminalSessionId,
        cwd: PathBuf,
    },
    Exited {
        session_id: TerminalSessionId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalSnapshot {
    pub session_id: TerminalSessionId,
    pub cwd: PathBuf,
    pub title: Option<String>,
    pub cols: u16,
    pub rows: u16,
    pub scrollback: String,
    pub exited: bool,
}

pub trait PtyProvider {
    fn spawn(&mut self, request: TerminalSpawn) -> TerminalResult<TerminalSessionId>;
    fn write(&mut self, session_id: TerminalSessionId, bytes: &[u8]) -> TerminalResult<()>;
    fn resize(&mut self, session_id: TerminalSessionId, cols: u16, rows: u16)
    -> TerminalResult<()>;
    fn kill(&mut self, session_id: TerminalSessionId) -> TerminalResult<()>;
    fn poll_event(&mut self, timeout: Duration) -> Option<TerminalEvent>;
    fn snapshot(&self, session_id: TerminalSessionId) -> TerminalResult<TerminalSnapshot>;
}

pub struct TerminalRuntime {
    sessions: HashMap<TerminalSessionId, TerminalSession>,
    tx: Sender<TerminalEvent>,
    rx: Receiver<TerminalEvent>,
}

impl Default for TerminalRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalRuntime {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            sessions: HashMap::new(),
            tx,
            rx,
        }
    }

    fn session_mut(&mut self, id: TerminalSessionId) -> TerminalResult<&mut TerminalSession> {
        self.sessions
            .get_mut(&id)
            .ok_or(TerminalError::MissingSession(id))
    }

    pub fn has_session(&self, id: TerminalSessionId) -> bool {
        self.sessions.contains_key(&id)
    }

    pub fn spawn_with_id(
        &mut self,
        id: TerminalSessionId,
        request: TerminalSpawn,
    ) -> TerminalResult<TerminalSessionId> {
        if self.sessions.contains_key(&id) {
            return Err(TerminalError::SessionAlreadyExists(id));
        }

        self.spawn_session(id, request)
    }

    fn spawn_session(
        &mut self,
        id: TerminalSessionId,
        request: TerminalSpawn,
    ) -> TerminalResult<TerminalSessionId> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: request.rows,
                cols: request.cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| TerminalError::Pty(error.to_string()))?;

        let mut command = CommandBuilder::new(&request.program);
        command.args(request.args.iter().map(String::as_str));
        command.cwd(&request.cwd);
        for (key, value) in &request.env {
            command.env(key, value);
        }

        let mut child = pair
            .slave
            .spawn_command(command)
            .map_err(|error| TerminalError::Pty(error.to_string()))?;
        let child_killer = child.clone_killer();
        drop(pair.slave);

        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|error| TerminalError::Pty(error.to_string()))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|error| TerminalError::Pty(error.to_string()))?;
        let tx = self.tx.clone();
        let reader_thread = thread::spawn(move || {
            let mut buffer = [0; 4096];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(read) => {
                        if tx
                            .send(TerminalEvent::Output {
                                session_id: id,
                                data: buffer[..read].to_vec(),
                            })
                            .is_err()
                        {
                            return;
                        }
                    }
                    Err(_) => break,
                }
            }
            let _ = tx.send(TerminalEvent::Exited { session_id: id });
        });
        let tx = self.tx.clone();
        let wait_thread = thread::spawn(move || {
            let _ = child.wait();
            let _ = tx.send(TerminalEvent::Exited { session_id: id });
        });

        self.sessions.insert(
            id,
            TerminalSession {
                id,
                cwd: request.cwd,
                cols: request.cols,
                rows: request.rows,
                scrollback: TerminalBuffer::new(request.scrollback_limit),
                exited: false,
                title: None,
                child_killer,
                writer,
                _master: pair.master,
                _reader_thread: Some(reader_thread),
                _wait_thread: Some(wait_thread),
            },
        );

        Ok(id)
    }
}

impl PtyProvider for TerminalRuntime {
    fn spawn(&mut self, request: TerminalSpawn) -> TerminalResult<TerminalSessionId> {
        let id = TerminalSessionId::new();
        self.spawn_session(id, request)
    }

    fn write(&mut self, session_id: TerminalSessionId, bytes: &[u8]) -> TerminalResult<()> {
        let session = self.session_mut(session_id)?;
        session.writer.write_all(bytes)?;
        session.writer.flush()?;
        Ok(())
    }

    fn resize(
        &mut self,
        session_id: TerminalSessionId,
        cols: u16,
        rows: u16,
    ) -> TerminalResult<()> {
        let session = self.session_mut(session_id)?;
        session.cols = cols;
        session.rows = rows;
        session
            ._master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| TerminalError::Pty(error.to_string()))
    }

    fn kill(&mut self, session_id: TerminalSessionId) -> TerminalResult<()> {
        let session = self.session_mut(session_id)?;
        let _ = session.child_killer.kill();
        session.exited = true;
        let _ = self.tx.send(TerminalEvent::Exited { session_id });
        Ok(())
    }

    fn poll_event(&mut self, timeout: Duration) -> Option<TerminalEvent> {
        let event = self.rx.recv_timeout(timeout).ok()?;
        match &event {
            TerminalEvent::Output { session_id, data } => {
                if let Some(session) = self.sessions.get_mut(session_id) {
                    if contains_device_status_report(data) {
                        let _ = session.writer.write_all(CURSOR_POSITION_REPORT);
                        let _ = session.writer.flush();
                    }
                    session.scrollback.push(&visible_terminal_output(data));
                }
            }
            TerminalEvent::Title { session_id, title } => {
                if let Some(session) = self.sessions.get_mut(session_id) {
                    session.title = Some(title.clone());
                }
            }
            TerminalEvent::Cwd { session_id, cwd } => {
                if let Some(session) = self.sessions.get_mut(session_id) {
                    session.cwd.clone_from(cwd);
                }
            }
            TerminalEvent::Exited { session_id } => {
                if let Some(session) = self.sessions.get_mut(session_id) {
                    session.exited = true;
                }
            }
        }
        Some(event)
    }

    fn snapshot(&self, session_id: TerminalSessionId) -> TerminalResult<TerminalSnapshot> {
        let session = self
            .sessions
            .get(&session_id)
            .ok_or(TerminalError::MissingSession(session_id))?;
        Ok(TerminalSnapshot {
            session_id: session.id,
            cwd: session.cwd.clone(),
            title: session.title.clone(),
            cols: session.cols,
            rows: session.rows,
            scrollback: session.scrollback.text(),
            exited: session.exited,
        })
    }
}

fn contains_device_status_report(data: &[u8]) -> bool {
    data.windows(DEVICE_STATUS_REPORT.len())
        .any(|window| window == DEVICE_STATUS_REPORT)
}

fn strip_device_status_report(data: &[u8]) -> Cow<'_, [u8]> {
    if !contains_device_status_report(data) {
        return Cow::Borrowed(data);
    }

    let mut stripped = Vec::with_capacity(data.len());
    let mut index = 0;
    while index < data.len() {
        if data[index..].starts_with(DEVICE_STATUS_REPORT) {
            index += DEVICE_STATUS_REPORT.len();
        } else {
            stripped.push(data[index]);
            index += 1;
        }
    }
    Cow::Owned(stripped)
}

fn visible_terminal_output(data: &[u8]) -> Cow<'_, [u8]> {
    let without_status_report = strip_device_status_report(data);
    if !contains_escape_sequence(without_status_report.as_ref()) {
        return without_status_report;
    }

    Cow::Owned(strip_ansi_escapes::strip(without_status_report.as_ref()))
}

fn contains_escape_sequence(data: &[u8]) -> bool {
    data.contains(&b'\x1b') || data.contains(&0x9b)
}

struct TerminalSession {
    id: TerminalSessionId,
    cwd: PathBuf,
    cols: u16,
    rows: u16,
    scrollback: TerminalBuffer,
    exited: bool,
    title: Option<String>,
    child_killer: Box<dyn ChildKiller + Send + Sync>,
    writer: Box<dyn Write + Send>,
    _master: Box<dyn MasterPty + Send>,
    _reader_thread: Option<JoinHandle<()>>,
    _wait_thread: Option<JoinHandle<()>>,
}

struct TerminalBuffer {
    chunks: VecDeque<String>,
    byte_len: usize,
    limit: usize,
}

impl TerminalBuffer {
    fn new(limit: usize) -> Self {
        Self {
            chunks: VecDeque::new(),
            byte_len: 0,
            limit,
        }
    }

    fn push(&mut self, bytes: &[u8]) {
        if self.limit == 0 {
            return;
        }

        let chunk = String::from_utf8_lossy(bytes).to_string();
        self.byte_len += chunk.len();
        self.chunks.push_back(chunk);
        while self.byte_len > self.limit {
            if let Some(removed) = self.chunks.pop_front() {
                self.byte_len = self.byte_len.saturating_sub(removed.len());
            } else {
                break;
            }
        }
    }

    fn text(&self) -> String {
        self.chunks.iter().cloned().collect()
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        if !self.exited {
            let _ = self.child_killer.kill();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;

    #[test]
    fn terminal_runtime_should_capture_pty_output() {
        let mut runtime = TerminalRuntime::new();
        let session_id = runtime
            .spawn(test_output_command())
            .expect("terminal should spawn");

        let deadline = Instant::now() + Duration::from_secs(5);
        let mut saw_output = false;
        while Instant::now() < deadline {
            match runtime.poll_event(Duration::from_millis(100)) {
                Some(TerminalEvent::Output { data, .. }) => {
                    if !String::from_utf8_lossy(&data).trim().is_empty() {
                        saw_output = true;
                        break;
                    }
                }
                Some(TerminalEvent::Exited { .. }) => {}
                Some(TerminalEvent::Title { .. } | TerminalEvent::Cwd { .. }) => {}
                None => {}
            }
        }

        let snapshot = runtime
            .snapshot(session_id)
            .expect("snapshot should be available");
        assert!(
            saw_output || !snapshot.scrollback.trim().is_empty(),
            "expected terminal output, got scrollback: {:?}",
            snapshot.scrollback
        );
    }

    #[test]
    fn terminal_runtime_should_resize_snapshot() {
        let mut runtime = TerminalRuntime::new();
        let session_id = runtime.spawn(test_shell()).expect("terminal should spawn");

        runtime
            .resize(session_id, 100, 32)
            .expect("resize should succeed");
        let snapshot = runtime
            .snapshot(session_id)
            .expect("snapshot should be available");

        assert_eq!((snapshot.cols, snapshot.rows), (100, 32));
    }

    #[test]
    fn terminal_runtime_should_mark_snapshot_exited_when_killed() {
        let mut runtime = TerminalRuntime::new();
        let session_id = runtime.spawn(test_shell()).expect("terminal should spawn");

        runtime.kill(session_id).expect("kill should succeed");
        let snapshot = runtime
            .snapshot(session_id)
            .expect("snapshot should be available");

        assert!(snapshot.exited, "terminal should be marked exited");
    }

    #[test]
    fn terminal_runtime_should_spawn_with_existing_logical_id() {
        let mut runtime = TerminalRuntime::new();
        let session_id = TerminalSessionId::new();

        let spawned_id = runtime
            .spawn_with_id(session_id, test_shell())
            .expect("terminal should spawn with caller id");

        assert_eq!(spawned_id, session_id);
        assert!(runtime.has_session(session_id));
        assert!(matches!(
            runtime
                .spawn_with_id(session_id, test_shell())
                .expect_err("duplicate logical id should fail"),
            TerminalError::SessionAlreadyExists(id) if id == session_id
        ));
    }

    #[test]
    fn terminal_output_should_strip_cursor_position_queries_from_scrollback() {
        let visible = visible_terminal_output(b"hello \x1b[6nworld");

        assert_eq!(visible.as_ref(), b"hello world");
    }

    #[test]
    fn terminal_output_should_strip_ansi_sequences_from_scrollback() {
        let visible = visible_terminal_output(b"\x1b[32mok\x1b[0m plain");

        assert_eq!(visible.as_ref(), b"ok plain");
    }

    #[cfg(windows)]
    fn test_shell() -> TerminalSpawn {
        TerminalSpawn {
            cwd: std::env::current_dir().expect("cwd should exist"),
            program: "cmd.exe".to_string(),
            args: vec!["/Q".to_string(), "/K".to_string()],
            env: Vec::new(),
            cols: 80,
            rows: 24,
            scrollback_limit: 4096,
        }
    }

    #[cfg(windows)]
    fn test_output_command() -> TerminalSpawn {
        TerminalSpawn {
            cwd: std::env::current_dir().expect("cwd should exist"),
            program: "cmd.exe".to_string(),
            args: vec![
                "/Q".to_string(),
                "/C".to_string(),
                "echo relay-terminal".to_string(),
            ],
            env: Vec::new(),
            cols: 80,
            rows: 24,
            scrollback_limit: 4096,
        }
    }

    #[cfg(not(windows))]
    fn test_shell() -> TerminalSpawn {
        TerminalSpawn {
            cwd: std::env::current_dir().expect("cwd should exist"),
            program: "sh".to_string(),
            args: Vec::new(),
            env: Vec::new(),
            cols: 80,
            rows: 24,
            scrollback_limit: 4096,
        }
    }

    #[cfg(not(windows))]
    fn test_output_command() -> TerminalSpawn {
        TerminalSpawn {
            cwd: std::env::current_dir().expect("cwd should exist"),
            program: "whoami".to_string(),
            args: Vec::new(),
            env: Vec::new(),
            cols: 80,
            rows: 24,
            scrollback_limit: 4096,
        }
    }
}
