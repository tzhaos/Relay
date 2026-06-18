use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    time::Duration,
};

use relay_core::{AgentKind, AgentRuntimeStatus, AgentStatusUpdate, Timestamp};
use relay_terminal::{TerminalEvent, TerminalSpawn};
use serde::{Deserialize, Serialize};

pub mod adapters;

pub use adapters::{ClaudeAdapter, CodexAdapter, GeminiAdapter};

pub type AgentResult<T> = Result<T, AgentError>;

#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("agent adapter not found: {0}")]
    MissingAdapter(String),
    #[error("agent command not found: {0}")]
    CommandNotFound(String),
    #[error("agent prompt is empty")]
    EmptyPrompt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromptDelivery {
    Argv,
    Flag { name: String },
    StdinAfterStart,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentLaunchRequest {
    pub cwd: PathBuf,
    pub initial_prompt: Option<String>,
    pub env: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentLaunchPlan {
    pub agent: AgentKind,
    pub program: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub cwd: PathBuf,
    pub expected_process: Option<String>,
    pub prompt_delivery: PromptDelivery,
    pub stdin_after_start: Option<Vec<u8>>,
}

impl AgentLaunchPlan {
    pub fn terminal_spawn(&self, cols: u16, rows: u16, scrollback_limit: usize) -> TerminalSpawn {
        TerminalSpawn {
            cwd: self.cwd.clone(),
            program: self.program.clone(),
            args: self.args.clone(),
            env: self.env.clone(),
            cols,
            rows,
            scrollback_limit,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentInput {
    pub bytes: Vec<u8>,
}

impl AgentInput {
    pub fn stdin_line(message: impl Into<String>) -> Self {
        let mut body = message.into();
        if !body.ends_with('\n') {
            body.push('\n');
        }
        Self {
            bytes: body.into_bytes(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentMessage {
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentAvailability {
    pub kind: AgentKind,
    pub command: String,
    pub path: Option<PathBuf>,
    pub available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeEnvironment {
    pub path: Option<OsString>,
    pub cwd: PathBuf,
}

impl RuntimeEnvironment {
    pub fn current() -> AgentResult<Self> {
        Ok(Self {
            path: std::env::var_os("PATH"),
            cwd: std::env::current_dir()
                .map_err(|_| AgentError::CommandNotFound("current directory".to_string()))?,
        })
    }

    pub fn with_path(cwd: impl Into<PathBuf>, path: impl Into<OsString>) -> Self {
        Self {
            path: Some(path.into()),
            cwd: cwd.into(),
        }
    }
}

pub trait AgentAdapter: Send + Sync {
    fn kind(&self) -> AgentKind;
    fn command(&self) -> &str;
    fn launch_plan(&self, request: AgentLaunchRequest) -> AgentResult<AgentLaunchPlan>;
    fn parse_terminal_event(
        &self,
        event: &TerminalEvent,
        observed_at: Timestamp,
    ) -> Option<AgentStatusUpdate>;
    fn format_followup(&self, message: AgentMessage) -> AgentResult<AgentInput>;

    fn detect(&self, env: &RuntimeEnvironment) -> AgentAvailability {
        let path = which::which_in(self.command(), env.path.as_ref(), &env.cwd).ok();
        AgentAvailability {
            kind: self.kind(),
            command: self.command().to_string(),
            available: path.is_some(),
            path,
        }
    }
}

pub struct AgentRegistry {
    adapters: Vec<Box<dyn AgentAdapter>>,
    stale_after: Duration,
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::built_in()
    }
}

impl AgentRegistry {
    pub fn built_in() -> Self {
        Self {
            adapters: vec![
                Box::new(ClaudeAdapter::default()),
                Box::new(CodexAdapter::default()),
                Box::new(GeminiAdapter::default()),
            ],
            stale_after: Duration::from_secs(120),
        }
    }

    pub fn with_adapters(adapters: Vec<Box<dyn AgentAdapter>>) -> Self {
        Self {
            adapters,
            stale_after: Duration::from_secs(120),
        }
    }

    pub fn with_stale_after(mut self, stale_after: Duration) -> Self {
        self.stale_after = stale_after;
        self
    }

    pub fn adapters(&self) -> &[Box<dyn AgentAdapter>] {
        &self.adapters
    }

    pub fn detect_available(&self, env: &RuntimeEnvironment) -> Vec<AgentAvailability> {
        self.adapters
            .iter()
            .map(|adapter| adapter.detect(env))
            .collect()
    }

    pub fn resolve(&self, kind: &AgentKind) -> AgentResult<&dyn AgentAdapter> {
        self.adapters
            .iter()
            .find(|adapter| adapter.kind().label() == kind.label())
            .map(|adapter| adapter.as_ref())
            .ok_or_else(|| AgentError::MissingAdapter(kind.label().to_string()))
    }

    pub fn launch_plan(
        &self,
        kind: &AgentKind,
        request: AgentLaunchRequest,
    ) -> AgentResult<AgentLaunchPlan> {
        self.resolve(kind)?.launch_plan(request)
    }

    pub fn parse_terminal_event(
        &self,
        kind: &AgentKind,
        event: &TerminalEvent,
        observed_at: Timestamp,
    ) -> AgentResult<Option<AgentStatusUpdate>> {
        Ok(self.resolve(kind)?.parse_terminal_event(event, observed_at))
    }

    pub fn stale_status(
        &self,
        kind: AgentKind,
        last_observed_at: Timestamp,
        now: Timestamp,
    ) -> Option<AgentStatusUpdate> {
        let elapsed = (now - last_observed_at).unsigned_abs();
        if elapsed < self.stale_after {
            return None;
        }

        Some(AgentStatusUpdate {
            state: AgentRuntimeStatus::Stale,
            prompt: "Agent status is stale".to_string(),
            agent_kind: Some(kind),
            observed_at: now,
        })
    }
}

pub fn initial_prompt_request(
    cwd: impl AsRef<Path>,
    prompt: impl Into<String>,
) -> AgentResult<AgentLaunchRequest> {
    let prompt = prompt.into();
    if prompt.trim().is_empty() {
        return Err(AgentError::EmptyPrompt);
    }

    Ok(AgentLaunchRequest {
        cwd: cwd.as_ref().to_path_buf(),
        initial_prompt: Some(prompt),
        env: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use std::{env, path::PathBuf, time::Duration};

    use relay_core::{AgentKind, AgentRuntimeStatus, Timestamp};
    use relay_terminal::TerminalEvent;

    use super::*;

    #[test]
    fn registry_should_register_builtin_adapters() {
        let registry = AgentRegistry::built_in();
        let kinds: Vec<_> = registry
            .adapters()
            .iter()
            .map(|adapter| adapter.kind())
            .collect();

        assert_eq!(
            kinds,
            vec![AgentKind::Claude, AgentKind::Codex, AgentKind::Gemini]
        );
    }

    #[test]
    fn claude_launch_plan_should_deliver_prompt_as_argv() {
        let adapter = ClaudeAdapter::new("claude-test");
        let plan = adapter
            .launch_plan(initial_prompt_request("F:\\Workspace\\Relay", "build ui").unwrap())
            .expect("plan should build");

        assert_eq!(plan.args, vec!["build ui"]);
        assert_eq!(plan.prompt_delivery, PromptDelivery::Argv);
    }

    #[test]
    fn codex_launch_plan_should_deliver_prompt_as_flag() {
        let adapter = CodexAdapter::new("codex-test");
        let plan = adapter
            .launch_plan(initial_prompt_request("F:\\Workspace\\Relay", "fix tests").unwrap())
            .expect("plan should build");

        assert_eq!(
            plan.args,
            vec!["--prompt".to_string(), "fix tests".to_string()]
        );
        assert_eq!(
            plan.prompt_delivery,
            PromptDelivery::Flag {
                name: "--prompt".to_string()
            }
        );
    }

    #[test]
    fn gemini_launch_plan_should_deliver_prompt_after_start() {
        let adapter = GeminiAdapter::new("gemini-test");
        let plan = adapter
            .launch_plan(initial_prompt_request("F:\\Workspace\\Relay", "review diff").unwrap())
            .expect("plan should build");

        assert_eq!(plan.args, Vec::<String>::new());
        assert_eq!(plan.prompt_delivery, PromptDelivery::StdinAfterStart);
        assert_eq!(plan.stdin_after_start, Some(b"review diff\n".to_vec()));
    }

    #[test]
    fn launch_plan_should_convert_to_terminal_spawn() {
        let plan = AgentLaunchPlan {
            agent: AgentKind::Codex,
            program: "codex-test".to_string(),
            args: vec!["--prompt".to_string(), "hello".to_string()],
            env: vec![("RELAY".to_string(), "1".to_string())],
            cwd: PathBuf::from("F:\\Workspace\\Relay"),
            expected_process: Some("codex-test".to_string()),
            prompt_delivery: PromptDelivery::Flag {
                name: "--prompt".to_string(),
            },
            stdin_after_start: None,
        };

        let spawn = plan.terminal_spawn(120, 30, 8192);

        assert_eq!(
            (spawn.cols, spawn.rows, spawn.scrollback_limit),
            (120, 30, 8192)
        );
    }

    #[test]
    fn adapter_should_parse_terminal_status_output() {
        let adapter = CodexAdapter::new("codex-test");
        let update = adapter
            .parse_terminal_event(
                &TerminalEvent::Output {
                    session_id: relay_core::TerminalSessionId::new(),
                    data: b"waiting for approval".to_vec(),
                },
                Timestamp::UNIX_EPOCH,
            )
            .expect("status should parse");

        assert_eq!(update.state, AgentRuntimeStatus::Waiting);
    }

    #[test]
    fn registry_should_emit_stale_status_after_threshold() {
        let registry = AgentRegistry::built_in().with_stale_after(Duration::from_secs(30));
        let now = Timestamp::UNIX_EPOCH + time::Duration::seconds(31);
        let update = registry
            .stale_status(AgentKind::Claude, Timestamp::UNIX_EPOCH, now)
            .expect("status should be stale");

        assert_eq!(update.state, AgentRuntimeStatus::Stale);
    }

    #[test]
    fn adapter_detection_should_find_available_command() {
        let adapter = ClaudeAdapter::new("rustc");
        let env = RuntimeEnvironment::with_path(
            env::current_dir().expect("cwd should exist"),
            env::var_os("PATH").expect("PATH should exist"),
        );
        let availability = adapter.detect(&env);

        assert!(
            availability.available,
            "rustc should be available while tests run"
        );
    }
}
