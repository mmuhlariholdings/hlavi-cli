mod commands;

use clap::{Parser, Subcommand};
use colored::*;
use std::process;

#[derive(Parser)]
#[command(name = "hlavi")]
#[command(version, about = "CLI-based kanban task management with AI agent support", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Hlavi project in the current directory
    Init,

    /// Manage tasks (defaults to list if no subcommand provided)
    Tasks {
        #[command(subcommand)]
        command: Option<Box<commands::tasks::TasksCommand>>,
    },

    /// Manage and view the kanban board
    #[command(subcommand)]
    Board(commands::board::BoardCommand),

    /// Manage AI agent configuration and execution
    #[command(subcommand)]
    Agent(commands::agent::AgentCommand),

    /// View tasks in a timeline view
    Timeline {
        /// Sort tasks by field (id, title, status, created, updated, start, end)
        #[arg(long)]
        sort_by: Option<String>,

        /// Sort order (asc or desc)
        #[arg(long)]
        sort_order: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init => commands::init::execute().await,
        Commands::Tasks { command } => {
            // Default to List if no subcommand is provided
            let cmd = command.map(|c| *c).unwrap_or(commands::tasks::TasksCommand::List {
                sort_by: "id".to_string(),
                sort_order: "asc".to_string(),
            });
            commands::tasks::execute(cmd).await
        }
        Commands::Board(cmd) => commands::board::execute(cmd).await,
        Commands::Agent(cmd) => commands::agent::execute(cmd).await,
        Commands::Timeline {
            sort_by,
            sort_order,
        } => commands::timeline::execute(sort_by, sort_order).await,
    };

    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        process::exit(1);
    }
}
