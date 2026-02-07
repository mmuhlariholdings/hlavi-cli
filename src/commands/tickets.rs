use crate::commands::get_storage;
use clap::Subcommand;
use colored::*;
use hlavi_core::{
    domain::ticket::{Ticket, TicketId},
    storage::Storage,
};
use tabled::{
    settings::{object::Columns, Alignment, Style},
    Table, Tabled,
};

#[derive(Subcommand)]
pub enum TicketsCommand {
    /// List all tickets
    List,

    /// Create a new ticket
    Create {
        /// Title of the ticket
        title: String,
    },

    /// Edit a ticket
    Edit {
        /// Ticket ID (e.g., TIK001)
        id: String,

        /// Set the description
        #[arg(short, long)]
        description: Option<String>,

        /// Add acceptance criteria
        #[arg(long = "ac")]
        add_ac: Option<String>,

        /// Remove acceptance criteria by description or index
        #[arg(long = "remove-ac")]
        remove_ac: Option<String>,
    },

    /// View ticket details
    Show {
        /// Ticket ID (e.g., TIK001)
        id: String,
    },

    /// Delete a ticket
    Delete {
        /// Ticket ID (e.g., TIK001)
        id: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(Tabled)]
struct TicketRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Title")]
    title: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "ACs")]
    acceptance_criteria_count: String,
}

pub async fn execute(cmd: TicketsCommand) -> anyhow::Result<()> {
    let storage = get_storage()?;

    if !storage.is_initialized().await {
        anyhow::bail!(
            "Project not initialized. Run 'hlavi init' first."
        );
    }

    match cmd {
        TicketsCommand::List => list_tickets(&storage).await,
        TicketsCommand::Create { title } => create_ticket(&storage, title).await,
        TicketsCommand::Edit {
            id,
            description,
            add_ac,
            remove_ac,
        } => edit_ticket(&storage, id, description, add_ac, remove_ac).await,
        TicketsCommand::Show { id } => show_ticket(&storage, id).await,
        TicketsCommand::Delete { id, force } => delete_ticket(&storage, id, force).await,
    }
}

async fn list_tickets(storage: &impl Storage) -> anyhow::Result<()> {
    let ticket_ids = storage.list_ticket_ids().await?;

    if ticket_ids.is_empty() {
        println!("{}", "No tickets found.".yellow());
        println!("\nCreate a ticket with:");
        println!("  {} hlavi tickets create \"Your ticket title\"", "$".yellow());
        return Ok(());
    }

    let mut rows = Vec::new();
    for id in ticket_ids {
        let ticket = storage.load_ticket(&id).await?;
        rows.push(TicketRow {
            id: ticket.id.to_string(),
            title: ticket.title.clone(),
            status: ticket.status.to_string(),
            acceptance_criteria_count: format!(
                "{}/{}",
                ticket
                    .acceptance_criteria
                    .iter()
                    .filter(|ac| ac.completed)
                    .count(),
                ticket.acceptance_criteria.len()
            ),
        });
    }

    let mut table = Table::new(rows);
    table
        .with(Style::rounded())
        .modify(Columns::new(2..3), Alignment::left());

    println!("{}", table);

    Ok(())
}

async fn create_ticket(storage: &impl Storage, title: String) -> anyhow::Result<()> {
    let mut board = storage.load_board().await?;
    let ticket_id = board.next_ticket_id();

    let ticket = Ticket::new(ticket_id.clone(), title);

    storage.save_ticket(&ticket).await?;
    board.add_ticket(ticket_id.clone());
    storage.save_board(&board).await?;

    println!(
        "{} Created ticket {}",
        "✓".green().bold(),
        ticket_id.to_string().cyan().bold()
    );

    Ok(())
}

async fn edit_ticket(
    storage: &impl Storage,
    id: String,
    description: Option<String>,
    add_ac: Option<String>,
    remove_ac: Option<String>,
) -> anyhow::Result<()> {
    let ticket_id = TicketId::from_str(&id)?;
    let mut ticket = storage.load_ticket(&ticket_id).await?;

    let mut modified = false;

    if let Some(desc) = description {
        ticket.set_description(desc);
        modified = true;
        println!("{} Updated description", "✓".green().bold());
    }

    if let Some(ac) = add_ac {
        ticket.add_acceptance_criterion(ac.clone());
        modified = true;
        println!(
            "{} Added acceptance criteria: {}",
            "✓".green().bold(),
            ac.cyan()
        );
    }

    if let Some(ac_identifier) = remove_ac {
        ticket.remove_acceptance_criterion(&ac_identifier)?;
        modified = true;
        println!("{} Removed acceptance criteria", "✓".green().bold());
    }

    if !modified {
        println!("{}", "No changes made.".yellow());
        return Ok(());
    }

    storage.save_ticket(&ticket).await?;

    println!(
        "{} Updated ticket {}",
        "✓".green().bold(),
        ticket_id.to_string().cyan().bold()
    );

    Ok(())
}

async fn show_ticket(storage: &impl Storage, id: String) -> anyhow::Result<()> {
    let ticket_id = TicketId::from_str(&id)?;
    let ticket = storage.load_ticket(&ticket_id).await?;

    println!("\n{}", format!("Ticket {}", ticket.id).cyan().bold());
    println!("{}", "─".repeat(50));
    println!("{}: {}", "Title".bold(), ticket.title);
    println!("{}: {}", "Status".bold(), ticket.status);

    if let Some(desc) = &ticket.description {
        println!("\n{}:", "Description".bold());
        println!("{}", desc);
    }

    if !ticket.acceptance_criteria.is_empty() {
        println!("\n{}:", "Acceptance Criteria".bold());
        for ac in &ticket.acceptance_criteria {
            let status = if ac.completed {
                "✓".green().to_string()
            } else {
                "○".white().to_string()
            };
            println!("  {} [{}] {}", status, ac.id, ac.description);
        }
    }

    println!("\n{}:", "Metadata".bold());
    println!("  Created: {}", ticket.created_at.format("%Y-%m-%d %H:%M:%S"));
    println!("  Updated: {}", ticket.updated_at.format("%Y-%m-%d %H:%M:%S"));

    if let Some(reason) = &ticket.rejection_reason {
        println!("\n{}: {}", "Rejection Reason".red().bold(), reason);
    }

    println!();

    Ok(())
}

async fn delete_ticket(storage: &impl Storage, id: String, force: bool) -> anyhow::Result<()> {
    let ticket_id = TicketId::from_str(&id)?;

    // Verify ticket exists
    let _ticket = storage.load_ticket(&ticket_id).await?;

    if !force {
        let confirm = dialoguer::Confirm::new()
            .with_prompt(format!("Delete ticket {}?", ticket_id))
            .default(false)
            .interact()?;

        if !confirm {
            println!("{}", "Cancelled.".yellow());
            return Ok(());
        }
    }

    storage.delete_ticket(&ticket_id).await?;

    println!(
        "{} Deleted ticket {}",
        "✓".green().bold(),
        ticket_id.to_string().cyan().bold()
    );

    Ok(())
}
