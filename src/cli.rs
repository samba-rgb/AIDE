use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new aide
    Create { 
        #[arg(value_name = "NAME")]
        name: String,
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

    /// Set a configuration value
    Set {
        #[arg(value_name = "KEY")]
        key: String,

        #[arg(value_name = "VALUE")]    
        value: String,
    },

    /// Get a configuration value
    Get {
        #[arg(value_name = "KEY")]
        key: String,
    },

    /// List all configuration keys and values
    ConfigList,

    /// Delete a configuration key
    ConfigDelete {
        #[arg(value_name = "KEY")]
        key: String,
    },

    /// Search for data by input text
    Search {
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

    /// Ask a question to the LLM
    Ask {
        #[arg(value_name = "QUESTION")]
        question: String,
    },

    /// Generate shell completion script
    Completions {
        #[arg(value_name = "SHELL")]
        shell: String,
    },
}