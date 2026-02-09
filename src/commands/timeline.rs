use crate::commands::get_storage;
use chrono::{DateTime, Utc};
use colored::*;
use hlavi_core::storage::Storage;

pub async fn execute() -> anyhow::Result<()> {
    let storage = get_storage()?;

    if !storage.is_initialized().await {
        anyhow::bail!("Project not initialized. Run 'hlavi init' first.");
    }

    show_timeline(&storage).await
}

async fn show_timeline(storage: &impl Storage) -> anyhow::Result<()> {
    let ticket_ids = storage.list_ticket_ids().await?;

    if ticket_ids.is_empty() {
        println!("{}", "No tickets found.".yellow());
        return Ok(());
    }

    // Load all tickets and filter those with dates
    let mut tickets_with_dates = Vec::new();
    let mut tickets_without_dates = Vec::new();

    for id in ticket_ids {
        let ticket = storage.load_ticket(&id).await?;
        if ticket.start_date.is_some() || ticket.end_date.is_some() {
            tickets_with_dates.push(ticket);
        } else {
            tickets_without_dates.push(ticket);
        }
    }

    if tickets_with_dates.is_empty() {
        println!("{}", "No tickets with dates found.".yellow());
        println!("\nAdd dates to tickets with:");
        println!(
            "  {} hlavi tickets edit <ID> --start-date YYYY-MM-DD --end-date YYYY-MM-DD",
            "$".yellow()
        );
        return Ok(());
    }

    // Find the overall date range
    let mut min_date: Option<DateTime<Utc>> = None;
    let mut max_date: Option<DateTime<Utc>> = None;

    for ticket in &tickets_with_dates {
        if let Some(start) = ticket.start_date {
            min_date = Some(match min_date {
                None => start,
                Some(current) => {
                    if start < current {
                        start
                    } else {
                        current
                    }
                }
            });
        }
        if let Some(end) = ticket.end_date {
            max_date = Some(match max_date {
                None => end,
                Some(current) => {
                    if end > current {
                        end
                    } else {
                        current
                    }
                }
            });
        }
    }

    // Use start dates as fallback for max, and end dates as fallback for min
    if min_date.is_none() {
        for ticket in &tickets_with_dates {
            if let Some(end) = ticket.end_date {
                min_date = Some(match min_date {
                    None => end,
                    Some(current) => {
                        if end < current {
                            end
                        } else {
                            current
                        }
                    }
                });
            }
        }
    }

    if max_date.is_none() {
        for ticket in &tickets_with_dates {
            if let Some(start) = ticket.start_date {
                max_date = Some(match max_date {
                    None => start,
                    Some(current) => {
                        if start > current {
                            start
                        } else {
                            current
                        }
                    }
                });
            }
        }
    }

    let timeline_start = min_date.unwrap();
    let timeline_end = max_date.unwrap();
    let total_days = (timeline_end - timeline_start).num_days().max(1);

    // Timeline width in characters
    let timeline_width = 60;

    // Sort tickets by start date
    tickets_with_dates.sort_by(|a, b| {
        let a_start = a.start_date.unwrap_or(a.end_date.unwrap_or(timeline_start));
        let b_start = b.start_date.unwrap_or(b.end_date.unwrap_or(timeline_start));
        a_start.cmp(&b_start)
    });

    // Print header
    println!("\n{}", "Timeline View".cyan().bold());
    println!("{}", "─".repeat(80));
    println!(
        "Range: {} to {}",
        timeline_start.format("%Y-%m-%d").to_string().cyan(),
        timeline_end.format("%Y-%m-%d").to_string().cyan()
    );
    println!();

    // Print scale
    print!("{:12} ", "");
    for i in 0..=10 {
        let pos = (i as f64 / 10.0) * total_days as f64;
        let date = timeline_start + chrono::Duration::days(pos as i64);
        if i == 0 {
            print!("{:<6}", date.format("%m/%d"));
        } else if i == 10 {
            print!("{:>6}", date.format("%m/%d"));
        } else {
            print!("     ");
        }
    }
    println!();

    print!("{:12} ", "");
    println!("{}", "─".repeat(timeline_width));

    // Print tickets
    for ticket in &tickets_with_dates {
        let ticket_start = ticket.start_date.unwrap_or(timeline_start);
        let ticket_end = ticket.end_date.unwrap_or(ticket_start);

        // Calculate position and length
        let days_from_start = (ticket_start - timeline_start).num_days();
        let ticket_duration = (ticket_end - ticket_start).num_days().max(1);

        let bar_start =
            ((days_from_start as f64 / total_days as f64) * timeline_width as f64) as usize;
        let bar_length = ((ticket_duration as f64 / total_days as f64) * timeline_width as f64)
            .max(1.0) as usize;

        // Ensure bar fits within timeline
        let bar_start = bar_start.min(timeline_width - 1);
        let bar_length = bar_length.min(timeline_width - bar_start);

        // Print ticket ID
        print!("{:8} ", ticket.id.to_string().cyan().bold());

        // Print timeline bar
        for i in 0..timeline_width {
            if i >= bar_start && i < bar_start + bar_length {
                if i == bar_start {
                    print!("{}", "┣".green());
                } else if i == bar_start + bar_length - 1 {
                    print!("{}", "┫".green());
                } else {
                    print!("{}", "━".green());
                }
            } else {
                print!(" ");
            }
        }

        // Print title (truncated)
        let max_title_len = 30;
        let title = if ticket.title.len() > max_title_len {
            format!("{}...", &ticket.title[..max_title_len - 3])
        } else {
            ticket.title.clone()
        };
        println!(" {}", title);
    }

    print!("{:12} ", "");
    println!("{}", "─".repeat(timeline_width));

    // Print legend
    println!();
    println!("Legend:");
    println!("  {} Start date", "┣".green());
    println!("  {} Duration", "━".green());
    println!("  {} End date", "┫".green());

    // Show tickets without dates
    if !tickets_without_dates.is_empty() {
        println!();
        println!("{}", "Tickets without dates:".yellow());
        for ticket in &tickets_without_dates {
            println!("  {} - {}", ticket.id.to_string().cyan(), ticket.title);
        }
    }

    println!();

    Ok(())
}
