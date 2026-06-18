#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod runtime;

use anyhow::Result;
use gpui::{App, Application};
use relay_infra::{logging, paths::RelayPaths};
use relay_persistence::RelayDatabase;
use relay_ui::{AppShell, app_shell::TaskDataSource};

use crate::runtime::RelayRuntime;

fn main() -> Result<()> {
    let paths = RelayPaths::discover()?;
    let _logging_guard = logging::init(&paths)?;
    let database = RelayDatabase::open(paths.database_path())?;
    let mut runtime = RelayRuntime::open(database, &paths)?;
    let workspace = runtime.workspace_data()?;
    let mut task_data_source = Some(Box::new(runtime) as Box<dyn TaskDataSource>);

    tracing::info!("starting Relay");
    Application::new().run(move |cx: &mut App| {
        let task_data_source = task_data_source
            .take()
            .expect("Relay window should open once");
        if let Err(error) = AppShell::open(cx, workspace, task_data_source) {
            tracing::error!(?error, "failed to open Relay window");
        }
    });

    Ok(())
}
