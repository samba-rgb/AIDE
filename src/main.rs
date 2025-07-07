mod cli;
mod models;
mod database;
mod ui;
mod editor;
mod tfidf;

use anyhow::Result;
use cli::{Cli, Commands};
use database::Database;
use ui::run_tui;
use clap::Parser;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut db = Database::new()?;
    
    match cli.command {
        Some(Commands::Create { name, aide_type }) => {
            db.create_aide(&name, &aide_type)?;
        }
        Some(Commands::Add { name, data, path }) => {
            // Validate that either data or path is provided
            match (data.as_deref(), path.as_deref()) {
                (Some(content), None) => {
                    // Use provided data
                    db.add_data(&name, content, None)?;
                }
                (None, Some(file_path)) => {
                    // Use file path
                    db.add_data(&name, "", Some(file_path))?;
                }
                (Some(_), Some(_)) => {
                    println!("Error: Cannot specify both data and path. Use either content or -p flag.");
                    return Ok(());
                }
                (None, None) => {
                    println!("Error: Must provide either content data or -p flag with file path.");
                    return Ok(());
                }
            }
        }
        Some(Commands::Search { input_text }) => {
            db.search_by_input(&input_text)?;
        }
        Some(Commands::Command { input_text }) => {
            db.search_by_command(&input_text)?;
        }
        Some(Commands::Task { task_name }) => {
            db.create_task(&task_name)?;
        }
        Some(Commands::TaskStatus { task_name, status }) => {
            db.update_task_status(&task_name, &status)?;
        }
        Some(Commands::TaskPriority { task_name, priority }) => {
            db.update_task_priority(&task_name, priority)?;
        }
        Some(Commands::TaskList) => {
            db.list_tasks()?;
        }
        Some(Commands::TaskEdit { task_name }) => {
            db.edit_task(&task_name)?;
        }
        Some(Commands::TaskLogUpdate { task_name, log_text }) => {
            db.add_task_log(&task_name, &log_text)?;
        }
        Some(Commands::AideList) => {
            db.list_aides()?;
        }
        Some(Commands::Clear) => {
            db.clear_all_data()?;
        }
        Some(Commands::Reset) => {
            db.clear_all_data()?;
        }
        Some(Commands::Write { aide_name }) => {
            db.write_aide(&aide_name)?;
        }
        Some(Commands::Tui) => {
            run_tui(db)?;
        }
        None => {
            // Default behavior: launch TUI
            run_tui(db)?;
        }
    }
    
    Ok(())
}
