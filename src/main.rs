mod task;
mod cli;

use clap::Parser;
use cli::{Cli, Commands, TaskSubcommand};
use task::TaskStore;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Parse terminal command-line arguments using clap
    let args = Cli::parse();

    // 2. Initialize our database/store
    let mut store = TaskStore::new();

    // 3. Handle subcommands if any were passed
    if let Some(command) = args.command {
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
        // 4. No subcommand passed! Launch the full TUI Dashboard
        println!("✨ Launching visual interactive TUI Dashboard... (Setup in progress!)");
        println!("💡 Tip: You can run direct commands too. Try running: cargo run -- --help");
    }

    Ok(())
}
