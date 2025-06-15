use std::env;
use std::fs;
use std::io::{self};
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use serde::Deserialize;

use crate::version::VERSION;
#[derive(Deserialize)]
struct Release {
    tag_name: String,
}

const REPO: &str = "MemerGamer/nodash";

pub fn check_for_update() -> io::Result<()> {
    let latest_version = get_latest_release_version()?;

    if latest_version != VERSION {
        println!("Updating from version {} to {}", VERSION, latest_version);
        download_and_replace_binary(&latest_version)?;
        println!("✅ Updated successfully to {}", latest_version);
    } else {
        println!("You are already running the latest version: {}", VERSION);
    }

    Ok(())
}


fn get_latest_release_version() -> io::Result<String> {
    let output = Command::new("curl")
        .args(&["-s", &format!("https://api.github.com/repos/{}/releases/latest", REPO)])
        .output()?;
    if !output.status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "Failed to fetch release info"));
    }

    let release: Release = serde_json::from_slice(&output.stdout)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("JSON parse error: {}", e)))?;
    let version = release.tag_name
        .strip_prefix('v')
        .map(|s| s.to_string())
        .unwrap_or(release.tag_name);

    Ok(version)
}


fn download_and_replace_binary(version: &str) -> io::Result<()> {
    let version = version.trim();
    let url = format!(
        "https://github.com/{}/releases/download/v{}/nodash-linux-v{}",
        REPO.trim(), version, version
    );

    // Debug output to verify the URL is correct
    println!("⬇️  Downloading: {}", url);

    let tmp_path = env::temp_dir().join("nodash-update");

    let status = Command::new("curl")
        .args(&["-L", "-o"])
        .arg(&tmp_path)
        .arg(&url)
        .status()?;

    if !status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "Download failed"));
    }

    println!("🔐 Making binary executable...");
    fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o755))?;

    let current_path = env::current_exe()?;
    println!("⚙️  Replacing binary at {:?}", current_path);

    match fs::copy(&tmp_path, &current_path) {
        Ok(_) => {}
        Err(_) => {
            println!("🔒 Permission denied. Retrying with sudo...");
            let status = Command::new("sudo")
                .arg("mv")
                .arg(&tmp_path)
                .arg(&current_path)
                .status()?;

            if !status.success() {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "sudo mv failed",
                ));
            }
        }
    }

    Ok(())
}
