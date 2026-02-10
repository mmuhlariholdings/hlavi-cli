use crate::commands::get_storage;
use chrono::{DateTime, Utc};
use colored::*;
use hlavi_core::storage::Storage;

pub async fn execute(sort_by: Option<String>, sort_order: Option<String>) -> anyhow::Result<()> {
    let storage = get_storage()?;

    if !storage.is_initialized().await {
        anyhow::bail!("Project not initialized. Run 'hlavi init' first.");
    }

    show_timeline(&storage, sort_by, sort_order).await
}

async fn show_timeline(
    storage: &impl Storage,
    sort_by: Option<String>,
    sort_order: Option<String>,
) -> anyhow::Result<()> {
    let task_ids = storage.list_task_ids().await?;

    if task_ids.is_empty() {
        println!("{}", "No tasks found.".yellow());
        return Ok(());
    }

    // Load all tickets and filter those with dates
    let mut tasks_with_dates = Vec::new();
    let mut tasks_without_dates = Vec::new();

    for id in task_ids {
        let task = storage.load_task(&id).await?;
        if task.start_date.is_some() || task.end_date.is_some() {
            tasks_with_dates.push(task);
        } else {
            tasks_without_dates.push(task);
        }
    }

    if tasks_with_dates.is_empty() {
        println!("{}", "No tasks with dates found.".yellow());
        println!("\nAdd dates to tasks with:");
        println!(
            "  {} hlavi tasks edit <ID> --start-date YYYY-MM-DD --end-date YYYY-MM-DD",
            "$".yellow()
        );
        return Ok(());
    }

    // Find the overall date range
    let mut min_date: Option<DateTime<Utc>> = None;
    let mut max_date: Option<DateTime<Utc>> = None;

    for task in &tasks_with_dates {
        if let Some(start) = task.start_date {
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
        if let Some(end) = task.end_date {
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
        for task in &tasks_with_dates {
            if let Some(end) = task.end_date {
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
        for task in &tasks_with_dates {
            if let Some(start) = task.start_date {
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

    // Apply sorting
    if let Some(sort_by_field) = sort_by {
        // Custom sorting specified by user
        let field = sort_by_field
            .parse::<crate::commands::sort::SortField>()
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let order = sort_order
            .unwrap_or_else(|| "asc".to_string())
            .parse::<crate::commands::sort::SortOrder>()
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        crate::commands::sort::sort_tasks(&mut tasks_with_dates, field, order);
    } else {
        // Default: sort by start date
        tasks_with_dates.sort_by(|a, b| {
            let a_start = a.start_date.unwrap_or(a.end_date.unwrap_or(timeline_start));
            let b_start = b.start_date.unwrap_or(b.end_date.unwrap_or(timeline_start));
            a_start.cmp(&b_start)
        });
    }

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
    for task in &tasks_with_dates {
        let task_start = task.start_date.unwrap_or(timeline_start);
        let task_end = task.end_date.unwrap_or(task_start);

        // Calculate position and length
        let days_from_start = (task_start - timeline_start).num_days();
        let task_duration = (task_end - task_start).num_days().max(1);

        let bar_start =
            ((days_from_start as f64 / total_days as f64) * timeline_width as f64) as usize;
        let bar_length =
            ((task_duration as f64 / total_days as f64) * timeline_width as f64).max(1.0) as usize;

        // Ensure bar fits within timeline
        let bar_start = bar_start.min(timeline_width - 1);
        let bar_length = bar_length.min(timeline_width - bar_start);

        // Print ticket ID
        print!("{:8} ", task.id.to_string().cyan().bold());

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
        let title = if task.title.len() > max_title_len {
            format!("{}...", &task.title[..max_title_len - 3])
        } else {
            task.title.clone()
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
    if !tasks_without_dates.is_empty() {
        println!();
        println!("{}", "Tasks without dates:".yellow());
        for task in &tasks_without_dates {
            println!("  {} - {}", task.id.to_string().cyan(), task.title);
        }
    }

    println!();

    Ok(())
}
