use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{env, io};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub name: String,
    pub path: PathBuf,
    pub last_opened: Option<DateTime<Utc>>,
    pub node_version: Option<String>,
}

pub fn detect_node_version(path: &PathBuf) -> io::Result<String> {
    // Check if there's a .nvmrc file
    let nvmrc_path = path.join(".nvmrc");
    if nvmrc_path.exists() {
        let content = std::fs::read_to_string(nvmrc_path)?;
        let version = content.trim();

        // Remove 'v' prefix if present and return clean version
        let clean_version = version.strip_prefix('v').unwrap_or(version);
        Ok(clean_version.to_string())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No .nvmrc file found",
        ))
    }
}

fn sort_projects(projects: &mut Vec<Project>) {
    projects.sort_by(|a, b| match (a.last_opened, b.last_opened) {
        (Some(a_date), Some(b_date)) => b_date.cmp(&a_date),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.name.cmp(&b.name),
    });
}

pub fn add_current_directory() -> io::Result<()> {
    let current_dir = env::current_dir()?;
    let mut projects = load_projects()?;

    // Check if project already exists
    if projects.iter().any(|p| p.path == current_dir) {
        println!("Project already exists in nodash");
        return Ok(());
    }

    let project_name = current_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Unknown")
        .to_string();

    // Get node version before creating the project
    let node_version = detect_node_version(&current_dir).ok();

    let project = Project {
        name: project_name.clone(),
        path: current_dir.clone(),
        last_opened: None,
        node_version: node_version.clone(),
    };

    projects.push(project);
    save_projects(&projects)?;

    println!("✅ Added '{}' to nodash", project_name);
    println!("   Path: {}", current_dir.display());
    if let Some(version) = node_version {
        println!("   Node version: {}", version);
    } else {
        println!("   No .nvmrc file found");
    }

    Ok(())
}

pub fn load_projects() -> io::Result<Vec<Project>> {
    let file = dirs::home_dir().unwrap().join(".nodash_projects.json");
    if !file.exists() {
        return Ok(vec![]);
    }
    let data = std::fs::read_to_string(file)?;
    let projects: Vec<Project> = serde_json::from_str(&data)?;

    // Projects are already sorted when saved, no need to sort again
    Ok(projects)
}

pub fn save_projects(projects: &[Project]) -> io::Result<()> {
    let file = dirs::home_dir().unwrap().join(".nodash_projects.json");

    // Sort before saving to maintain order
    let mut sorted_projects = projects.to_vec();
    sort_projects(&mut sorted_projects);

    let data = serde_json::to_string_pretty(&sorted_projects)?;
    std::fs::write(file, data)?;
    Ok(())
}
