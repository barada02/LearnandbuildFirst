use clap::{Parser, Subcommand};

/// A premium, terminal-based Productivity CLI & TUI Dashboard.
/// If you run this without subcommands, it launches the full interactive visual UI!
#[derive(Parser, Debug)]
#[command(name = "flow")]
#[command(author = "DeepMind Developer Pair")]
#[command(version = "1.0")]
#[command(about = "Streamline your daily tasks and workflow directly from your shell", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage tasks directly via standard shell commands
    Task {
        #[command(subcommand)]
        subcommand: TaskSubcommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum TaskSubcommand {
    /// Add a new task
    Add {
        /// The title of the task to add
        title: String,
    },
    /// Toggle a task's completion status by its ID
    Toggle {
        /// The numeric ID of the task
        id: usize,
    },
    /// Delete a task by its ID
    Delete {
        /// The numeric ID of the task
        id: usize,
    },
    /// List all tasks in your database
    List,
}
