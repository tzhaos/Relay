//! Terminal UI components.
//!
//! These components frame real PTY projections and CLI-agent shortcuts. They do
//! not emulate a terminal and they do not spawn processes.

mod agent_quick_launch;
mod line;
mod session_row;
mod status_badge;
mod surface;
mod tab;
mod toolbar;

pub use agent_quick_launch::AgentQuickLaunch;
pub use line::{TerminalLine, TerminalLineStyle};
pub use session_row::TerminalSessionRow;
pub use status_badge::TerminalStatusBadge;
pub use surface::TerminalSurface;
pub use tab::TerminalTab;
pub use toolbar::TerminalToolbar;
