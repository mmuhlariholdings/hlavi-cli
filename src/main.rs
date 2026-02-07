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

    /// Manage tickets
    #[command(subcommand)]
    Tickets(commands::tickets::TicketsCommand),

    /// Manage and view the kanban board
    #[command(subcommand)]
    Board(commands::board::BoardCommand),

    /// Manage AI agent configuration and execution
    #[command(subcommand)]
    Agent(commands::agent::AgentCommand),
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init => commands::init::execute().await,
        Commands::Tickets(cmd) => commands::tickets::execute(cmd).await,
        Commands::Board(cmd) => commands::board::execute(cmd).await,
        Commands::Agent(cmd) => commands::agent::execute(cmd).await,
    };

    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        process::exit(1);
    }
}
