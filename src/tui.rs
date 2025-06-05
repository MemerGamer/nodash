use std::io::{self, Write};
use std::path::PathBuf;
use crossterm::{event, execute, terminal};
use crossterm::event::{Event, KeyCode};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Terminal;

use crate::project::{Project, save_projects};

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

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .constraints([Constraint::Min(5), Constraint::Length(3)].as_ref())
                .split(f.area());

            let items: Vec<ListItem> = projects.iter().enumerate().map(|(i, p)| {
                let mut line = format!("{}: {}", i + 1, p.name);
                if let Some(ts) = p.last_opened {
                    line.push_str(&format!(" (last opened: {})", ts.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M")));
                }
                ListItem::new(line)
            }).collect();

            let list = List::new(items)
                .block(Block::default().title("Nodash Projects").borders(Borders::ALL))
                .highlight_style(Style::default().fg(Color::Yellow).bg(Color::Blue));

            f.render_stateful_widget(list, chunks[0], &mut list_state);

            let help_text = vec![
                ListItem::new("↑/↓: Navigate   Enter: Open   a: Add   q: Quit"),
            ];
            let help = List::new(help_text)
                .block(Block::default().title("Controls").borders(Borders::ALL));

            f.render_widget(help, chunks[1]);
        })?;

        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Down => {
                        selected = (selected + 1).min(projects.len().saturating_sub(1));
                        list_state.select(Some(selected));
                    }
                    KeyCode::Up => {
                        if selected > 0 {
                            selected -= 1;
                            list_state.select(Some(selected));
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(proj) = projects.get_mut(selected) {
                            open_cb(proj)?;
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

                        let project = Project {
                            name: name.trim().to_string(),
                            path: PathBuf::from(path.trim()),
                            last_opened: None,
                        };

                        projects.push(project);
                        selected = projects.len().saturating_sub(1);
                        list_state.select(Some(selected));
                        save_projects(projects)?;

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

    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), terminal::LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
