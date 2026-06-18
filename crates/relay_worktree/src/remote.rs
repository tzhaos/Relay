use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{WorktreeError, WorktreeResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionHostKind {
    Local,
    Ssh {
        host: String,
        user: Option<String>,
        port: Option<u16>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteCommand {
    pub cwd: PathBuf,
    pub program: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteCommandOutput {
    pub status_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

pub trait RemoteExecutionHost {
    fn kind(&self) -> ExecutionHostKind;
    fn resolve_path(&self, path: &Path) -> PathBuf;
    fn run(&self, command: &RemoteCommand) -> WorktreeResult<RemoteCommandOutput>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct LocalExecutionHost;

impl RemoteExecutionHost for LocalExecutionHost {
    fn kind(&self) -> ExecutionHostKind {
        ExecutionHostKind::Local
    }

    fn resolve_path(&self, path: &Path) -> PathBuf {
        path.to_path_buf()
    }

    fn run(&self, command: &RemoteCommand) -> WorktreeResult<RemoteCommandOutput> {
        let mut process = Command::new(&command.program);
        process.current_dir(&command.cwd);
        process.args(&command.args);
        for (key, value) in &command.env {
            process.env(key, value);
        }
        let output = process.output()?;
        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;
        if !output.status.success() {
            return Err(WorktreeError::RemoteCommand {
                command: format_command(&command.program, &command.args),
                stderr,
            });
        }

        Ok(RemoteCommandOutput {
            status_code: output.status.code(),
            stdout,
            stderr,
        })
    }
}

fn format_command(program: &str, args: &[String]) -> String {
    std::iter::once(OsString::from(program))
        .chain(args.iter().map(OsString::from))
        .map(|arg| arg.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_execution_host_should_run_command_in_cwd() {
        let host = LocalExecutionHost;
        let output = host.run(&test_command()).expect("local command should run");

        assert!(output.stdout.contains("relay-remote"));
    }

    #[cfg(windows)]
    fn test_command() -> RemoteCommand {
        RemoteCommand {
            cwd: std::env::current_dir().expect("cwd should exist"),
            program: "cmd.exe".to_string(),
            args: vec![
                "/Q".to_string(),
                "/C".to_string(),
                "echo relay-remote".to_string(),
            ],
            env: Vec::new(),
        }
    }

    #[cfg(not(windows))]
    fn test_command() -> RemoteCommand {
        RemoteCommand {
            cwd: std::env::current_dir().expect("cwd should exist"),
            program: "sh".to_string(),
            args: vec!["-c".to_string(), "printf relay-remote".to_string()],
            env: Vec::new(),
        }
    }
}
