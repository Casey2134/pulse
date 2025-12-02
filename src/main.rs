mod app;
mod cli;
mod config;
mod models;
mod providers;
mod ui;

use std::time::{Duration, Instant};

use clap::Parser;
use crossterm::event::{self, Event, KeyCode};

use crate::app::InputMode;
use crate::providers::{Provider, ProxmoxProvider};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::Args::parse();

    let path = std::path::Path::new(&args.config);
    let config = config::load(path)?;

    let mut providers: Vec<Box<dyn Provider>> = Vec::new();

    if let Some(proxmox_configs) = &config.providers.proxmox {
        for proxmox_config in proxmox_configs {
            match ProxmoxProvider::new(proxmox_config) {
                Ok(provider) => {
                    providers.push(Box::new(provider));
                }
                Err(e) => {
                    eprintln!("Failed to create provider '{}': {}", proxmox_config.name, e);
                }
            }
        }
    }

    if providers.is_empty() {
        eprintln!("No providers configured.");
        std::process::exit(1);
    }

    let mut terminal = ratatui::init();

    let mut app = app::App::new();

    app.refresh(&providers);

    let mut last_refresh = Instant::now();
    let refresh_interval = Duration::from_secs(5);

    while app.running {
        terminal.draw(|frame| ui::draw(frame, &app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Handle help popup first - any key closes it
                if app.show_help {
                    app.toggle_help();
                    continue;
                }

                match app.input_mode {
                    InputMode::Search => match key.code {
                        KeyCode::Esc => {
                            app.exit_search_mode();
                            app.clear_search();
                        }
                        KeyCode::Enter => {
                            app.exit_search_mode();
                        }
                        KeyCode::Backspace => {
                            app.pop_search_char();
                        }
                        KeyCode::Char(c) => {
                            app.push_search_char(c);
                        }
                        _ => {}
                    },
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => app.quit(),
                        KeyCode::Tab => app.next_panel(),
                        KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
                        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
                        KeyCode::Char('r') => app.refresh(&providers),
                        KeyCode::Char('s') => app.cycle_sort(),
                        KeyCode::Char('S') => app.toggle_sort_order(),
                        KeyCode::Char('/') => app.enter_search_mode(),
                        KeyCode::Char('?') => app.toggle_help(),
                        KeyCode::Esc => {
                            if !app.search_query.is_empty() {
                                app.clear_search();
                            }
                        }
                        _ => {}
                    },
                }
            }
        }

        if last_refresh.elapsed() >= refresh_interval {
            app.refresh(&providers);
            last_refresh = Instant::now();
        }
    }

    ratatui::restore();
    Ok(())
}
