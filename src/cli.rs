use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new aide type
    Create { 
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(value_name = "TYPE")]
        aide_type: String,
    },
    /// Add data to an aide
    Add {
        #[arg(value_name = "NAME")]
        name: String,
        #[arg(value_name = "DATA")]
        data: Option<String>,
        /// Read content from file path instead of using data argument
        #[arg(short = 'p', long = "path")]
        path: Option<String>,
    },
    /// Search for data by input text
    Search {
        #[arg(value_name = "INPUT_TEXT")]
        input_text: String,
    },
    /// Search for data by command and input text
    Command {
        #[arg(value_name = "INPUT_TEXT")]
        input_text: String,
    },
    /// Create or edit a task
    Task {
        #[arg(value_name = "TASK_NAME")]
        task_name: String,
    },
    /// Change task status
    TaskStatus {
        #[arg(value_name = "TASK_NAME")]
        task_name: String,
        #[arg(value_name = "STATUS")]
        status: String,
    },
    /// Change task priority
    TaskPriority {
        #[arg(value_name = "TASK_NAME")]
        task_name: String,
        #[arg(value_name = "PRIORITY")]
        priority: u8,
    },
    /// List all tasks
    TaskList,
    /// Edit task log file
    TaskEdit {
        #[arg(value_name = "TASK_NAME")]
        task_name: String,
    },
    /// Add log entry to task
    TaskLogUpdate {
        #[arg(value_name = "TASK_NAME")]
        task_name: String,
        #[arg(value_name = "LOG_TEXT")]
        log_text: String,
    },
    /// List all aides
    AideList,
    /// Clear all data from database and TF-IDF indexes
    Clear,
    /// Reset all data (WARNING: Deletes all tasks and aides)
    Reset,
    /// Open aide file in vim editor
    Write {
        #[arg(value_name = "AIDE_NAME")]
        aide_name: String,
    },
    /// Launch TUI interface
    Tui,
}