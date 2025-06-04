use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{io};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub path: PathBuf,
    pub last_opened: Option<DateTime<Utc>>,
}

pub fn load_projects() -> io::Result<Vec<Project>> {
    let file = dirs::home_dir().unwrap().join(".nodash_projects.json");
    if !file.exists() {
        return Ok(vec![]);
    }
    let data = std::fs::read_to_string(file)?;
    let projects: Vec<Project> = serde_json::from_str(&data)?;
    Ok(projects)
}

pub fn save_projects(projects: &[Project]) -> io::Result<()> {
    let file = dirs::home_dir().unwrap().join(".nodash_projects.json");
    let data = serde_json::to_string_pretty(projects)?;
    std::fs::write(file, data)?;
    Ok(())
}
