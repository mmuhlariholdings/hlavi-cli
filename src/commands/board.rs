use clap::Subcommand;

#[derive(Subcommand)]
pub enum BoardCommand {
    /// Show the kanban board
    Show,

    /// Configure board columns
    Configure,
}

pub async fn execute(cmd: BoardCommand) -> anyhow::Result<()> {
    match cmd {
        BoardCommand::Show => {
            println!("Board view - Coming soon!");
            Ok(())
        }
        BoardCommand::Configure => {
            println!("Board configuration - Coming soon!");
            Ok(())
        }
    }
}
