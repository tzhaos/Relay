use relay_ui_kit::{IconName, TerminalLine, TerminalLineStyle, Tone};

use super::WorkbenchState;

#[derive(Clone, Copy)]
pub(super) struct DemoTask {
    pub title: &'static str,
    pub status: &'static str,
    pub tone: Tone,
    pub branch: &'static str,
    pub worktree: &'static str,
    pub changed: usize,
    pub review: usize,
    pub session_key: &'static str,
}

#[derive(Clone, Copy)]
pub(super) struct DemoSession {
    pub key: &'static str,
    pub label: &'static str,
    pub subtitle: &'static str,
    pub command: &'static str,
    pub cwd: &'static str,
    pub tone: Tone,
}

#[derive(Clone, Copy)]
pub(super) struct DemoFile {
    pub depth: usize,
    pub icon: IconName,
    pub name: &'static str,
    pub expandable: bool,
}

pub(super) const DEMO_TASKS: [DemoTask; 4] = [
    DemoTask {
        title: "Wire workbench shell",
        status: "Running",
        tone: Tone::Accent,
        branch: "ui/workbench-shell",
        worktree: "F:/Workspace/Relay",
        changed: 12,
        review: 0,
        session_key: "codex",
    },
    DemoTask {
        title: "Persist window state",
        status: "Review",
        tone: Tone::Warning,
        branch: "feat/window-state",
        worktree: "F:/Workspace/Relay/.worktrees/window-state",
        changed: 4,
        review: 3,
        session_key: "powershell",
    },
    DemoTask {
        title: "Terminal scrollback buffer",
        status: "Idle",
        tone: Tone::Muted,
        branch: "fix/scrollback",
        worktree: "F:/Workspace/Relay/.worktrees/scrollback",
        changed: 0,
        review: 0,
        session_key: "powershell",
    },
    DemoTask {
        title: "Agent retry on timeout",
        status: "Failed",
        tone: Tone::Danger,
        branch: "fix/agent-retry",
        worktree: "F:/Workspace/Relay/.worktrees/agent-retry",
        changed: 7,
        review: 1,
        session_key: "claude",
    },
];

pub(super) const DEMO_SESSIONS: [DemoSession; 3] = [
    DemoSession {
        key: "powershell",
        label: "PowerShell",
        subtitle: "Workspace terminal",
        command: "pwsh",
        cwd: "F:/Workspace/Relay",
        tone: Tone::Info,
    },
    DemoSession {
        key: "codex",
        label: "Codex",
        subtitle: "Agent terminal",
        command: "codex",
        cwd: "F:/Workspace/Relay",
        tone: Tone::Accent,
    },
    DemoSession {
        key: "claude",
        label: "Claude",
        subtitle: "Agent terminal",
        command: "claude",
        cwd: "F:/Workspace/Relay/.worktrees/agent-retry",
        tone: Tone::Warning,
    },
];

pub(super) const DEMO_FILES: [DemoFile; 13] = [
    DemoFile {
        depth: 0,
        icon: IconName::Folder,
        name: "crates",
        expandable: true,
    },
    DemoFile {
        depth: 1,
        icon: IconName::Folder,
        name: "relay_ui_kit",
        expandable: true,
    },
    DemoFile {
        depth: 2,
        icon: IconName::FileText,
        name: "theme.rs",
        expandable: false,
    },
    DemoFile {
        depth: 2,
        icon: IconName::FileText,
        name: "icon.rs",
        expandable: false,
    },
    DemoFile {
        depth: 2,
        icon: IconName::FileText,
        name: "button.rs",
        expandable: false,
    },
    DemoFile {
        depth: 2,
        icon: IconName::FileText,
        name: "terminal/surface.rs",
        expandable: false,
    },
    DemoFile {
        depth: 2,
        icon: IconName::FileText,
        name: "shell/split_pane.rs",
        expandable: false,
    },
    DemoFile {
        depth: 1,
        icon: IconName::Folder,
        name: "relay_gallery",
        expandable: true,
    },
    DemoFile {
        depth: 2,
        icon: IconName::FileText,
        name: "main.rs",
        expandable: false,
    },
    DemoFile {
        depth: 2,
        icon: IconName::FileText,
        name: "gallery.rs",
        expandable: false,
    },
    DemoFile {
        depth: 2,
        icon: IconName::FileText,
        name: "workbench_demo.rs",
        expandable: false,
    },
    DemoFile {
        depth: 0,
        icon: IconName::FileText,
        name: "DESIGN.md",
        expandable: false,
    },
    DemoFile {
        depth: 0,
        icon: IconName::FileText,
        name: "Cargo.toml",
        expandable: false,
    },
];

pub(super) fn active_task(state: &WorkbenchState) -> &'static DemoTask {
    DEMO_TASKS.get(state.active_task).unwrap_or(&DEMO_TASKS[0])
}

pub(super) fn active_session(state: &WorkbenchState) -> &'static DemoSession {
    DEMO_SESSIONS
        .get(state.active_session)
        .unwrap_or(&DEMO_SESSIONS[0])
}

pub(super) fn session_index_for_key(key: &str) -> usize {
    DEMO_SESSIONS
        .iter()
        .position(|session| session.key == key)
        .unwrap_or(0)
}

pub(super) fn session_lines(session: &DemoSession) -> Vec<TerminalLine> {
    match session.key {
        "codex" => vec![
            TerminalLine::new("relay@workspace").style(TerminalLineStyle::Prompt),
            TerminalLine::new("$ codex").style(TerminalLineStyle::Input),
            TerminalLine::new("Reviewing crates/relay_ui_kit shell and terminal components.")
                .style(TerminalLineStyle::Output),
            TerminalLine::new("UI actions are routed through the workbench command layer.")
                .style(TerminalLineStyle::Success),
        ],
        "claude" => vec![
            TerminalLine::new("relay@agent-retry").style(TerminalLineStyle::Prompt),
            TerminalLine::new("$ claude").style(TerminalLineStyle::Input),
            TerminalLine::new("Waiting for retry policy context.").style(TerminalLineStyle::Muted),
            TerminalLine::new("Last run exited with timeout.").style(TerminalLineStyle::Error),
        ],
        _ => vec![
            TerminalLine::new("relay@workspace").style(TerminalLineStyle::Prompt),
            TerminalLine::new("$ cargo test -p relay_ui_kit").style(TerminalLineStyle::Input),
            TerminalLine::new("21 tests passed").style(TerminalLineStyle::Success),
            TerminalLine::new("$ cargo build -p relay_gallery").style(TerminalLineStyle::Input),
        ],
    }
}

pub(super) fn prompt_prefix(session: &DemoSession) -> &'static str {
    match session.key {
        "codex" => "codex>",
        "claude" => "claude>",
        _ => "pwsh>",
    }
}
