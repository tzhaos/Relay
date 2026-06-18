#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use gpui::{App, Application};
use relay_infra::{logging, paths::RelayPaths};
use relay_persistence::RelayDatabase;
use relay_ui::AppShell;

fn main() -> Result<()> {
    let paths = RelayPaths::discover()?;
    let _logging_guard = logging::init(&paths)?;
    let _database = RelayDatabase::open(paths.database_path())?;

    tracing::info!("starting Relay");
    Application::new().run(|cx: &mut App| {
        if let Err(error) = AppShell::open(cx) {
            tracing::error!(?error, "failed to open Relay window");
        }
    });

    Ok(())
}
