use crossterm::event::{Event, KeyCode};
use crossterm::{event, execute, terminal};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use std::io::{self, Write};
use std::path::PathBuf;

use crate::project::{Project, detect_node_version, save_projects};

pub fn run_app<F>(projects: &mut Vec<Project>, mut open_cb: F) -> io::Result<()>
where
    F: FnMut(&mut Project) -> io::Result<()>,
{
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal::enable_raw_mode()?;
    execute!(terminal.backend_mut(), terminal::EnterAlternateScreen)?;
    terminal.clear()?;

    let mut selected = 0;
    let mut list_state = ListState::default();
    list_state.select(Some(selected));

    let mut search_query = String::new();
    let mut search_mode = false;

    loop {
        // Filter projects based on search query - collect indices instead of references
        let filtered_indices: Vec<usize> = projects
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                if search_query.is_empty() {
                    true
                } else {
                    p.name.to_lowercase().contains(&search_query.to_lowercase())
                        || p.path
                            .to_string_lossy()
                            .to_lowercase()
                            .contains(&search_query.to_lowercase())
                }
            })
            .map(|(idx, _)| idx)
            .collect();

        // Adjust selected index if it's out of bounds
        if selected >= filtered_indices.len() && !filtered_indices.is_empty() {
            selected = filtered_indices.len() - 1;
        }
        list_state.select(if filtered_indices.is_empty() {
            None
        } else {
            Some(selected)
        });

        terminal.draw(|f| {
            let chunks = if search_mode {
                Layout::default()
                    .constraints(
                        [
                            Constraint::Length(3),
                            Constraint::Min(5),
                            Constraint::Length(3),
                        ]
                        .as_ref(),
                    )
                    .split(f.area())
            } else {
                Layout::default()
                    .constraints([Constraint::Min(5), Constraint::Length(3)].as_ref())
                    .split(f.area())
            };

            let mut chunk_idx = 0;

            // Search input (only if in search mode)
            if search_mode {
                let search_input = Paragraph::new(format!("Search: {}", search_query)).block(
                    Block::default()
                        .title("Search Projects")
                        .borders(Borders::ALL),
                );
                f.render_widget(search_input, chunks[chunk_idx]);
                chunk_idx += 1;
            }

            // Project list
            let items: Vec<ListItem> = filtered_indices
                .iter()
                .filter_map(|&idx| projects.get(idx))
                .enumerate()
                .map(|(display_idx, p)| {
                    let original_idx = filtered_indices[display_idx];
                    let mut line = format!("{}: {}", original_idx + 1, p.name);

                    // Add node version if available
                    if let Some(ref version) = p.node_version {
                        line.push_str(&format!(" (Node: {})", version));
                    }

                    // Add last opened date
                    if let Some(ts) = p.last_opened {
                        line.push_str(&format!(
                            " - {}",
                            ts.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M")
                        ));
                    }

                    ListItem::new(line)
                })
                .collect();

            let list_title = if search_mode && !search_query.is_empty() {
                format!(
                    "Nodash Projects (Filtered: {}/{})",
                    filtered_indices.len(),
                    projects.len()
                )
            } else {
                "Nodash Projects".to_string()
            };

            let list = List::new(items)
                .block(Block::default().title(list_title).borders(Borders::ALL))
                .highlight_style(Style::default().fg(Color::Yellow).bg(Color::Blue));

            f.render_stateful_widget(list, chunks[chunk_idx], &mut list_state);
            chunk_idx += 1;

            // Help text
            let help_text = if search_mode {
                vec![ListItem::new(
                    "Type to search | Esc: Exit search | Enter: Open | q: Quit",
                )]
            } else {
                vec![ListItem::new(
                    "↑/↓: Navigate | Enter: Open | a: Add | /: Search | q: Quit",
                )]
            };

            let help = List::new(help_text)
                .block(Block::default().title("Controls").borders(Borders::ALL));

            f.render_widget(help, chunks[chunk_idx]);
        })?;

        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if search_mode {
                    match key.code {
                        KeyCode::Esc => {
                            search_mode = false;
                            search_query.clear();
                            selected = 0;
                        }
                        KeyCode::Enter => {
                            if let Some(&original_idx) = filtered_indices.get(selected) {
                                if let Some(proj) = projects.get_mut(original_idx) {
                                    open_cb(proj)?;
                                    // No need to sort here, save_projects will handle it
                                    save_projects(projects)?;
                                    selected = 0;
                                }
                            }
                        }
                        KeyCode::Backspace => {
                            search_query.pop();
                            selected = 0;
                        }
                        KeyCode::Down => {
                            if !filtered_indices.is_empty() {
                                selected =
                                    (selected + 1).min(filtered_indices.len().saturating_sub(1));
                            }
                        }
                        KeyCode::Up => {
                            if selected > 0 {
                                selected -= 1;
                            }
                        }
                        KeyCode::Char(c) => {
                            search_query.push(c);
                            selected = 0;
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('/') => {
                            search_mode = true;
                            search_query.clear();
                            selected = 0;
                        }
                        KeyCode::Down => {
                            if !filtered_indices.is_empty() {
                                selected =
                                    (selected + 1).min(filtered_indices.len().saturating_sub(1));
                            }
                        }
                        KeyCode::Up => {
                            if selected > 0 {
                                selected -= 1;
                            }
                        }
                        KeyCode::Enter => {
                            if let Some(&original_idx) = filtered_indices.get(selected) {
                                if let Some(proj) = projects.get_mut(original_idx) {
                                    open_cb(proj)?;
                                    // No need to sort here, save_projects will handle it
                                    save_projects(projects)?;
                                    selected = 0;
                                }
                            }
                        }
                        KeyCode::Char('a') => {
                            terminal::disable_raw_mode()?;
                            execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
                            terminal.show_cursor()?;

                            println!("\nAdd New Project");
                            let mut name = String::new();
                            let mut path = String::new();

                            print!("Project name: ");
                            io::stdout().flush()?;
                            io::stdin().read_line(&mut name)?;

                            print!("Project path: ");
                            io::stdout().flush()?;
                            io::stdin().read_line(&mut path)?;

                            let project_path = PathBuf::from(path.trim());
                            let project = Project {
                                name: name.trim().to_string(),
                                path: project_path.clone(),
                                last_opened: None,
                                node_version: detect_node_version(&project_path).ok(),
                            };

                            projects.push(project);

                            // No need to sort here, save_projects will handle it
                            save_projects(projects)?;
                            selected = 0;

                            println!("\nProject added. Press Enter to return...");
                            let mut dummy = String::new();
                            io::stdin().read_line(&mut dummy)?;

                            terminal.clear()?;
                            execute!(terminal.backend_mut(), terminal::EnterAlternateScreen)?;
                            terminal::enable_raw_mode()?;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
