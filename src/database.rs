use anyhow::Result;
use rusqlite::{Connection, OptionalExtension};
use std::path::PathBuf;
use std::process::Command;
use std::fs;
use std::io::{self, Write};
use fuzzy_matcher::FuzzyMatcher;
use crate::models::{TaskItem, AideItem, ConfigItem};
use crate::tfidf::{TfIdfIndex, FuzzyMatchResult, build_tfidf_index, find_fuzzy_match_in_index, FUZZY_MATCH_THRESHOLD};

// Helper function to ask user for confirmation
fn ask_user_confirmation(input_name: &str, suggested_name: &str) -> bool {
    print!("'{}' not found. Did you mean '{}'? (y/n): ", input_name, suggested_name);
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes"
}

pub struct Database {
    conn: Connection,
    task_index: Option<TfIdfIndex>,
    aide_index: Option<TfIdfIndex>,
    config_index: Option<TfIdfIndex>,
}

impl Database {
    pub fn new() -> Result<Self> {
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let db_path = PathBuf::from(home_dir).join(".aide.db");
        
        let conn = Connection::open(db_path)?;
        
        // Create tables if they don't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS aides (
                id INTEGER PRIMARY KEY,
                name TEXT UNIQUE NOT NULL
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS data (
                id INTEGER PRIMARY KEY,
                aide_id INTEGER NOT NULL,
                input_text TEXT NOT NULL,
                command_output TEXT NOT NULL,
                FOREIGN KEY (aide_id) REFERENCES aides (id)
            )",
            [],
        )?;
        
        // Create tasks table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY,
                name TEXT UNIQUE NOT NULL,
                priority INTEGER NOT NULL DEFAULT 3,
                status TEXT NOT NULL DEFAULT 'created',
                task_log_file_path TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Create config_data table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS config_data (
                id INTEGER PRIMARY KEY,
                key_name TEXT UNIQUE NOT NULL,
                value TEXT NOT NULL,
                description TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        
        // Create default task_log aide if it doesn't exist
        let _ = conn.execute(
            "INSERT OR IGNORE INTO aides (name) VALUES ('task_log')",
            [],
        );
        
        let mut db = Database { 
            conn,
            task_index: None,
            aide_index: None,
            config_index: None,
        };
        
        // Build initial indexes
        db.rebuild_task_index()?;
        db.rebuild_aide_index()?;
        db.rebuild_config_index()?;
        
        Ok(db)
    }
    
    // Build TF-IDF index for tasks
    pub fn rebuild_task_index(&mut self) -> Result<()> {
        let mut stmt = self.conn.prepare("SELECT name FROM tasks")?;
        let rows = stmt.query_map([], |row| {
            Ok(row.get::<_, String>(0)?)
        })?;
        
        let mut task_names = Vec::new();
        for row in rows {
            task_names.push(row?);
        }
        
        self.task_index = Some(build_tfidf_index(task_names)?);
        Ok(())
    }
    
    // Build TF-IDF index for aides
    pub fn rebuild_aide_index(&mut self) -> Result<()> {
        let mut stmt = self.conn.prepare("SELECT name FROM aides")?;
        let rows = stmt.query_map([], |row| {
            Ok(row.get::<_, String>(0)?)
        })?;
        
        let mut aide_names = Vec::new();
        for row in rows {
            aide_names.push(row?);
        }
        
        self.aide_index = Some(build_tfidf_index(aide_names)?);
        Ok(())
    }
    
    // Build TF-IDF index for config keys
    pub fn rebuild_config_index(&mut self) -> Result<()> {
        let mut stmt = self.conn.prepare("SELECT key_name FROM config_data")?;
        let rows = stmt.query_map([], |row| {
            Ok(row.get::<_, String>(0)?)
        })?;
        
        let mut config_keys = Vec::new();
        for row in rows {
            config_keys.push(row?);
        }
        
        self.config_index = Some(build_tfidf_index(config_keys)?);
        Ok(())
    }
    
    // Find fuzzy matches for tasks using TF-IDF
    pub fn find_fuzzy_task_match(&self, input_name: &str) -> Result<FuzzyMatchResult> {
        if let Some(index) = &self.task_index {
            find_fuzzy_match_in_index(input_name, index)
        } else {
            Ok(FuzzyMatchResult {
                exact_match: false,
                suggested_name: None,
                score: None,
            })
        }
    }
    
    // Find fuzzy matches for aides using TF-IDF
    pub fn find_fuzzy_aide_match(&self, input_name: &str) -> Result<FuzzyMatchResult> {
        if let Some(index) = &self.aide_index {
            find_fuzzy_match_in_index(input_name, index)
        } else {
            Ok(FuzzyMatchResult {
                exact_match: false,
                suggested_name: None,
                score: None,
            })
        }
    }

    // Find fuzzy matches for config keys using TF-IDF
    pub fn find_fuzzy_config_match(&self, input_name: &str) -> Result<FuzzyMatchResult> {
        if let Some(index) = &self.config_index {
            find_fuzzy_match_in_index(input_name, index)
        } else {
            Ok(FuzzyMatchResult {
                exact_match: false,
                suggested_name: None,
                score: None,
            })
        }
    }
    
    pub fn create_aide(&mut self, name: &str) -> Result<()> {
        match self.conn.execute(
            "INSERT INTO aides (name) VALUES (?1)",
            [name],
        ) {
            Ok(_) => {
                // Create the file for this aide
                let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                let aide_dir = PathBuf::from(&home_dir).join(".aide");
                fs::create_dir_all(&aide_dir)?;
                
                let file_path = aide_dir.join(format!("{}.txt", name));
                if !file_path.exists() {
                    let initial_content = format!("# {}\n\nCreated: {}\n\n", 
                                                 name, 
                                                 chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"));
                    fs::write(&file_path, initial_content)?;
                }
                
                println!("Aide '{}' created successfully", name);
                // Use incremental update instead of full rebuild
                if let Some(ref mut index) = self.aide_index {
                    index.add_entity(name.to_string())?;
                }
                Ok(())
            }
            Err(rusqlite::Error::SqliteFailure(err, _)) 
                if err.code == rusqlite::ErrorCode::ConstraintViolation => {
                println!("Aide '{}' already exists", name);
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }
    
    pub fn add_data(&mut self, name: &str, data: &str, path: Option<&str>) -> Result<()> {
        // Use fuzzy matching to find the aide
        let fuzzy_result = self.find_fuzzy_aide_match(name)?;
        
        let actual_aide_name = match fuzzy_result {
            FuzzyMatchResult { exact_match: true, suggested_name: Some(name), .. } => name,
            FuzzyMatchResult { suggested_name: Some(suggestion), score: Some(score), .. } => {
                if score >= FUZZY_MATCH_THRESHOLD {
                    if ask_user_confirmation(name, &suggestion) {
                        suggestion
                    } else {
                        println!("Operation cancelled.");
                        return Ok(());
                    }
                } else {
                    println!("Aide '{}' not found.", name);
                    return Ok(());
                }
            }
            _ => {
                println!("Aide '{}' not found.", name);
                return Ok(());
            }
        };
        
        // Determine the actual content to add
        let content = if let Some(file_path) = path {
            // Read content from file
            match fs::read_to_string(file_path) {
                Ok(file_content) => {
                    println!("Reading content from file: {}", file_path);
                    file_content.trim().to_string() // Remove trailing whitespace/newlines
                }
                Err(e) => {
                    println!("Error reading file '{}': {}", file_path, e);
                    return Ok(());
                }
            }
        } else {
            // Use the provided data
            data.to_string()
        };
        
        // Find the aide by name
        let aide_id: i64 = match self.conn.query_row(
            "SELECT id FROM aides WHERE name = ?1",
            [&actual_aide_name],
            |row| Ok(row.get(0)?),
        ) {
            Ok(id) => id,
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                println!("Aide '{}' not found in database", actual_aide_name);
                return Ok(());
            }
            Err(e) => return Err(e.into()),
        };
        
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S");
        let timestamped_data = format!("[{}] {}", timestamp, content);
        
        // Create/append to file for this aide
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let aide_dir = PathBuf::from(&home_dir).join(".aide");
        fs::create_dir_all(&aide_dir)?;
        
        let file_path = aide_dir.join(format!("{}.txt", actual_aide_name));
        
        // Append to existing file or create new one with better formatting
        let existing_content = if file_path.exists() {
            fs::read_to_string(&file_path)?
        } else {
            format!("# {}\n\nCreated: {}\n\n", 
                   actual_aide_name, 
                   chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"))
        };
        
        // Use the new format: date time\n* input
        let new_entry = format!("{}\n* {}\n", timestamp, content);
        let updated_content = format!("{}{}", existing_content, new_entry);
        fs::write(&file_path, updated_content)?;
        println!("Data appended to file: {}", file_path.display());
        
        // Store in database
        self.conn.execute(
            "INSERT INTO data (aide_id, input_text, command_output) VALUES (?1, ?2, ?3)",
            [&aide_id.to_string(), &content, &timestamped_data],
        )?;
        
        if path.is_some() {
            println!("File content added successfully to aide '{}'", actual_aide_name);
        } else {
            println!("Data added successfully to aide '{}'", actual_aide_name);
        }
        Ok(())
    }
    
    pub fn search_by_input(&self, input_text: &str) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "SELECT d.input_text, d.command_output, a.name 
             FROM data d 
             JOIN aides a ON d.aide_id = a.id"
        )?;
        
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,  // input_text
                row.get::<_, String>(1)?,  // command_output
                row.get::<_, String>(2)?,  // name
            ))
        })?;
        
        let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
        let mut best_match: Option<(i64, String, String, String)> = None;
        
        for row in rows {
            let (db_input, output, name) = row?;
            if let Some(score) = matcher.fuzzy_match(&db_input, input_text) {
                if best_match.is_none() || score > best_match.as_ref().unwrap().0 {
                    best_match = Some((score, db_input, output, name));
                }
            }
        }
        
        match best_match {
            Some((_score, matched_input, output, name)) => {
                println!("Found match in aide '{}': {}", name, matched_input);
                println!("Output: {}", output);
            }
            None => {
                println!("No matches found for '{}'", input_text);
            }
        }
        
        Ok(())
    }
    
    pub fn search_by_command(&self, input_text: &str) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "SELECT d.input_text, d.command_output, a.name 
             FROM data d 
             JOIN aides a ON d.aide_id = a.id"
        )?;
        
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,  // input_text
                row.get::<_, String>(1)?,  // command_output
                row.get::<_, String>(2)?,  // name
            ))
        })?;
        
        let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
        let mut best_match: Option<(i64, String, String, String)> = None;
        
        for row in rows {
            let (db_input, output, name) = row?;
            let search_text = format!("{} {}", name, db_input);
            if let Some(score) = matcher.fuzzy_match(&search_text, input_text) {
                if best_match.is_none() || score > best_match.as_ref().unwrap().0 {
                    best_match = Some((score, db_input, output, name));
                }
            }
        }
        
        match best_match {
            Some((_score, matched_input, output, name)) => {
                println!("Found match in aide '{}': {}", name, matched_input);
                println!("Output: {}", output);
            }
            None => {
                println!("No matches found for '{}'", input_text);
            }
        }
        
        Ok(())
    }
    
    // Updated functions with TF-IDF fuzzy matching
    
    pub fn create_task(&mut self, task_name: &str) -> Result<()> {
        // Use fuzzy matching to check for similar tasks
        let fuzzy_result = self.find_fuzzy_task_match(task_name)?;
        
        let actual_task_name = match fuzzy_result {
            FuzzyMatchResult { exact_match: true, .. } => {
                println!("Task '{}' already exists. Opening task log file...", task_name);
                task_name.to_string()
            }
            FuzzyMatchResult { suggested_name: Some(suggestion), score: Some(score), .. } => {
                if score >= FUZZY_MATCH_THRESHOLD {
                    if ask_user_confirmation(task_name, &suggestion) {
                        println!("Opening existing task '{}'...", suggestion);
                        suggestion
                    } else {
                        // User wants to create new task with original name
                        task_name.to_string()
                    }
                } else {
                    task_name.to_string()
                }
            }
            _ => task_name.to_string()
        };
        
        // Create tasks directory if it doesn't exist
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let tasks_dir = PathBuf::from(&home_dir).join(".aide").join("tasks");
        fs::create_dir_all(&tasks_dir)?;
        
        // Create task log file path
        let task_log_file = tasks_dir.join(format!("{}.txt", actual_task_name));
        
        // Check if task already exists in database
        let exists = self.conn.query_row(
            "SELECT 1 FROM tasks WHERE name = ?1",
            [&actual_task_name],
            |_| Ok(()),
        );
        
        if exists.is_err() {
            // Create new task
            self.conn.execute(
                "INSERT INTO tasks (name, priority, status, task_log_file_path) VALUES (?1, 3, 'created', ?2)",
                [&actual_task_name, &task_log_file.to_string_lossy().to_string()],
            )?;
            
            // Create initial task log content
            let initial_content = format!(
                "Task: {}\nStatus: created\nPriority: 3\nCreated: {}\n\n--- Task Log ---\n",
                actual_task_name,
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
            );
            
            fs::write(&task_log_file, initial_content)?;
            println!("Task '{}' created successfully!", actual_task_name);
            
            // Use incremental update instead of full rebuild
            if let Some(ref mut index) = self.task_index {
                index.add_entity(actual_task_name.to_string())?;
            }
        }
        
        // Open the task log file in editor
        let status = Command::new("vi")
            .arg(&task_log_file)
            .status();
        
        match status {
            Ok(exit_status) => {
                if !exit_status.success() {
                    println!("Editor exited with status: {:?}", exit_status);
                }
            }
            Err(e) => {
                println!("Failed to open vi editor: {}", e);
                println!("Task log file is at: {}", task_log_file.display());
            }
        }
        
        Ok(())
    }
    
    pub fn update_task_status(&self, task_name: &str, status: &str) -> Result<()> {
        let valid_statuses = ["created", "in_progress", "completed"];
        if !valid_statuses.contains(&status) {
            println!("Invalid status. Valid statuses are: created, in_progress, completed");
            return Ok(());
        }
        
        // Use fuzzy matching to find the task
        let fuzzy_result = self.find_fuzzy_task_match(task_name)?;
        
        let actual_task_name = match fuzzy_result {
            FuzzyMatchResult { exact_match: true, suggested_name: Some(name), .. } => name,
            FuzzyMatchResult { suggested_name: Some(suggestion), score: Some(score), .. } => {
                if score >= FUZZY_MATCH_THRESHOLD {
                    if ask_user_confirmation(task_name, &suggestion) {
                        suggestion
                    } else {
                        println!("Operation cancelled.");
                        return Ok(());
                    }
                } else {
                    println!("Task '{}' not found.", task_name);
                    return Ok(());
                }
            }
            _ => {
                println!("Task '{}' not found.", task_name);
                return Ok(());
            }
        };
        
        let rows_affected = self.conn.execute(
            "UPDATE tasks SET status = ?1 WHERE name = ?2",
            [status, &actual_task_name],
        )?;
        
        if rows_affected == 0 {
            println!("Task '{}' not found in database", actual_task_name);
        } else {
            println!("Task '{}' status updated to '{}'", actual_task_name, status);
        }
        
        Ok(())
    }
    
    pub fn update_task_priority(&self, task_name: &str, priority: u8) -> Result<()> {
        if priority < 1 || priority > 5 {
            println!("Invalid priority. Priority must be between 1 (highest) and 5 (lowest)");
            return Ok(());
        }
        
        // Use fuzzy matching to find the task
        let fuzzy_result = self.find_fuzzy_task_match(task_name)?;
        
        let actual_task_name = match fuzzy_result {
            FuzzyMatchResult { exact_match: true, suggested_name: Some(name), .. } => name,
            FuzzyMatchResult { suggested_name: Some(suggestion), score: Some(score), .. } => {
                if score >= FUZZY_MATCH_THRESHOLD {
                    if ask_user_confirmation(task_name, &suggestion) {
                        suggestion
                    } else {
                        println!("Operation cancelled.");
                        return Ok(());
                    }
                } else {
                    println!("Task '{}' not found.", task_name);
                    return Ok(());
                }
            }
            _ => {
                println!("Task '{}' not found.", task_name);
                return Ok(());
            }
        };
        
        let rows_affected = self.conn.execute(
            "UPDATE tasks SET priority = ?1 WHERE name = ?2",
            [&priority.to_string(), &actual_task_name],
        )?;
        
        if rows_affected == 0 {
            println!("Task '{}' not found in database", actual_task_name);
        } else {
            println!("Task '{}' priority updated to {}", actual_task_name, priority);
        }
        
        Ok(())
    }
    
    pub fn list_tasks(&self) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "SELECT name, priority, status, created_at FROM tasks ORDER BY priority, created_at"
        )?;
        
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,  // name
                row.get::<_, i32>(1)?,     // priority
                row.get::<_, String>(2)?,  // status
                row.get::<_, String>(3)?,  // created_at
            ))
        })?;
        
        println!("Tasks:");
        println!("------");
        for row in rows {
            let (name, priority, status, created_at) = row?;
            println!("{} | Priority: {} | Status: {} | Created: {}", 
                     name, priority, status, created_at);
        }
        
        Ok(())
    }
    
    pub fn list_aides(&self) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "SELECT a.name, COUNT(d.id) as data_count 
             FROM aides a 
             LEFT JOIN data d ON a.id = d.aide_id 
             GROUP BY a.name
             ORDER BY a.name"
        )?;
        
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,  // name
                row.get::<_, i32>(1)?,     // data_count
            ))
        })?;
        
        println!("Aides:");
        println!("------");
        for row in rows {
            let (name, data_count) = row?;
            println!("{} | Data entries: {}", name, data_count);
        }
        
        Ok(())
    }
    
    pub fn edit_task(&self, task_name: &str) -> Result<()> {
        // Use fuzzy matching to find the task
        let fuzzy_result = self.find_fuzzy_task_match(task_name)?;
        
        let actual_task_name = match fuzzy_result {
            FuzzyMatchResult { exact_match: true, suggested_name: Some(name), .. } => name,
            FuzzyMatchResult { suggested_name: Some(suggestion), score: Some(score), .. } => {
                if score >= FUZZY_MATCH_THRESHOLD {
                    if ask_user_confirmation(task_name, &suggestion) {
                        suggestion
                    } else {
                        println!("Operation cancelled.");
                        return Ok(());
                    }
                } else {
                    println!("Task '{}' not found.", task_name);
                    return Ok(());
                }
            }
            _ => {
                println!("Task '{}' not found.", task_name);
                return Ok(());
            }
        };
        
        let task_log_file: String = match self.conn.query_row(
            "SELECT task_log_file_path FROM tasks WHERE name = ?1",
            [&actual_task_name],
            |row| row.get(0),
        ) {
            Ok(path) => path,
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                println!("Task '{}' not found in database", actual_task_name);
                return Ok(());
            }
            Err(e) => return Err(e.into()),
        };
        
        let status = Command::new("vi")
            .arg(&task_log_file)
            .status();
        
        match status {
            Ok(exit_status) => {
                if !exit_status.success() {
                    println!("Editor exited with status: {:?}", exit_status);
                }
            }
            Err(e) => {
                println!("Failed to open vi editor: {}", e);
                println!("Task log file is at: {}", task_log_file);
            }
        }
        
        Ok(())
    }

    pub fn update_aide_content(&mut self, aide_name: &str, new_content: &str) -> Result<()> {
        // Use fuzzy matching to find the aide
        let fuzzy_result = self.find_fuzzy_aide_match(aide_name)?;
        
        let actual_aide_name = match fuzzy_result {
            FuzzyMatchResult { exact_match: true, suggested_name: Some(name), .. } => name,
            FuzzyMatchResult { suggested_name: Some(suggestion), score: Some(score), .. } => {
                if score >= FUZZY_MATCH_THRESHOLD {
                    if ask_user_confirmation(aide_name, &suggestion) {
                        suggestion
                    } else {
                        println!("Operation cancelled.");
                        return Ok(());
                    }
                } else {
                    println!("Aide '{}' not found.", aide_name);
                    return Ok(());
                }
            }
            _ => {
                println!("Aide '{}' not found.", aide_name);
                return Ok(());
            }
        };
        
        let rows_affected = self.conn.execute(
            "UPDATE data SET command_output = ?1 WHERE aide_id = (SELECT id FROM aides WHERE name = ?2)",
            [new_content, &actual_aide_name],
        )?;
        
        if rows_affected == 0 {
            // If no existing data, create a new entry
            self.add_data(&actual_aide_name, "TUI Edit", None)?;
        }
        
        Ok(())
    }

    pub fn add_task_log(&self, task_name: &str, log_text: &str) -> Result<()> {
        // Use fuzzy matching to find the task
        let fuzzy_result = self.find_fuzzy_task_match(task_name)?;
        
        let actual_task_name = match fuzzy_result {
            FuzzyMatchResult { exact_match: true, suggested_name: Some(name), .. } => name,
            FuzzyMatchResult { suggested_name: Some(suggestion), score: Some(score), .. } => {
                if score >= FUZZY_MATCH_THRESHOLD {
                    if ask_user_confirmation(task_name, &suggestion) {
                        suggestion
                    } else {
                        println!("Operation cancelled.");
                        return Ok(());
                    }
                } else {
                    println!("Task '{}' not found.", task_name);
                    return Ok(());
                }
            }
            _ => {
                println!("Task '{}' not found.", task_name);
                return Ok(());
            }
        };
        
        let task_log_file: String = match self.conn.query_row(
            "SELECT task_log_file_path FROM tasks WHERE name = ?1",
            [&actual_task_name],
            |row| row.get(0),
        ) {
            Ok(path) => path,
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                println!("Task '{}' not found in database", actual_task_name);
                return Ok(());
            }
            Err(e) => return Err(e.into()),
        };
        
        // Read existing content
        let mut content = if PathBuf::from(&task_log_file).exists() {
            fs::read_to_string(&task_log_file)?
        } else {
            format!("Task: {}\n\n--- Task Log ---\n", actual_task_name)
        };
        
        // Add timestamp and new log entry
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S");
        let log_entry = format!("\n[{}] {}", timestamp, log_text);
        content.push_str(&log_entry);
        
        // Write back to file
        fs::write(&task_log_file, content)?;
        println!("Log entry added to task '{}'", actual_task_name);
        
        Ok(())
    }

    pub fn write_aide(&self, aide_name: &str) -> Result<()> {
        // Use fuzzy matching to find the aide
        let fuzzy_result = self.find_fuzzy_aide_match(aide_name)?;
        
        let actual_aide_name = match fuzzy_result {
            FuzzyMatchResult { exact_match: true, suggested_name: Some(name), .. } => name,
            FuzzyMatchResult { suggested_name: Some(suggestion), score: Some(score), .. } => {
                if score >= FUZZY_MATCH_THRESHOLD {
                    if ask_user_confirmation(aide_name, &suggestion) {
                        suggestion
                    } else {
                        println!("Operation cancelled.");
                        return Ok(());
                    }
                } else {
                    println!("Aide '{}' not found.", aide_name);
                    return Ok(());
                }
            }
            _ => {
                println!("Aide '{}' not found.", aide_name);
                return Ok(());
            }
        };
        
        // Construct file path (all aides are now files)
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let aide_dir = PathBuf::from(&home_dir).join(".aide");
        let file_path = aide_dir.join(format!("{}.txt", actual_aide_name));
        
        // Create file if it doesn't exist
        if !file_path.exists() {
            fs::create_dir_all(&aide_dir)?;
            let initial_content = format!("# {}\n\nCreated: {}\n\n", 
                                        actual_aide_name, 
                                        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"));
            fs::write(&file_path, initial_content)?;
            println!("Created new file: {}", file_path.display());
        }
        
        // Try editors in order of preference: vim, vi, nano
        let editors = ["vim", "vi", "nano"];
        let mut editor_found = false;
        
        for editor in &editors {
            // Check if editor is available
            if Command::new("which")
                .arg(editor)
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
            {
                println!("Opening {} with {}...", file_path.display(), editor);
                let status = Command::new(editor)
                    .arg(&file_path)
                    .status();
                
                match status {
                    Ok(exit_status) => {
                        if exit_status.success() {
                            println!("File edited successfully with {}.", editor);
                        } else {
                            println!("Editor {} exited with status: {:?}", editor, exit_status);
                        }
                        editor_found = true;
                        break;
                    }
                    Err(e) => {
                        println!("Failed to open {} editor: {}", editor, e);
                        continue;
                    }
                }
            }
        }
        
        if !editor_found {
            println!("No suitable editor found. Tried: {}", editors.join(", "));
            println!("File is located at: {}", file_path.display());
            println!("You can edit it manually with any text editor.");
            
            // Try to use $EDITOR environment variable as last resort
            if let Ok(editor_env) = std::env::var("EDITOR") {
                println!("Trying $EDITOR environment variable: {}", editor_env);
                let status = Command::new(&editor_env)
                    .arg(&file_path)
                    .status();
                    
                match status {
                    Ok(exit_status) => {
                        if exit_status.success() {
                            println!("File edited successfully with {}.", editor_env);
                        } else {
                            println!("Editor {} exited with status: {:?}", editor_env, exit_status);
                        }
                    }
                    Err(e) => {
                        println!("Failed to open {} editor: {}", editor_env, e);
                    }
                }
            }
        }
        
        Ok(())
    }

    pub fn get_all_tasks(&self) -> Result<Vec<TaskItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT name, priority, status, created_at FROM tasks ORDER BY priority, created_at"
        )?;
        
        let rows = stmt.query_map([], |row| {
            Ok(TaskItem {
                name: row.get(0)?,
                priority: row.get(1)?,
                status: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        
        let mut tasks = Vec::new();
        for row in rows {
            tasks.push(row?);
        }
        
        Ok(tasks)
    }

    pub fn get_all_aides(&self) -> Result<Vec<AideItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT a.name,
                    GROUP_CONCAT(d.input_text, '|||') as all_inputs,
                    GROUP_CONCAT(d.command_output, '|||') as all_outputs
             FROM aides a 
             LEFT JOIN data d ON a.id = d.aide_id 
             GROUP BY a.name
             ORDER BY a.name"
        )?;
        
        let rows = stmt.query_map([], |row| {
            Ok(AideItem {
                name: row.get(0)?,
                input_text: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                command_output: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            })
        })?;
        
        let mut aides = Vec::new();
        for row in rows {
            aides.push(row?);
        }
        
        Ok(aides)
    }

    pub fn get_all_configs(&self) -> Result<Vec<ConfigItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT key_name, value, description, created_at, updated_at 
             FROM config_data 
             ORDER BY key_name"
        )?;
        
        let rows = stmt.query_map([], |row| {
            Ok(ConfigItem {
                key_name: row.get(0)?,
                value: row.get(1)?,
                description: row.get::<_, Option<String>>(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?;
        
        let mut configs = Vec::new();
        for row in rows {
            configs.push(row?);
        }
        
        Ok(configs)
    }

    pub fn set_config(&mut self, key: &str, value: &str) -> Result<()> {
        // Use fuzzy matching to find existing config key
        let fuzzy_result = self.find_fuzzy_config_match(key)?;
        
        let actual_config_key = match fuzzy_result {
            FuzzyMatchResult { exact_match: true, suggested_name: Some(name), .. } => {
                // Exact match found, update existing config
                println!("Updating existing config key '{}'", name);
                name
            }
            FuzzyMatchResult { suggested_name: Some(suggestion), score: Some(score), .. } => {
                if score >= FUZZY_MATCH_THRESHOLD {
                    if ask_user_confirmation(key, &suggestion) {
                        // User confirmed, update existing config
                        println!("Updating existing config key '{}'", suggestion);
                        suggestion
                    } else {
                        // User declined, create new config with original key
                        println!("Creating new config key '{}'", key);
                        key.to_string()
                    }
                } else {
                    // Score too low, create new config
                    println!("Creating new config key '{}'", key);
                    key.to_string()
                }
            }
            _ => {
                // No suggestions, create new config
                println!("Creating new config key '{}'", key);
                key.to_string()
            }
        };

        // Check if config already exists
        let existing_value: Option<String> = self.conn.query_row(
            "SELECT value FROM config_data WHERE key_name = ?1",
            [&actual_config_key],
            |row| Ok(row.get(0)?),
        ).optional()?;

        if let Some(old_value) = existing_value {
            // Update existing config
            self.conn.execute(
                "UPDATE config_data SET value = ?1, updated_at = CURRENT_TIMESTAMP WHERE key_name = ?2",
                [value, &actual_config_key],
            )?;
            println!("Config '{}' updated from '{}' to '{}'", actual_config_key, old_value, value);
        } else {
            // Insert new config
            self.conn.execute(
                "INSERT INTO config_data (key_name, value) VALUES (?1, ?2)",
                [&actual_config_key, value],
            )?;
            println!("Config '{}' set to '{}'", actual_config_key, value);
            
            // Use incremental update instead of full rebuild
            if let Some(ref mut index) = self.config_index {
                index.add_entity(actual_config_key.to_string())?;
            }
        }

        Ok(())
    }

    pub fn get_config(&self, key: &str) -> Result<Option<String>> {
        // Use fuzzy matching to find config key
        let fuzzy_result = self.find_fuzzy_config_match(key)?;
        
        let actual_config_key = match fuzzy_result {
            FuzzyMatchResult { exact_match: true, suggested_name: Some(name), .. } => name,
            FuzzyMatchResult { suggested_name: Some(suggestion), score: Some(score), .. } => {
                if score >= FUZZY_MATCH_THRESHOLD {
                    if ask_user_confirmation(key, &suggestion) {
                        suggestion
                    } else {
                        println!("Operation cancelled.");
                        return Ok(None);
                    }
                } else {
                    println!("Config key '{}' not found.", key);
                    return Ok(None);
                }
            }
            _ => {
                println!("Config key '{}' not found.", key);
                return Ok(None);
            }
        };

        let value: Option<String> = self.conn.query_row(
            "SELECT value FROM config_data WHERE key_name = ?1",
            [&actual_config_key],
            |row| Ok(row.get(0)?),
        ).optional()?;

        if let Some(ref val) = value {
            println!("Config '{}' = '{}'", actual_config_key, val);
        }

        Ok(value)
    }

    pub fn list_configs(&self) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "SELECT key_name, value, description, created_at, updated_at 
             FROM config_data 
             ORDER BY key_name"
        )?;
        
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,  // key_name
                row.get::<_, String>(1)?,  // value
                row.get::<_, Option<String>>(2)?,  // description
                row.get::<_, String>(3)?,  // created_at
                row.get::<_, String>(4)?,  // updated_at
            ))
        })?;
        
        println!("Configuration:");
        println!("--------------");
        for row in rows {
            let (key_name, value, description, created_at, updated_at) = row?;
            println!("{} = {}", key_name, value);
            if let Some(desc) = description {
                println!("  Description: {}", desc);
            }
            println!("  Created: {} | Updated: {}", created_at, updated_at);
            println!();
        }
        
        Ok(())
    }

    pub fn delete_config(&mut self, key: &str) -> Result<()> {
        // Use fuzzy matching to find config key
        let fuzzy_result = self.find_fuzzy_config_match(key)?;
        
        let actual_config_key = match fuzzy_result {
            FuzzyMatchResult { exact_match: true, suggested_name: Some(name), .. } => name,
            FuzzyMatchResult { suggested_name: Some(suggestion), score: Some(score), .. } => {
                if score >= FUZZY_MATCH_THRESHOLD {
                    if ask_user_confirmation(key, &suggestion) {
                        suggestion
                    } else {
                        println!("Operation cancelled.");
                        return Ok(());
                    }
                } else {
                    println!("Config key '{}' not found.", key);
                    return Ok(());
                }
            }
            _ => {
                println!("Config key '{}' not found.", key);
                return Ok(());
            }
        };

        let rows_affected = self.conn.execute(
            "DELETE FROM config_data WHERE key_name = ?1",
            [&actual_config_key],
        )?;

        if rows_affected > 0 {
            println!("Config '{}' deleted successfully", actual_config_key);
            // Use incremental removal instead of full rebuild
            if let Some(ref mut index) = self.config_index {
                index.remove_entity(&actual_config_key)?;
            }
        } else {
            println!("Config '{}' not found in database", actual_config_key);
        }

        Ok(())
    }

    pub fn update_config_value(&mut self, key: &str, value: &str) -> Result<()> {
        let rows_affected = self.conn.execute(
            "UPDATE config_data SET value = ?1, updated_at = CURRENT_TIMESTAMP WHERE key_name = ?2",
            [value, key],
        )?;

        if rows_affected > 0 {
            println!("Config '{}' updated to '{}'", key, value);
        } else {
            println!("Config key '{}' not found", key);
        }

        Ok(())
    }

    // Clear all data and rebuild indexes
    pub fn clear_all_data(&mut self) -> Result<()> {
        // Clear all data from tables
        self.conn.execute("DELETE FROM data", [])?;
        self.conn.execute("DELETE FROM tasks", [])?;
        self.conn.execute("DELETE FROM aides", [])?;
        self.conn.execute("DELETE FROM config_data", [])?;
        
        // Recreate default task_log aide if it doesn't exist
        let _ = self.conn.execute(
            "INSERT OR IGNORE INTO aides (name) VALUES ('task_log')",
            [],
        );
        
        // Rebuild indexes (will be empty now)
        self.rebuild_task_index()?;
        self.rebuild_aide_index()?;
        self.rebuild_config_index()?;
        
        println!("All data cleared successfully!");
        Ok(())
    }
}