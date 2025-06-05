use std::env;
use std::fs;
use std::io::{self};
use std::process::Command;

use crate::VERSION;

const REPO: &str = "MemerGamer/nodash";

pub fn check_for_update() -> io::Result<()> {
    let latest_version = get_latest_release_version()?;

    if latest_version != VERSION {
        println!("Updating from version {} to {}", VERSION, latest_version);
        download_and_replace_binary(&latest_version)?;
        println!("Updated successfully.");
    } else {
        println!("You are already running the latest version: {}", VERSION);
    }

    Ok(())
}

fn get_latest_release_version() -> io::Result<String> {
    let output = Command::new("curl")
        .arg("-s")
        .arg(format!(
            "https://api.github.com/repos/{}/releases/latest",
            REPO
        ))
        .output()?;

    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to fetch latest release info",
        ));
    }

    let json = String::from_utf8_lossy(&output.stdout);
    let version = json
        .lines()
        .find(|line| line.trim_start().starts_with("\"tag_name\":"))
        .and_then(|line| {
            line.split(':')
                .nth(1)?
                .trim()
                .trim_matches('"')
                .strip_prefix('v')
                .map(String::from)
        })
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to parse release version"))?;

    Ok(version)
}

fn download_and_replace_binary(version: &str) -> io::Result<()> {
    let arch = match env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "arm64",
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Unsupported architecture",
            ));
        }
    };

    let os = match env::consts::OS {
        "linux" => "linux",
        "macos" => "macos",
        _ => return Err(io::Error::new(io::ErrorKind::Other, "Unsupported OS")),
    };

    let url = format!(
        "https://github.com/{}/releases/download/v{}/nodash-{}-{}",
        REPO, version, os, arch
    );

    let tmp_path = env::temp_dir().join("nodash-update");
    let status = Command::new("curl")
        .args(&["-L", "-o"])
        .arg(&tmp_path)
        .arg(&url)
        .status()?;

    if !status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "Download failed"));
    }

    let current_path = env::current_exe()?;
    fs::copy(&tmp_path, &current_path).map_err(|e| {
        io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("Failed to replace binary at {:?}: {}", current_path, e),
        )
    })?;
    Ok(())
}
