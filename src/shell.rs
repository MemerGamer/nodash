use crate::project::{Project, detect_node_version};
use chrono::Utc;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn get_current_shell() -> String {
    if let Ok(shell) = env::var("SHELL") {
        return shell;
    }
    "/bin/bash".to_string()
}

fn is_fish_shell(shell: &str) -> bool {
    shell.contains("fish")
}

fn is_zsh_shell(shell: &str) -> bool {
    shell.contains("zsh")
}

fn get_terminal_emulator() -> Option<String> {
    // Detect some popular terminals by environment
    if env::var("KITTY_WINDOW_ID").is_ok() {
        return Some("kitty".to_string());
    }
    if env::var("ALACRITTY_SOCKET").is_ok() || env::var("ALACRITTY_LOG").is_ok() {
        return Some("alacritty".to_string());
    }
    if env::var("WEZTERM_EXECUTABLE").is_ok() {
        return Some("wezterm".to_string());
    }
    if matches!(
        env::var("TERM_PROGRAM").as_deref(),
        Ok("Ghostty") | Ok("ghostty")
    ) {
        return Some("ghostty".to_string());
    }

    // No tmux/screen/Termux special handling
    None
}

fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

// Escape a path for single-quoted shell contexts: ' -> '\'' pattern
fn sh_escape_single_quoted(s: &str) -> String {
    s.replace('\'', r#"'\''"#)
}

// Create a temporary ZDOTDIR with a .zshrc shim that:
// 1) sources user's ~/.zshrc
// 2) cd's into the project
// 3) initializes nvm/fnm and runs nvm use (or install) last
fn create_zsh_shim(project_path: &Path) -> io::Result<PathBuf> {
    let base = env::temp_dir();
    let unique = format!(
        "nodash-zsh-{}-{}",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    );
    let zdotdir = base.join(unique);
    fs::create_dir_all(&zdotdir)?;

    let mut zshrc = String::new();
    let proj = sh_escape_single_quoted(&project_path.display().to_string());

    zshrc.push_str(
        r#"
# nodash zsh shim
# Load user's regular zshrc if present
if [ -f "$HOME/.zshrc" ]; then
  . "$HOME/.zshrc"
fi

"#,
    );

    zshrc.push_str(&format!("cd '{}'\n\n", proj));

    zshrc.push_str(
        r#"export NVM_DIR="${NVM_DIR:-$HOME/.nvm}"
if [ -s "$NVM_DIR/nvm.sh" ]; then
  . "$NVM_DIR/nvm.sh"
elif [ -s /usr/share/nvm/init-nvm.sh ]; then
  . /usr/share/nvm/init-nvm.sh
elif [ -s "$HOME/.config/nvm/nvm.sh" ]; then
  . "$HOME/.config/nvm/nvm.sh"
fi

if command -v nvm >/dev/null 2>&1; then
  nvm use >/dev/null 2>&1 || nvm install
elif command -v fnm >/dev/null 2>&1; then
  eval "$(fnm env)"
  fnm use >/dev/null 2>&1 || fnm install
elif command -v node >/dev/null 2>&1; then
  echo 'Node.js available'
else
  echo 'No Node.js version manager found'
fi

# Refresh command hash
hash -r
"#,
    );

    fs::write(zdotdir.join(".zshrc"), zshrc)?;
    Ok(zdotdir)
}

pub fn open_project(proj: &mut Project) -> io::Result<()> {
    proj.last_opened = Some(Utc::now());
    proj.node_version = detect_node_version(&proj.path).ok();

    let shell = get_current_shell();
    let is_fish = is_fish_shell(&shell);
    let is_zsh = is_zsh_shell(&shell);

    // Prepare zsh shim if we are launching zsh
    let zdotdir = if is_zsh {
        Some(create_zsh_shim(&proj.path)?)
    } else {
        None
    };

    // Build the command string for non-zsh POSIX shells (bash, sh, etc.)
    // We intentionally DO NOT "exec {shell}" at the end. We start an
    // interactive shell as a child ("{shell} -i") to avoid losing PATH.
    let nvm_command = if is_fish {
        // fish branch
        format!(
            "cd '{}' ; \
             if functions -q nvm\n  nvm use 2>/dev/null; or nvm install\n\
             else if command -q fnm\n  fnm use 2>/dev/null; or fnm install\n\
             else if command -q node\n  echo 'Node.js available'\n\
             else\n  echo 'No Node.js version manager found'\n\
             end ; {}",
            proj.path.display(),
            shell
        )
    } else if !is_zsh {
        // bash/sh branch
        format!(
            "cd '{}' && \
             export NVM_DIR=\"${{NVM_DIR:-$HOME/.nvm}}\"; \
             if [ -s \"$NVM_DIR/nvm.sh\" ]; then \
               . \"$NVM_DIR/nvm.sh\"; \
             elif [ -s /usr/share/nvm/init-nvm.sh ]; then \
               . /usr/share/nvm/init-nvm.sh; \
             elif [ -s \"$HOME/.config/nvm/nvm.sh\" ]; then \
               . \"$HOME/.config/nvm/nvm.sh\"; \
             fi; \
             if command -v nvm >/dev/null 2>&1; then \
               (nvm use 2>/dev/null || nvm install); \
             elif command -v fnm >/dev/null 2>&1; then \
               eval \"$(fnm env)\" && (fnm use 2>/dev/null || fnm install); \
             elif command -v node >/dev/null 2>&1; then \
               echo 'Node.js available'; \
             else \
               echo 'No Node.js version manager found'; \
             fi; \
             hash -r; \
             {} -i",
            proj.path.display(),
            shell
        )
    } else {
        // zsh is handled via ZDOTDIR shim; we won't pass a "-c" command
        String::new()
    };

    match get_terminal_emulator() {
        Some(terminal) => match terminal.as_str() {
            "kitty" => {
                if !command_exists("kitty") {
                    return Err(io::Error::new(io::ErrorKind::NotFound, "kitty not found"));
                }
                let mut cmd = Command::new("kitty");
                cmd.arg("--hold").arg("--");
                // Silence kitty noises, no more pspsps
                cmd.stdout(Stdio::null()).stderr(Stdio::null());

                if is_zsh {
                    if let Some(zd) = &zdotdir {
                        cmd.env("ZDOTDIR", zd);
                    }
                    cmd.arg(&shell).arg("-i");
                } else {
                    cmd.arg(&shell).arg("-i").arg("-c").arg(&nvm_command);
                }
                cmd.spawn()?;
            }
            "alacritty" => {
                if !command_exists("alacritty") {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        "alacritty not found",
                    ));
                }
                let mut cmd = Command::new("alacritty");
                if is_zsh {
                    if let Some(zd) = &zdotdir {
                        cmd.env("ZDOTDIR", zd);
                    }
                    cmd.arg("-e").arg(&shell).arg("-i");
                } else {
                    cmd.arg("-e")
                        .arg(&shell)
                        .arg("-i")
                        .arg("-c")
                        .arg(&nvm_command);
                }
                cmd.spawn()?;
            }
            "wezterm" => {
                if !command_exists("wezterm") {
                    return Err(io::Error::new(io::ErrorKind::NotFound, "wezterm not found"));
                }
                let mut cmd = Command::new("wezterm");
                cmd.arg("start").arg("--");
                // Silence wezterm info/warnings
                cmd.stdout(Stdio::null()).stderr(Stdio::null());

                if is_zsh {
                    if let Some(zd) = &zdotdir {
                        cmd.env("ZDOTDIR", zd);
                    }
                    cmd.arg(&shell).arg("-i");
                } else {
                    cmd.arg(&shell).arg("-i").arg("-c").arg(&nvm_command);
                }
                cmd.spawn()?;
            }
            "ghostty" => {
                if !command_exists("ghostty") {
                    return Err(io::Error::new(io::ErrorKind::NotFound, "ghostty not found"));
                }
                let mut cmd = Command::new("ghostty");
                // Silence ghostty info/warnings
                cmd.stdout(Stdio::null()).stderr(Stdio::null());

                if is_zsh {
                    if let Some(zd) = &zdotdir {
                        cmd.env("ZDOTDIR", zd);
                    }
                    // Linux supports -e to run a command
                    cmd.arg("-e").arg(&shell).arg("-i");
                } else {
                    cmd.arg("-e")
                        .arg(&shell)
                        .arg("-i")
                        .arg("-c")
                        .arg(&nvm_command);
                }
                cmd.spawn()?;
            }
            _ => {
                // Unknown detected terminal; try generic "-e"
                if !command_exists(&terminal) {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("{} not found", terminal),
                    ));
                }
                let mut cmd = Command::new(&terminal);
                // Silence generic terminal noise
                cmd.stdout(Stdio::null()).stderr(Stdio::null());

                if is_zsh {
                    if let Some(zd) = &zdotdir {
                        cmd.env("ZDOTDIR", zd);
                    }
                    cmd.arg("-e").arg(&shell).arg("-i");
                } else {
                    cmd.arg("-e")
                        .arg(&shell)
                        .arg("-i")
                        .arg("-c")
                        .arg(&nvm_command);
                }
                cmd.spawn()?;
            }
        },
        None => {
            // Fallback order:
            // 1) xterm
            // 2) gnome-terminal
            // 3) konsole
            let terminals = [
                ("xterm", vec!["-e"]),
                ("gnome-terminal", vec!["--"]),
                ("konsole", vec!["-e"]),
            ];

            let mut found = false;
            for (term, args) in &terminals {
                if command_exists(term) {
                    let mut cmd = Command::new(term);
                    for arg in args {
                        cmd.arg(arg);
                    }
                    // Silence fallback terminal noise
                    cmd.stdout(Stdio::null()).stderr(Stdio::null());

                    if is_zsh {
                        if let Some(zd) = &zdotdir {
                            cmd.env("ZDOTDIR", zd);
                        }
                        cmd.arg(&shell).arg("-i");
                    } else {
                        cmd.arg(&shell).arg("-i").arg("-c").arg(&nvm_command);
                    }
                    cmd.spawn()?;
                    found = true;
                    break;
                }
            }

            if !found {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "No suitable terminal emulator found",
                ));
            }
        }
    }

    Ok(())
}
