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

    /// Manage tickets (defaults to list if no subcommand provided)
    Tickets {
        #[command(subcommand)]
        command: Option<commands::tickets::TicketsCommand>,
    },

    /// Manage and view the kanban board
    #[command(subcommand)]
    Board(commands::board::BoardCommand),

    /// Manage AI agent configuration and execution
    #[command(subcommand)]
    Agent(commands::agent::AgentCommand),

    /// View tickets in a timeline view
    Timeline {
        /// Sort tickets by field (id, title, status, created, updated, start, end)
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
        Commands::Tickets { command } => {
            // Default to List if no subcommand is provided
            let cmd = command.unwrap_or(commands::tickets::TicketsCommand::List {
                sort_by: "id".to_string(),
                sort_order: "asc".to_string(),
            });
            commands::tickets::execute(cmd).await
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
