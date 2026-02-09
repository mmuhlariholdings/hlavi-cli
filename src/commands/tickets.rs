use crate::commands::get_storage;
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use clap::Subcommand;
use colored::*;
use hlavi_core::{
    domain::ticket::{Ticket, TicketId},
    storage::Storage,
};
use std::str::FromStr;
use tabled::{
    settings::{object::Columns, Alignment, Style},
    Table, Tabled,
};

#[derive(Subcommand)]
pub enum TicketsCommand {
    /// List all tickets
    List {
        /// Sort tickets by field (id, title, status, created, updated, start, end, ac-progress, ac-count)
        #[arg(long, default_value = "id")]
        sort_by: String,

        /// Sort order (asc or desc)
        #[arg(long, default_value = "asc")]
        sort_order: String,
    },

    /// Create a new ticket
    Create {
        /// Title of the ticket
        title: String,
    },

    /// Edit a ticket
    Edit {
        /// Ticket ID (e.g., HLA1)
        id: String,

        /// Set the title
        #[arg(short, long)]
        title: Option<String>,

        /// Set the description
        #[arg(short, long)]
        description: Option<String>,

        /// Set the start date (RFC 3339 format, e.g., 2024-02-09T10:00:00Z or 2024-02-09)
        #[arg(long)]
        start_date: Option<String>,

        /// Set the end date (RFC 3339 format, e.g., 2024-02-15T17:00:00Z or 2024-02-15)
        #[arg(long)]
        end_date: Option<String>,

        /// Clear the start date
        #[arg(long)]
        clear_start_date: bool,

        /// Clear the end date
        #[arg(long)]
        clear_end_date: bool,

        /// Add acceptance criteria
        #[arg(long = "ac")]
        add_ac: Option<String>,

        /// Remove acceptance criteria by description or index
        #[arg(long = "remove-ac")]
        remove_ac: Option<String>,

        /// Mark acceptance criteria as complete by ID
        #[arg(long = "complete-ac")]
        complete_ac: Option<usize>,

        /// Mark acceptance criteria as incomplete by ID
        #[arg(long = "incomplete-ac")]
        incomplete_ac: Option<usize>,

        /// Toggle acceptance criteria completion status by ID
        #[arg(long = "toggle-ac")]
        toggle_ac: Option<usize>,
    },

    /// View ticket details
    Show {
        /// Ticket ID (e.g., HLA1)
        id: String,
    },

    /// Search tickets by title, description, or acceptance criteria
    Search {
        /// Search query
        query: String,

        /// Sort tickets by field (id, title, status, created, updated, start, end, ac-progress, ac-count)
        #[arg(long, default_value = "id")]
        sort_by: String,

        /// Sort order (asc or desc)
        #[arg(long, default_value = "asc")]
        sort_order: String,
    },

    /// Delete a ticket
    Delete {
        /// Ticket ID (e.g., HLA1)
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
        anyhow::bail!("Project not initialized. Run 'hlavi init' first.");
    }

    match cmd {
        TicketsCommand::List {
            sort_by,
            sort_order,
        } => list_tickets(&storage, sort_by, sort_order).await,
        TicketsCommand::Create { title } => create_ticket(&storage, title).await,
        TicketsCommand::Edit {
            id,
            title,
            description,
            start_date,
            end_date,
            clear_start_date,
            clear_end_date,
            add_ac,
            remove_ac,
            complete_ac,
            incomplete_ac,
            toggle_ac,
        } => {
            edit_ticket(
                &storage,
                EditTicketOptions {
                    id,
                    title,
                    description,
                    start_date,
                    end_date,
                    clear_start_date,
                    clear_end_date,
                    add_ac,
                    remove_ac,
                    complete_ac,
                    incomplete_ac,
                    toggle_ac,
                },
            )
            .await
        }
        TicketsCommand::Show { id } => show_ticket(&storage, id).await,
        TicketsCommand::Search {
            query,
            sort_by,
            sort_order,
        } => search_tickets(&storage, query, sort_by, sort_order).await,
        TicketsCommand::Delete { id, force } => delete_ticket(&storage, id, force).await,
    }
}

async fn list_tickets(
    storage: &impl Storage,
    sort_by: String,
    sort_order: String,
) -> anyhow::Result<()> {
    let ticket_ids = storage.list_ticket_ids().await?;

    if ticket_ids.is_empty() {
        println!("{}", "No tickets found.".yellow());
        println!("\nCreate a ticket with:");
        println!(
            "  {} hlavi tickets create \"Your ticket title\"",
            "$".yellow()
        );
        return Ok(());
    }

    // Load all tickets
    let mut tickets = Vec::new();
    for id in ticket_ids {
        let ticket = storage.load_ticket(&id).await?;
        tickets.push(ticket);
    }

    // Parse and apply sorting
    let field = sort_by
        .parse::<crate::commands::sort::SortField>()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let order = sort_order
        .parse::<crate::commands::sort::SortOrder>()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    crate::commands::sort::sort_tickets(&mut tickets, field, order);

    // Build rows from sorted tickets
    let mut rows = Vec::new();
    for ticket in tickets {
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

struct EditTicketOptions {
    id: String,
    title: Option<String>,
    description: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    clear_start_date: bool,
    clear_end_date: bool,
    add_ac: Option<String>,
    remove_ac: Option<String>,
    complete_ac: Option<usize>,
    incomplete_ac: Option<usize>,
    toggle_ac: Option<usize>,
}

async fn edit_ticket(storage: &impl Storage, options: EditTicketOptions) -> anyhow::Result<()> {
    let EditTicketOptions {
        id,
        title,
        description,
        start_date,
        end_date,
        clear_start_date,
        clear_end_date,
        add_ac,
        remove_ac,
        complete_ac,
        incomplete_ac,
        toggle_ac,
    } = options;
    let ticket_id = TicketId::from_str(&id)?;
    let mut ticket = storage.load_ticket(&ticket_id).await?;

    let mut modified = false;

    if let Some(new_title) = title {
        ticket.set_title(new_title.clone());
        modified = true;
        println!(
            "{} Updated title to {}",
            "✓".green().bold(),
            new_title.cyan()
        );
    }

    if let Some(desc) = description {
        ticket.set_description(desc);
        modified = true;
        println!("{} Updated description", "✓".green().bold());
    }

    if let Some(date_str) = start_date {
        let date = parse_date(&date_str)?;
        ticket.set_start_date(date)?;
        modified = true;
        println!(
            "{} Set start date to {}",
            "✓".green().bold(),
            date.format("%Y-%m-%d").to_string().cyan()
        );
    }

    if let Some(date_str) = end_date {
        let date = parse_date(&date_str)?;
        ticket.set_end_date(date)?;
        modified = true;
        println!(
            "{} Set end date to {}",
            "✓".green().bold(),
            date.format("%Y-%m-%d").to_string().cyan()
        );
    }

    if clear_start_date {
        ticket.clear_start_date();
        modified = true;
        println!("{} Cleared start date", "✓".green().bold());
    }

    if clear_end_date {
        ticket.clear_end_date();
        modified = true;
        println!("{} Cleared end date", "✓".green().bold());
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

    if let Some(ac_id) = complete_ac {
        // Find the acceptance criteria by ID
        if let Some(ac) = ticket
            .acceptance_criteria
            .iter_mut()
            .find(|ac| ac.id == ac_id)
        {
            ac.mark_completed();
            modified = true;
            println!(
                "{} Marked acceptance criteria {} as {}",
                "✓".green().bold(),
                ac_id,
                "completed".green()
            );
        } else {
            anyhow::bail!("Acceptance criteria with ID {} not found", ac_id);
        }
    }

    if let Some(ac_id) = incomplete_ac {
        // Find the acceptance criteria by ID
        if let Some(ac) = ticket
            .acceptance_criteria
            .iter_mut()
            .find(|ac| ac.id == ac_id)
        {
            ac.mark_incomplete();
            modified = true;
            println!(
                "{} Marked acceptance criteria {} as {}",
                "✓".green().bold(),
                ac_id,
                "incomplete".yellow()
            );
        } else {
            anyhow::bail!("Acceptance criteria with ID {} not found", ac_id);
        }
    }

    if let Some(ac_id) = toggle_ac {
        // Find the acceptance criteria by ID
        if let Some(ac) = ticket
            .acceptance_criteria
            .iter_mut()
            .find(|ac| ac.id == ac_id)
        {
            let new_status = !ac.completed;
            ac.toggle();
            modified = true;
            let status_text = if new_status {
                "completed".green()
            } else {
                "incomplete".yellow()
            };
            println!(
                "{} Toggled acceptance criteria {} to {}",
                "✓".green().bold(),
                ac_id,
                status_text
            );
        } else {
            anyhow::bail!("Acceptance criteria with ID {} not found", ac_id);
        }
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
    println!(
        "  Created: {}",
        ticket.created_at.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "  Updated: {}",
        ticket.updated_at.format("%Y-%m-%d %H:%M:%S")
    );

    if let Some(start) = ticket.start_date {
        println!(
            "  Start Date: {}",
            start.format("%Y-%m-%d").to_string().cyan()
        );
    }

    if let Some(end) = ticket.end_date {
        println!("  End Date: {}", end.format("%Y-%m-%d").to_string().cyan());
    }

    if let Some(reason) = &ticket.rejection_reason {
        println!("\n{}: {}", "Rejection Reason".red().bold(), reason);
    }

    println!();

    Ok(())
}

async fn search_tickets(
    storage: &impl Storage,
    query: String,
    sort_by: String,
    sort_order: String,
) -> anyhow::Result<()> {
    let mut matching_tickets = storage.search_tickets(&query).await?;

    if matching_tickets.is_empty() {
        println!(
            "{} No tickets found matching \"{}\"",
            "✗".red().bold(),
            query.yellow()
        );
        return Ok(());
    }

    // Parse and apply sorting
    let field = sort_by
        .parse::<crate::commands::sort::SortField>()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let order = sort_order
        .parse::<crate::commands::sort::SortOrder>()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    crate::commands::sort::sort_tickets(&mut matching_tickets, field, order);

    println!(
        "\n{} {} ticket(s) matching \"{}\"\n",
        "✓".green().bold(),
        matching_tickets.len(),
        query.yellow()
    );

    let mut rows = Vec::new();
    for ticket in matching_tickets {
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

/// Parse a date string in RFC 3339 format or simple YYYY-MM-DD format
fn parse_date(date_str: &str) -> anyhow::Result<DateTime<Utc>> {
    // Try parsing as RFC 3339 first
    if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
        return Ok(dt.with_timezone(&Utc));
    }

    // Try parsing as simple date (YYYY-MM-DD) and set time to midnight UTC
    if let Ok(naive_date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        return Ok(Utc.from_utc_datetime(&naive_date.and_hms_opt(0, 0, 0).unwrap()));
    }

    anyhow::bail!(
        "Invalid date format: '{}'. Use RFC 3339 format (e.g., 2024-02-09T10:00:00Z) or YYYY-MM-DD (e.g., 2024-02-09)",
        date_str
    )
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
