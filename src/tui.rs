use crossterm::event::{Event, KeyCode};
use crossterm::{event, execute, terminal};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use std::io::{self, Write};
use std::path::PathBuf;

use crate::project::{Project, detect_node_version, save_projects};

const HIGHLIGHT_COLOR: Color = Color::LightCyan;
const ACCENT_COLOR: Color = Color::LightGreen;
const TEXT_COLOR: Color = Color::White;
const MUTED_COLOR: Color = Color::DarkGray;
const ERROR_COLOR: Color = Color::Red;

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

        if selected >= filtered_indices.len() && !filtered_indices.is_empty() {
            selected = filtered_indices.len() - 1;
        }
        list_state.select(if filtered_indices.is_empty() {
            None
        } else {
            Some(selected)
        });

        terminal.draw(|f| {
            // Main layout: Content Area + Footer
            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),    // Content
                    Constraint::Length(3), // Footer
                ])
                .split(f.area());

            // Content area
            let content_layout = if search_mode {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3), // Search box
                        Constraint::Min(5),    // Project list
                    ])
                    .margin(1) // Small margin around content for spacing
                    .split(main_layout[0])
            } else {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(5)])
                    .margin(1) // Small margin around content for spacing
                    .split(main_layout[0])
            };

            let mut content_idx = 0;

            // Search input (only if in search mode)
            if search_mode {
                let search_text = if search_query.is_empty() {
                    "Type to search projects...".to_string()
                } else {
                    search_query.clone()
                };

                let search_style = if search_query.is_empty() {
                    Style::default()
                        .fg(MUTED_COLOR)
                        .add_modifier(Modifier::ITALIC)
                } else {
                    Style::default().fg(TEXT_COLOR)
                };

                let search_input = Paragraph::new(search_text).style(search_style).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(HIGHLIGHT_COLOR))
                        .title(" Search Projects ") // Simplified title
                        .title_style(
                            Style::default()
                                .fg(ACCENT_COLOR)
                                .add_modifier(Modifier::BOLD),
                        ),
                );
                f.render_widget(search_input, content_layout[content_idx]);
                content_idx += 1;
            }

            // Project list
            let items: Vec<ListItem> = if filtered_indices.is_empty() {
                vec![ListItem::new(Line::from(vec![Span::styled(
                    "No projects found.",
                    Style::default()
                        .fg(MUTED_COLOR)
                        .add_modifier(Modifier::ITALIC),
                )]))]
            } else {
                filtered_indices
                    .iter()
                    .filter_map(|&idx| projects.get(idx))
                    .enumerate()
                    .map(|(display_idx, p)| {
                        let original_idx = filtered_indices[display_idx]; // Keep original index for display numbering

                        let mut spans = vec![
                            Span::styled(
                                format!("{}. ", original_idx + 1),
                                Style::default().fg(MUTED_COLOR),
                            ),
                            Span::styled(
                                &p.name,
                                Style::default().fg(TEXT_COLOR).add_modifier(Modifier::BOLD),
                            ),
                        ];

                        // Add node version
                        if let Some(ref version) = p.node_version {
                            spans.push(Span::styled(" (Node ", Style::default().fg(MUTED_COLOR)));
                            spans.push(Span::styled(version, Style::default().fg(ACCENT_COLOR)));
                            spans.push(Span::styled(")", Style::default().fg(MUTED_COLOR)));
                        }

                        // Add last opened date
                        if let Some(ts) = p.last_opened {
                            spans.push(Span::styled(" - ", Style::default().fg(MUTED_COLOR)));
                            spans.push(Span::styled(
                                ts.with_timezone(&chrono::Local)
                                    .format("%Y-%m-%d %H:%M")
                                    .to_string(),
                                Style::default().fg(MUTED_COLOR),
                            ));
                        }

                        ListItem::new(Line::from(spans))
                    })
                    .collect()
            };

            let list_title_text = if projects.is_empty() {
                " Projects (No projects yet) ".to_string()
            } else if search_mode && !search_query.is_empty() {
                format!(" Projects ({}/{}) ", filtered_indices.len(), projects.len())
            } else {
                " Projects ".to_string()
            };

            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(HIGHLIGHT_COLOR))
                        .title(list_title_text)
                        .title_style(
                            Style::default()
                                .fg(HIGHLIGHT_COLOR)
                                .add_modifier(Modifier::BOLD),
                        ),
                )
                .highlight_style(
                    Style::default()
                        .fg(Color::Black) // Text on highlight
                        .bg(HIGHLIGHT_COLOR) // Highlighted background
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("❯ "); // Simple, modern arrow

            f.render_stateful_widget(list, content_layout[content_idx], &mut list_state);

            // Footer with controls
            let footer_text = if search_mode {
                vec![
                    Span::styled(
                        "ESC",
                        Style::default()
                            .fg(HIGHLIGHT_COLOR)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" exit search", Style::default().fg(TEXT_COLOR)),
                    Span::raw(" | "),
                    Span::styled(
                        "↑↓",
                        Style::default()
                            .fg(HIGHLIGHT_COLOR)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" navigate", Style::default().fg(TEXT_COLOR)),
                    Span::raw(" | "),
                    Span::styled(
                        "ENTER",
                        Style::default()
                            .fg(ACCENT_COLOR)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" open", Style::default().fg(TEXT_COLOR)),
                    Span::raw(" | "),
                    Span::styled(
                        "Q",
                        Style::default()
                            .fg(ERROR_COLOR)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" quit", Style::default().fg(TEXT_COLOR)),
                ]
            } else {
                vec![
                    Span::styled(
                        "↑↓",
                        Style::default()
                            .fg(HIGHLIGHT_COLOR)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" navigate", Style::default().fg(TEXT_COLOR)),
                    Span::raw(" | "),
                    Span::styled(
                        "ENTER",
                        Style::default()
                            .fg(ACCENT_COLOR)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" open", Style::default().fg(TEXT_COLOR)),
                    Span::raw(" | "),
                    Span::styled(
                        "A",
                        Style::default()
                            .fg(HIGHLIGHT_COLOR)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" add", Style::default().fg(TEXT_COLOR)),
                    Span::raw(" | "),
                    Span::styled(
                        "/",
                        Style::default()
                            .fg(HIGHLIGHT_COLOR)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" search", Style::default().fg(TEXT_COLOR)),
                    Span::raw(" | "),
                    Span::styled(
                        "Q",
                        Style::default()
                            .fg(ERROR_COLOR)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" quit", Style::default().fg(TEXT_COLOR)),
                ]
            };

            let footer = Paragraph::new(Line::from(footer_text))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(MUTED_COLOR))
                        .title(" Commands ")
                        .title_style(
                            Style::default()
                                .fg(MUTED_COLOR)
                                .add_modifier(Modifier::BOLD),
                        ),
                );
            f.render_widget(footer, main_layout[1]);
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
                            println!("---------------");
                            let mut name = String::new();
                            let mut path = String::new();

                            print!("Project name: ");
                            io::stdout().flush()?;
                            io::stdin().read_line(&mut name)?;

                            print!("Project path: ");
                            io::stdout().flush()?;
                            io::stdin().read_line(&mut path)?;

                            let project_path = PathBuf::from(path.trim());
                            let node_version = detect_node_version(&project_path).ok();

                            let project = Project {
                                name: name.trim().to_string(),
                                path: project_path.clone(),
                                last_opened: None,
                                node_version: node_version.clone(),
                            };

                            projects.push(project);
                            save_projects(projects)?;
                            selected = 0;

                            println!("\nProject added.");
                            if let Some(version) = node_version {
                                println!("Node version detected: {}", version);
                            } else {
                                println!("No .nvmrc file found.");
                            }
                            println!("\nPress Enter to return to dashboard...");
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
