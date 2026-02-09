use crate::commands::get_storage;
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use clap::Subcommand;
use colored::*;
use hlavi_core::{
    domain::task::{Task, TaskId},
    storage::Storage,
};
use std::str::FromStr;
use tabled::{
    settings::{object::Columns, Alignment, Style},
    Table, Tabled,
};

#[derive(Subcommand)]
pub enum TasksCommand {
    /// List all tasks
    List {
        /// Sort tasks by field (id, title, status, created, updated, start, end, ac-progress, ac-count)
        #[arg(long, default_value = "id")]
        sort_by: String,

        /// Sort order (asc or desc)
        #[arg(long, default_value = "asc")]
        sort_order: String,
    },

    /// Create a new task
    Create {
        /// Title of the task
        title: String,
    },

    /// Edit a task
    Edit {
        /// Task ID (e.g., HLA1)
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

    /// View task details
    Show {
        /// Task ID (e.g., HLA1)
        id: String,
    },

    /// Search tasks by title, description, or acceptance criteria
    Search {
        /// Search query
        query: String,

        /// Sort tasks by field (id, title, status, created, updated, start, end, ac-progress, ac-count)
        #[arg(long, default_value = "id")]
        sort_by: String,

        /// Sort order (asc or desc)
        #[arg(long, default_value = "asc")]
        sort_order: String,
    },

    /// Delete a ticket
    Delete {
        /// Task ID (e.g., HLA1)
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

pub async fn execute(cmd: TasksCommand) -> anyhow::Result<()> {
    let storage = get_storage()?;

    if !storage.is_initialized().await {
        anyhow::bail!("Project not initialized. Run 'hlavi init' first.");
    }

    match cmd {
        TasksCommand::List {
            sort_by,
            sort_order,
        } => list_tasks(&storage, sort_by, sort_order).await,
        TasksCommand::Create { title } => create_task(&storage, title).await,
        TasksCommand::Edit {
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
        TasksCommand::Show { id } => show_ticket(&storage, id).await,
        TasksCommand::Search {
            query,
            sort_by,
            sort_order,
        } => search_tasks(&storage, query, sort_by, sort_order).await,
        TasksCommand::Delete { id, force } => delete_task(&storage, id, force).await,
    }
}

async fn list_tasks(
    storage: &impl Storage,
    sort_by: String,
    sort_order: String,
) -> anyhow::Result<()> {
    let ticket_ids = storage.list_task_ids().await?;

    if ticket_ids.is_empty() {
        println!("{}", "No tasks found.".yellow());
        println!("\nCreate a ticket with:");
        println!(
            "  {} hlavi tasks create \"Your ticket title\"",
            "$".yellow()
        );
        return Ok(());
    }

    // Load all tasks
    let mut tasks = Vec::new();
    for id in ticket_ids {
        let task = storage.load_task(&id).await?;
        tasks.push(task);
    }

    // Parse and apply sorting
    let field = sort_by
        .parse::<crate::commands::sort::SortField>()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let order = sort_order
        .parse::<crate::commands::sort::SortOrder>()
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    crate::commands::sort::sort_tasks(&mut tasks, field, order);

    // Build rows from sorted tasks
    let mut rows = Vec::new();
    for task in tasks {
        rows.push(TicketRow {
            id: task.id.to_string(),
            title: task.title.clone(),
            status: task.status.to_string(),
            acceptance_criteria_count: format!(
                "{}/{}",
                task
                    .acceptance_criteria
                    .iter()
                    .filter(|ac| ac.completed)
                    .count(),
                task.acceptance_criteria.len()
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

async fn create_task(storage: &impl Storage, title: String) -> anyhow::Result<()> {
    let mut board = storage.load_board().await?;
    let task_id = board.next_task_id();

    let task = Task::new(task_id.clone(), title);

    storage.save_task(&task).await?;
    board.add_task(task_id.clone());
    storage.save_board(&board).await?;

    println!(
        "{} Created task {}",
        "✓".green().bold(),
        task_id.to_string().cyan().bold()
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
    let ticket_id = TaskId::from_str(&id)?;
    let mut task = storage.load_task(&ticket_id).await?;

    let mut modified = false;

    if let Some(new_title) = title {
        task.set_title(new_title.clone());
        modified = true;
        println!(
            "{} Updated title to {}",
            "✓".green().bold(),
            new_title.cyan()
        );
    }

    if let Some(desc) = description {
        task.set_description(desc);
        modified = true;
        println!("{} Updated description", "✓".green().bold());
    }

    if let Some(date_str) = start_date {
        let date = parse_date(&date_str)?;
        task.set_start_date(date)?;
        modified = true;
        println!(
            "{} Set start date to {}",
            "✓".green().bold(),
            date.format("%Y-%m-%d").to_string().cyan()
        );
    }

    if let Some(date_str) = end_date {
        let date = parse_date(&date_str)?;
        task.set_end_date(date)?;
        modified = true;
        println!(
            "{} Set end date to {}",
            "✓".green().bold(),
            date.format("%Y-%m-%d").to_string().cyan()
        );
    }

    if clear_start_date {
        task.clear_start_date();
        modified = true;
        println!("{} Cleared start date", "✓".green().bold());
    }

    if clear_end_date {
        task.clear_end_date();
        modified = true;
        println!("{} Cleared end date", "✓".green().bold());
    }

    if let Some(ac) = add_ac {
        task.add_acceptance_criterion(ac.clone());
        modified = true;
        println!(
            "{} Added acceptance criteria: {}",
            "✓".green().bold(),
            ac.cyan()
        );
    }

    if let Some(ac_identifier) = remove_ac {
        task.remove_acceptance_criterion(&ac_identifier)?;
        modified = true;
        println!("{} Removed acceptance criteria", "✓".green().bold());
    }

    if let Some(ac_id) = complete_ac {
        // Find the acceptance criteria by ID
        if let Some(ac) = task
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
        if let Some(ac) = task
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
        if let Some(ac) = task
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

    storage.save_task(&task).await?;

    println!(
        "{} Updated ticket {}",
        "✓".green().bold(),
        ticket_id.to_string().cyan().bold()
    );

    Ok(())
}

async fn show_ticket(storage: &impl Storage, id: String) -> anyhow::Result<()> {
    let ticket_id = TaskId::from_str(&id)?;
    let task = storage.load_task(&ticket_id).await?;

    println!("\n{}", format!("Ticket {}", task.id).cyan().bold());
    println!("{}", "─".repeat(50));
    println!("{}: {}", "Title".bold(), task.title);
    println!("{}: {}", "Status".bold(), task.status);

    if let Some(desc) = &task.description {
        println!("\n{}:", "Description".bold());
        println!("{}", desc);
    }

    if !task.acceptance_criteria.is_empty() {
        println!("\n{}:", "Acceptance Criteria".bold());
        for ac in &task.acceptance_criteria {
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
        task.created_at.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "  Updated: {}",
        task.updated_at.format("%Y-%m-%d %H:%M:%S")
    );

    if let Some(start) = task.start_date {
        println!(
            "  Start Date: {}",
            start.format("%Y-%m-%d").to_string().cyan()
        );
    }

    if let Some(end) = task.end_date {
        println!("  End Date: {}", end.format("%Y-%m-%d").to_string().cyan());
    }

    if let Some(reason) = &task.rejection_reason {
        println!("\n{}: {}", "Rejection Reason".red().bold(), reason);
    }

    println!();

    Ok(())
}

async fn search_tasks(
    storage: &impl Storage,
    query: String,
    sort_by: String,
    sort_order: String,
) -> anyhow::Result<()> {
    let mut matching_tasks = storage.search_tasks(&query).await?;

    if matching_tasks.is_empty() {
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

    crate::commands::sort::sort_tasks(&mut matching_tasks, field, order);

    println!(
        "\n{} {} ticket(s) matching \"{}\"\n",
        "✓".green().bold(),
        matching_tasks.len(),
        query.yellow()
    );

    let mut rows = Vec::new();
    for task in matching_tasks {
        rows.push(TicketRow {
            id: task.id.to_string(),
            title: task.title.clone(),
            status: task.status.to_string(),
            acceptance_criteria_count: format!(
                "{}/{}",
                task
                    .acceptance_criteria
                    .iter()
                    .filter(|ac| ac.completed)
                    .count(),
                task.acceptance_criteria.len()
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

async fn delete_task(storage: &impl Storage, id: String, force: bool) -> anyhow::Result<()> {
    let ticket_id = TaskId::from_str(&id)?;

    // Verify ticket exists
    let _ticket = storage.load_task(&ticket_id).await?;

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

    storage.delete_task(&ticket_id).await?;

    println!(
        "{} Deleted ticket {}",
        "✓".green().bold(),
        ticket_id.to_string().cyan().bold()
    );

    Ok(())
}
