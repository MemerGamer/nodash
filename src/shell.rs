use crate::project::Project;
use std::io;
use std::process::Command;
use chrono::Utc;

pub fn open_project(proj: &mut Project) -> io::Result<()> {
    // println!("Opening project: {}", proj.name);
    proj.last_opened = Some(Utc::now());

    let command = format!(
        "zsh -c 'cd {} && source $NVM_DIR/nvm.sh && nvm install && nvm use && exec zsh'",
        proj.path.display()
    );

    Command::new("kitty")
        .arg("--")
        .arg("zsh")
        .arg("-c")
        .arg(&command)
        .spawn()?;

    Ok(())
}
