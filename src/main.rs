mod cli;
mod models;
mod database;
mod ui;
mod editor;
mod tfidf;
mod llm;

use anyhow::Result;
use clap::{CommandFactory, Parser};
use cli::{Cli, Commands};
use database::Database;
use ui::run_tui;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut db = Database::new()?;
    
    match cli.command {
        Some(Commands::Create { name }) => {
            db.create_aide(&name)?;
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

        Some(Commands::Set { key, value }) => {
            db.set_config(&key, &value)?;
        }

        Some(Commands::Get { key }) => {
            db.get_config(&key)?;
        }

        Some(Commands::ConfigList) => {
            db.list_configs()?;
        }

        Some(Commands::ConfigDelete { key }) => {
            db.delete_config(&key)?;
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
        Some(Commands::Ask { question }) => {
            // Call LLM and print answer
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                let base_url = std::env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
                let model_name = std::env::var("OLLAMA_MODEL_NAME").unwrap_or_else(|_| "qwen2.5-coder:0.5b".to_string());
                let mut processor = llm::command_processor::CommandProcessor::new(base_url, model_name).await?;
                let answer = processor.process_query(&question).await?;
                println!("{}", answer);
                Ok::<(), anyhow::Error>(())
            })?;
        }
        Some(Commands::Completions { shell }) => {
            use clap_complete::{generate, Shell};
            let shell = shell.to_lowercase();
            let shell_enum = match shell.as_str() {
                "bash" => Shell::Bash,
                "zsh" => Shell::Zsh,
                "fish" => Shell::Fish,
                "elvish" => Shell::Elvish,
                "powershell" => Shell::PowerShell,
                _ => {
                    println!("Unsupported shell: {}", shell);
                    return Ok(());
                }
            };
            let mut cmd = Cli::command();
            generate(shell_enum, &mut cmd, "aide", &mut std::io::stdout());
        }
        None => {
            // Default behavior: launch TUI
            run_tui(db)?;
        }
    }
    
    Ok(())
}
