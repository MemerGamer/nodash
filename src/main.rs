mod project;
mod shell;
mod tui;

use crate::project::{load_projects, save_projects};
use crate::shell::open_project;
use crate::tui::run_app;
use std::io;

fn main() -> io::Result<()> {
    let mut projects = load_projects()?;
    run_app(&mut projects, open_project)?;
    save_projects(&projects)?;
    Ok(())
}