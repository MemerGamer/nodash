mod help;
mod project;
mod shell;
mod tui;
mod updater;
mod version;

use crate::help::show_help;
use crate::project::{add_current_directory, load_projects, save_projects};
use crate::shell::open_project;
use crate::tui::run_app;
use crate::updater::check_for_update;
use std::io;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "help" | "--help" | "-h" => {
                show_help();
                return Ok(());
            }
            "add" => {
                add_current_directory()?;
                return Ok(());
            }
            "update" => {
                check_for_update()?;
                return Ok(());
            }
            "version" | "--version" | "-v" => {
                println!("nodash version: {}", version::VERSION);
                return Ok(());
            }
            _ => {
                println!("Unknown command: {}", args[1]);
                println!("Use 'nodash help' for available commands");
                return Ok(());
            }
        }
    }

    let mut projects = load_projects()?;
    run_app(&mut projects, open_project)?;
    save_projects(&projects)?;
    Ok(())
}
