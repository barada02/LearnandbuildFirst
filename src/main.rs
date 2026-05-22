mod task;
mod cli;
mod app;
mod tui;
mod ui;

use clap::Parser;
use cli::{Cli, Commands, TaskSubcommand};
use task::TaskStore;
use app::{App, ActiveTab};
use tui::Tui;
use std::time::Duration;
use crossterm::event::{KeyCode, KeyModifiers, KeyEventKind};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Parse command-line arguments using clap
    let args = Cli::parse();

    // 2. Handle direct CLI subcommands if any were passed
    if let Some(command) = args.command {
        let mut store = TaskStore::new();
        match command {
            Commands::Task { subcommand } => match subcommand {
                TaskSubcommand::Add { title } => {
                    let id = store.add_task(title.clone())?;
                    println!("🚀 Successfully added task #{} : \"{}\"", id, title);
                }
                TaskSubcommand::Toggle { id } => {
                    match store.toggle_task(id)? {
                        true => {
                            let task = store.tasks.iter().find(|t| t.id == id).unwrap();
                            let status = if task.completed { "✅ Completed" } else { "⏳ Pending" };
                            println!("🔄 Task #{} is now marked as: {}", id, status);
                        }
                        false => println!("❌ Error: Task with ID {} not found.", id),
                    }
                }
                TaskSubcommand::Delete { id } => {
                    match store.delete_task(id)? {
                        true => println!("🗑️ Task #{} successfully deleted.", id),
                        false => println!("❌ Error: Task with ID {} not found.", id),
                    }
                }
                TaskSubcommand::List => {
                    if store.tasks.is_empty() {
                        println!("📝 Your task list is currently empty! Add one using: cargo run -- task add \"your task\"");
                        return Ok(());
                    }

                    println!("\n========= 📝 CURRENT TASKS =========");
                    println!("{:<5} | {:<30} | {:<10} | {:<20}", "ID", "Title", "Status", "Created At");
                    println!("{:-<5}-+-{:-<30}-+-{:-<10}-+-{:-<20}", "", "", "", "");
                    
                    for task in &store.tasks {
                        let status_symbol = if task.completed { "✅ Done" } else { "⏳ Pending" };
                        println!(
                            "{:<5} | {:<30} | {:<10} | {:<20}",
                            task.id, task.title, status_symbol, task.created_at
                        );
                    }
                    println!("====================================\n");
                }
            },
        }
    } else {
        // 3. Launch full interactive visual TUI Dashboard!
        let mut app = App::new();
        let mut tui = Tui::new()?;

        // Enter rendering loop
        while !app.should_quit {
            // Draw standard visual frame layout
            tui.terminal.draw(|f| ui::render(f, &mut app))?;

            // Poll keyboard events with a 100ms timeout
            if let Some(key) = tui.poll_event(Duration::from_millis(100))? {
                // On Windows, Crossterm registers both key presses and key releases.
                // We ONLY want to handle standard Press events to prevent double-triggers!
                if key.kind == KeyEventKind::Press {
                    // If keyboard text modal is visible, route text characters
                    if app.show_add_popup {
                        match key.code {
                            KeyCode::Enter => {
                                app.add_task_from_input();
                            }
                            KeyCode::Esc => {
                                app.show_add_popup = false;
                                app.input_text.clear();
                            }
                            KeyCode::Backspace => {
                                app.input_text.pop();
                            }
                            KeyCode::Char(c) => {
                                // Only append if it's not a control code
                                app.input_text.push(c);
                            }
                            _ => {}
                        }
                    } else {
                        // Else, standard normal hotkey navigation
                        match key.code {
                            // Quit bindings
                            KeyCode::Char('q') | KeyCode::Esc => {
                                app.quit();
                            }
                            // Handle Ctrl+C manually to guarantee clean terminal exit!
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                app.quit();
                            }
                            // Tab Switching
                            KeyCode::Tab => {
                                app.next_tab();
                            }
                            KeyCode::Char('1') => {
                                app.active_tab = ActiveTab::Dashboard;
                            }
                            KeyCode::Char('2') => {
                                app.active_tab = ActiveTab::Tasks;
                            }
                            KeyCode::Char('3') => {
                                app.active_tab = ActiveTab::SystemInfo;
                            }
                            // Navigation in Lists
                            KeyCode::Down | KeyCode::Char('j') => {
                                if app.active_tab == ActiveTab::Tasks {
                                    app.list_next();
                                }
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                if app.active_tab == ActiveTab::Tasks {
                                    app.list_prev();
                                }
                            }
                            // Task Actions (Only works on the Task List Tab)
                            KeyCode::Char(' ') => {
                                if app.active_tab == ActiveTab::Tasks {
                                    app.toggle_selected_task();
                                }
                            }
                            KeyCode::Delete | KeyCode::Char('d') => {
                                if app.active_tab == ActiveTab::Tasks {
                                    app.delete_selected_task();
                                }
                            }
                            KeyCode::Char('a') => {
                                if app.active_tab == ActiveTab::Tasks {
                                    app.show_add_popup = true;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
