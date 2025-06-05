mod project;
mod shell;
mod tui;
mod updater;

use crate::project::{load_projects, save_projects};
use crate::shell::open_project;
use crate::tui::run_app;
use crate::updater::check_for_update;
use std::io;

pub const VERSION: &str = concat!("v", env!("CARGO_PKG_VERSION"));
fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "update" {
        check_for_update()?;
        return Ok(());
    }

    let mut projects = load_projects()?;
    run_app(&mut projects, open_project)?;
    save_projects(&projects)?;
    Ok(())
}
