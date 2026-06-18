use std::sync::LazyLock;

use regex::Regex;
use relay_core::{AgentKind, AgentRuntimeStatus, AgentStatusUpdate, Timestamp};
use relay_terminal::TerminalEvent;

use crate::{
    AgentAdapter, AgentInput, AgentLaunchPlan, AgentLaunchRequest, AgentMessage, AgentResult,
    PromptDelivery,
};

static WORKING_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\b(working|thinking|running|processing)\b").unwrap());
static WAITING_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(waiting|need input|confirm|approve|continue\?)\b").unwrap()
});
static BLOCKED_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\b(blocked|failed|error|permission denied)\b").unwrap());
static DONE_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\b(done|complete|completed|finished|success)\b").unwrap());

#[derive(Debug, Clone)]
pub struct ClaudeAdapter {
    command: String,
}

impl Default for ClaudeAdapter {
    fn default() -> Self {
        Self::new("claude")
    }
}

impl ClaudeAdapter {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
        }
    }
}

impl AgentAdapter for ClaudeAdapter {
    fn kind(&self) -> AgentKind {
        AgentKind::Claude
    }

    fn command(&self) -> &str {
        &self.command
    }

    fn launch_plan(&self, request: AgentLaunchRequest) -> AgentResult<AgentLaunchPlan> {
        let mut args = Vec::new();
        if let Some(prompt) = request.initial_prompt {
            args.push(prompt);
        }
        Ok(AgentLaunchPlan {
            agent: AgentKind::Claude,
            program: self.command.clone(),
            args,
            env: request.env,
            cwd: request.cwd,
            expected_process: Some(self.command.clone()),
            prompt_delivery: PromptDelivery::Argv,
            stdin_after_start: None,
        })
    }

    fn parse_terminal_event(
        &self,
        event: &TerminalEvent,
        observed_at: Timestamp,
    ) -> Option<AgentStatusUpdate> {
        parse_status_event(AgentKind::Claude, event, observed_at)
    }

    fn format_followup(&self, message: AgentMessage) -> AgentResult<AgentInput> {
        Ok(AgentInput::stdin_line(message.body))
    }
}

#[derive(Debug, Clone)]
pub struct CodexAdapter {
    command: String,
}

impl Default for CodexAdapter {
    fn default() -> Self {
        Self::new("codex")
    }
}

impl CodexAdapter {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
        }
    }
}

impl AgentAdapter for CodexAdapter {
    fn kind(&self) -> AgentKind {
        AgentKind::Codex
    }

    fn command(&self) -> &str {
        &self.command
    }

    fn launch_plan(&self, request: AgentLaunchRequest) -> AgentResult<AgentLaunchPlan> {
        let mut args = Vec::new();
        if let Some(prompt) = request.initial_prompt {
            args.push("--prompt".to_string());
            args.push(prompt);
        }
        Ok(AgentLaunchPlan {
            agent: AgentKind::Codex,
            program: self.command.clone(),
            args,
            env: request.env,
            cwd: request.cwd,
            expected_process: Some(self.command.clone()),
            prompt_delivery: PromptDelivery::Flag {
                name: "--prompt".to_string(),
            },
            stdin_after_start: None,
        })
    }

    fn parse_terminal_event(
        &self,
        event: &TerminalEvent,
        observed_at: Timestamp,
    ) -> Option<AgentStatusUpdate> {
        parse_status_event(AgentKind::Codex, event, observed_at)
    }

    fn format_followup(&self, message: AgentMessage) -> AgentResult<AgentInput> {
        Ok(AgentInput::stdin_line(message.body))
    }
}

#[derive(Debug, Clone)]
pub struct GeminiAdapter {
    command: String,
}

impl Default for GeminiAdapter {
    fn default() -> Self {
        Self::new("gemini")
    }
}

impl GeminiAdapter {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
        }
    }
}

impl AgentAdapter for GeminiAdapter {
    fn kind(&self) -> AgentKind {
        AgentKind::Gemini
    }

    fn command(&self) -> &str {
        &self.command
    }

    fn launch_plan(&self, request: AgentLaunchRequest) -> AgentResult<AgentLaunchPlan> {
        let stdin_after_start = request
            .initial_prompt
            .map(|prompt| AgentInput::stdin_line(prompt).bytes);
        Ok(AgentLaunchPlan {
            agent: AgentKind::Gemini,
            program: self.command.clone(),
            args: Vec::new(),
            env: request.env,
            cwd: request.cwd,
            expected_process: Some(self.command.clone()),
            prompt_delivery: PromptDelivery::StdinAfterStart,
            stdin_after_start,
        })
    }

    fn parse_terminal_event(
        &self,
        event: &TerminalEvent,
        observed_at: Timestamp,
    ) -> Option<AgentStatusUpdate> {
        parse_status_event(AgentKind::Gemini, event, observed_at)
    }

    fn format_followup(&self, message: AgentMessage) -> AgentResult<AgentInput> {
        Ok(AgentInput::stdin_line(message.body))
    }
}

fn parse_status_event(
    agent_kind: AgentKind,
    event: &TerminalEvent,
    observed_at: Timestamp,
) -> Option<AgentStatusUpdate> {
    let TerminalEvent::Output { data, .. } = event else {
        return None;
    };
    let text = String::from_utf8_lossy(data);
    let state = if DONE_PATTERN.is_match(&text) {
        AgentRuntimeStatus::Done
    } else if BLOCKED_PATTERN.is_match(&text) {
        AgentRuntimeStatus::Blocked
    } else if WAITING_PATTERN.is_match(&text) {
        AgentRuntimeStatus::Waiting
    } else if WORKING_PATTERN.is_match(&text) {
        AgentRuntimeStatus::Working
    } else {
        return None;
    };

    Some(AgentStatusUpdate {
        state,
        prompt: text.trim().to_string(),
        agent_kind: Some(agent_kind),
        observed_at,
    })
}
