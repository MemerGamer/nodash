use crate::project::{Project, detect_node_version};
use chrono::Utc;
use std::env;
use std::io;
use std::process::Command;

fn get_current_shell() -> String {
    // Try SHELL environment variable first
    if let Ok(shell) = env::var("SHELL") {
        return shell;
    }

    // Fallback to bash
    "/bin/bash".to_string()
}

fn is_fish_shell(shell: &str) -> bool {
    shell.contains("fish")
}

fn get_terminal_emulator() -> Option<String> {
    // Check common terminal environment variables
    if env::var("KITTY_WINDOW_ID").is_ok() {
        return Some("kitty".to_string());
    }

    if env::var("ALACRITTY_SOCKET").is_ok() || env::var("ALACRITTY_LOG").is_ok() {
        return Some("alacritty".to_string());
    }

    if env::var("WEZTERM_EXECUTABLE").is_ok() {
        return Some("wezterm".to_string());
    }

    if env::var("TERM_PROGRAM").as_deref() == Ok("iTerm.app") {
        return Some("iterm2".to_string());
    }

    if env::var("TERM_PROGRAM").as_deref() == Ok("Apple_Terminal") {
        return Some("terminal".to_string());
    }

    // Check if we're in tmux or screen
    if env::var("TMUX").is_ok() {
        return Some("tmux".to_string());
    }

    if env::var("STY").is_ok() {
        return Some("screen".to_string());
    }

    None
}

fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn open_project(proj: &mut Project) -> io::Result<()> {
    proj.last_opened = Some(Utc::now());

    // Update node version when opening project
    proj.node_version = detect_node_version(&proj.path).ok();

    let shell = get_current_shell();

    // Create shell-specific commands
    let nvm_command = if is_fish_shell(&shell) {
        format!(
            "cd '{}' ; \
            if functions -q nvm\n  nvm use 2>/dev/null; or nvm install\n\
            else if command -q fnm\n  fnm use 2>/dev/null; or fnm install\n\
            else if command -q node\n  echo 'Node.js available'\n\
            else\n  echo 'No Node.js version manager found'\n\
            end ; exec {}",
            proj.path.display(),
            shell
        )
    } else {
        // Bash/Zsh: robust nvm init with fallbacks, then use .nvmrc or install
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
            fi && exec {} -i",
            proj.path.display(),
            shell
        )
    };

    match get_terminal_emulator() {
        Some(terminal) => match terminal.as_str() {
            "kitty" => {
                if command_exists("kitty") {
                    Command::new("kitty")
                        .arg("--hold")
                        .arg("--")
                        .arg(&shell)
                        .arg("-i")
                        .arg("-c")
                        .arg(&nvm_command)
                        .spawn()?;
                } else {
                    return Err(io::Error::new(io::ErrorKind::NotFound, "kitty not found"));
                }
            }
            "alacritty" => {
                if command_exists("alacritty") {
                    Command::new("alacritty")
                        .arg("-e")
                        .arg(&shell)
                        .arg("-i")
                        .arg("-c")
                        .arg(&nvm_command)
                        .spawn()?;
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        "alacritty not found",
                    ));
                }
            }
            "wezterm" => {
                if command_exists("wezterm") {
                    Command::new("wezterm")
                        .arg("start")
                        .arg("--")
                        .arg(&shell)
                        .arg("-i")
                        .arg("-c")
                        .arg(&nvm_command)
                        .spawn()?;
                } else {
                    return Err(io::Error::new(io::ErrorKind::NotFound, "wezterm not found"));
                }
            }
            "tmux" => {
                if command_exists("tmux") {
                    let tmux_command = if is_fish_shell(&shell) {
                        format!(
                            "cd '{}' && if functions -q nvm; \
                            nvm use 2>/dev/null; \
                            or nvm install; \
                            else if command -q fnm; \
                            fnm use 2>/dev/null; \
                            or fnm install; \
                            else if command -q node; \
                            echo 'Node.js available'; \
                            else \
                            echo 'No Node.js version manager found'; \
                            end; \
                            end; \
                            end",
                            proj.path.display()
                        )
                    } else {
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
                            fi",
                            proj.path.display()
                        )
                    };
                    Command::new("tmux")
                        .arg("new-window")
                        .arg("-c")
                        .arg(&proj.path)
                        .arg(&shell)
                        .arg("-i")
                        .arg("-c")
                        .arg(&tmux_command)
                        .spawn()?;
                } else {
                    return Err(io::Error::new(io::ErrorKind::NotFound, "tmux not found"));
                }
            }
            _ => {
                if command_exists(&terminal) {
                    Command::new(&terminal)
                        .arg("-e")
                        .arg(&shell)
                        .arg("-i")
                        .arg("-c")
                        .arg(&nvm_command)
                        .spawn()?;
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("{} not found", terminal),
                    ));
                }
            }
        },
        None => {
            let terminals = [
                ("kitty", vec!["--hold", "--"]),
                ("alacritty", vec!["-e"]),
                ("wezterm", vec!["start", "--"]),
                ("gnome-terminal", vec!["--"]),
                ("xterm", vec!["-e"]),
            ];

            let mut found = false;
            for (term, args) in &terminals {
                if command_exists(term) {
                    let mut cmd = Command::new(term);
                    for arg in args {
                        cmd.arg(arg);
                    }
                    cmd.arg(&shell).arg("-i").arg("-c").arg(&nvm_command).spawn()?;
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
