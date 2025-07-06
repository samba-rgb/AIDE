use anyhow::Result;
use clap::{Parser, Subcommand};
use fuzzy_matcher::FuzzyMatcher;
use rusqlite::Connection;
use std::path::PathBuf;
use std::process::Command;
use std::fs;
use chrono::Utc;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap},
    Frame, Terminal,
};
use std::io;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
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
        #[arg(value_name = "INPUT_TEXT")]
        input_text: String,
        #[arg(value_name = "COMMAND_OUTPUT")]
        command_output: Option<String>,
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
    /// Launch TUI interface
    Tui,
}

#[derive(Debug, Clone)]
struct TaskItem {
    name: String,
    priority: i32,
    status: String,
    created_at: String,
}

#[derive(Debug, Clone)]
struct AideItem {
    name: String,
    aide_type: String,
    input_text: String,
    command_output: String,
}

#[derive(Debug, Clone, PartialEq)]
enum PopupMode {
    None,
    TaskPriority,
    TaskStatus,
    AideEdit,
    TextEditor,
}

#[derive(Debug, Clone)]
struct TextEditor {
    content: Vec<String>,
    cursor_row: usize,
    cursor_col: usize,
    scroll_offset: usize,
    title: String,
    is_dirty: bool,
}

impl TextEditor {
    fn new(title: String, content: String) -> Self {
        let lines: Vec<String> = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(|s| s.to_string()).collect()
        };
        
        TextEditor {
            content: lines,
            cursor_row: 0,
            cursor_col: 0,
            scroll_offset: 0,
            title,
            is_dirty: false,
        }
    }
    
    fn insert_char(&mut self, c: char) {
        if self.cursor_row >= self.content.len() {
            self.content.push(String::new());
        }
        
        let line = &mut self.content[self.cursor_row];
        if self.cursor_col > line.len() {
            self.cursor_col = line.len();
        }
        
        line.insert(self.cursor_col, c);
        self.cursor_col += 1;
        self.is_dirty = true;
    }
    
    fn insert_newline(&mut self) {
        if self.cursor_row >= self.content.len() {
            self.content.push(String::new());
        }
        
        let line = &mut self.content[self.cursor_row];
        let remaining = line.split_off(self.cursor_col);
        
        self.cursor_row += 1;
        self.cursor_col = 0;
        self.content.insert(self.cursor_row, remaining);
        self.is_dirty = true;
    }
    
    fn delete_char(&mut self) {
        if self.cursor_row >= self.content.len() {
            return;
        }
        
        let line = &mut self.content[self.cursor_row];
        if self.cursor_col > 0 && self.cursor_col <= line.len() {
            line.remove(self.cursor_col - 1);
            self.cursor_col -= 1;
            self.is_dirty = true;
        } else if self.cursor_col == 0 && self.cursor_row > 0 {
            // Join with previous line
            let current_line = self.content.remove(self.cursor_row);
            self.cursor_row -= 1;
            self.cursor_col = self.content[self.cursor_row].len();
            self.content[self.cursor_row].push_str(&current_line);
            self.is_dirty = true;
        }
    }
    
    fn move_cursor_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.content[self.cursor_row].len();
        }
    }
    
    fn move_cursor_right(&mut self) {
        if self.cursor_row < self.content.len() {
            let line_len = self.content[self.cursor_row].len();
            if self.cursor_col < line_len {
                self.cursor_col += 1;
            } else if self.cursor_row < self.content.len() - 1 {
                self.cursor_row += 1;
                self.cursor_col = 0;
            }
        }
    }
    
    fn move_cursor_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            let line_len = self.content[self.cursor_row].len();
            if self.cursor_col > line_len {
                self.cursor_col = line_len;
            }
        }
    }
    
    fn move_cursor_down(&mut self) {
        if self.cursor_row < self.content.len() - 1 {
            self.cursor_row += 1;
            let line_len = self.content[self.cursor_row].len();
            if self.cursor_col > line_len {
                self.cursor_col = line_len;
            }
        }
    }
    
    fn get_content(&self) -> String {
        self.content.join("\n")
    }
}

struct App {
    db: Database,
    current_tab: usize,
    tasks: Vec<TaskItem>,
    aides: Vec<AideItem>,
    task_list_state: ListState,
    aide_list_state: ListState,
    should_quit: bool,
    // UI state
    show_priority_popup: bool,
    show_status_popup: bool,
    show_aide_popup: bool,
    input_buffer: String,
    popup_mode: PopupMode,
    // Text editor
    text_editor: Option<TextEditor>,
    editor_save_callback: Option<EditorCallback>,
}

#[derive(Debug, Clone)]
enum EditorCallback {
    SaveTask(String),
    SaveAide(String),
}

impl App {
    fn new(db: Database) -> Result<Self> {
        let mut app = App {
            db,
            current_tab: 0,
            tasks: Vec::new(),
            aides: Vec::new(),
            task_list_state: ListState::default(),
            aide_list_state: ListState::default(),
            should_quit: false,
            show_priority_popup: false,
            show_status_popup: false,
            show_aide_popup: false,
            input_buffer: String::new(),
            popup_mode: PopupMode::None,
            text_editor: None,
            editor_save_callback: None,
        };
        app.refresh_data()?;
        Ok(app)
    }

    fn refresh_data(&mut self) -> Result<()> {
        self.tasks = self.db.get_all_tasks()?;
        self.aides = self.db.get_all_aides()?;
        Ok(())
    }

    fn next_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % 2;
    }

    fn previous_tab(&mut self) {
        self.current_tab = if self.current_tab == 0 { 1 } else { 0 };
    }

    fn next_item(&mut self) {
        match self.current_tab {
            0 => {
                let i = match self.task_list_state.selected() {
                    Some(i) => {
                        if i >= self.tasks.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.task_list_state.select(Some(i));
            }
            1 => {
                let i = match self.aide_list_state.selected() {
                    Some(i) => {
                        if i >= self.aides.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.aide_list_state.select(Some(i));
            }
            _ => {}
        }
    }

    fn previous_item(&mut self) {
        match self.current_tab {
            0 => {
                let i = match self.task_list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            self.tasks.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.task_list_state.select(Some(i));
            }
            1 => {
                let i = match self.aide_list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            self.aides.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.aide_list_state.select(Some(i));
            }
            _ => {}
        }
    }

    fn show_priority_popup(&mut self) {
        self.show_priority_popup = true;
        self.popup_mode = PopupMode::TaskPriority;
        self.input_buffer.clear();
    }

    fn show_status_popup(&mut self) {
        self.show_status_popup = true;
        self.popup_mode = PopupMode::TaskStatus;
        self.input_buffer.clear();
    }

    fn show_aide_popup(&mut self) {
        self.show_aide_popup = true;
        self.popup_mode = PopupMode::AideEdit;
        self.input_buffer.clear();
    }

    fn close_popup(&mut self) {
        self.show_priority_popup = false;
        self.show_status_popup = false;
        self.show_aide_popup = false;
        self.popup_mode = PopupMode::None;
        self.input_buffer.clear();
    }

    fn handle_popup_input(&mut self, c: char) -> Result<()> {
        match self.popup_mode {
            PopupMode::TaskPriority => {
                if c.is_ascii_digit() && c >= '1' && c <= '5' {
                    if let Some(i) = self.task_list_state.selected() {
                        if let Some(task) = self.tasks.get(i) {
                            let priority = c as u8 - b'0';
                            self.db.update_task_priority(&task.name, priority)?;
                            self.refresh_data()?;
                        }
                    }
                    self.close_popup();
                }
            }
            PopupMode::TaskStatus => {
                match c {
                    '1' => {
                        if let Some(i) = self.task_list_state.selected() {
                            if let Some(task) = self.tasks.get(i) {
                                self.db.update_task_status(&task.name, "created")?;
                                self.refresh_data()?;
                            }
                        }
                        self.close_popup();
                    }
                    '2' => {
                        if let Some(i) = self.task_list_state.selected() {
                            if let Some(task) = self.tasks.get(i) {
                                self.db.update_task_status(&task.name, "in_progress")?;
                                self.refresh_data()?;
                            }
                        }
                        self.close_popup();
                    }
                    '3' => {
                        if let Some(i) = self.task_list_state.selected() {
                            if let Some(task) = self.tasks.get(i) {
                                self.db.update_task_status(&task.name, "completed")?;
                                self.refresh_data()?;
                            }
                        }
                        self.close_popup();
                    }
                    _ => {}
                }
            }
            PopupMode::AideEdit => {
                if c == '\n' || c == '\r' {
                    self.handle_aide_edit()?;
                } else if c.is_ascii() && c != '\x08' {
                    self.input_buffer.push(c);
                }
            }
            PopupMode::TextEditor => {
                // Text editor input is handled separately in handle_text_editor_input
            }
            PopupMode::None => {}
        }
        Ok(())
    }

    fn handle_backspace(&mut self) {
        if matches!(self.popup_mode, PopupMode::AideEdit) {
            self.input_buffer.pop();
        }
    }

    fn open_text_editor(&mut self, title: String, content: String, callback: EditorCallback) {
        self.text_editor = Some(TextEditor::new(title, content));
        self.editor_save_callback = Some(callback);
        self.popup_mode = PopupMode::TextEditor;
    }

    fn close_text_editor(&mut self, save: bool) -> Result<()> {
        if let Some(editor) = &self.text_editor {
            if save && editor.is_dirty {
                if let Some(callback) = &self.editor_save_callback {
                    let content = editor.get_content();
                    match callback {
                        EditorCallback::SaveTask(task_name) => {
                            // Save task log content to file
                            let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                            let tasks_dir = PathBuf::from(&home_dir).join(".aide").join("tasks");
                            let task_file = tasks_dir.join(format!("{}.txt", task_name));
                            fs::write(&task_file, &content)?;
                        }
                        EditorCallback::SaveAide(aide_name) => {
                            self.db.update_aide_content(aide_name, &content)?;
                            self.refresh_data()?;
                        }
                    }
                }
            }
        }
        
        self.text_editor = None;
        self.editor_save_callback = None;
        self.popup_mode = PopupMode::None;
        Ok(())
    }

    fn handle_text_editor_input(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        if let Some(editor) = &mut self.text_editor {
            match key {
                KeyCode::Char(c) => {
                    if modifiers.contains(KeyModifiers::CONTROL) {
                        match c {
                            's' => {
                                // Save and close
                                self.close_text_editor(true)?;
                            }
                            'q' => {
                                // Quit without saving
                                self.close_text_editor(false)?;
                            }
                            _ => {}
                        }
                    } else {
                        editor.insert_char(c);
                    }
                }
                KeyCode::Enter => {
                    editor.insert_newline();
                }
                KeyCode::Backspace => {
                    editor.delete_char();
                }
                KeyCode::Left => {
                    editor.move_cursor_left();
                }
                KeyCode::Right => {
                    editor.move_cursor_right();
                }
                KeyCode::Up => {
                    editor.move_cursor_up();
                }
                KeyCode::Down => {
                    editor.move_cursor_down();
                }
                KeyCode::Esc => {
                    self.close_text_editor(false)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn edit_selected_task(&mut self) -> Result<()> {
        if let Some(i) = self.task_list_state.selected() {
            if let Some(task) = self.tasks.get(i) {
                // Read existing task log content
                let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                let tasks_dir = PathBuf::from(&home_dir).join(".aide").join("tasks");
                let task_file = tasks_dir.join(format!("{}.txt", task.name));
                
                let content = if task_file.exists() {
                    fs::read_to_string(&task_file).unwrap_or_default()
                } else {
                    format!("Task: {}\nStatus: {}\nPriority: {}\nCreated: {}\n\n--- Task Log ---\n", 
                            task.name, task.status, task.priority, task.created_at)
                };
                
                self.open_text_editor(
                    format!("Edit Task: {}", task.name),
                    content,
                    EditorCallback::SaveTask(task.name.clone())
                );
            }
        }
        Ok(())
    }

    fn edit_selected_aide(&mut self) -> Result<()> {
        if let Some(i) = self.aide_list_state.selected() {
            if let Some(aide) = self.aides.get(i) {
                self.open_text_editor(
                    format!("Edit Aide: {}", aide.name),
                    aide.command_output.clone(),
                    EditorCallback::SaveAide(aide.name.clone())
                );
            }
        }
        Ok(())
    }

    fn show_aide_edit_popup(&mut self) {
        if let Some(i) = self.aide_list_state.selected() {
            if let Some(aide) = self.aides.get(i) {
                self.input_buffer = aide.command_output.clone();
                self.show_aide_popup = true;
                self.popup_mode = PopupMode::AideEdit;
            }
        }
    }

    fn handle_aide_edit(&mut self) -> Result<()> {
        if let Some(i) = self.aide_list_state.selected() {
            if let Some(aide) = self.aides.get(i) {
                self.db.update_aide_content(&aide.name, &self.input_buffer)?;
                self.refresh_data()?;
            }
        }
        self.close_popup();
        Ok(())
    }
}

struct Database {
    conn: Connection,
}

impl Database {
    fn new() -> Result<Self> {
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let db_path = PathBuf::from(home_dir).join(".aide.db");
        
        let conn = Connection::open(db_path)?;
        
        // Create tables if they don't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS aides (
                id INTEGER PRIMARY KEY,
                name TEXT UNIQUE NOT NULL,
                aide_type TEXT NOT NULL
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
        
        // Create default task_log aide if it doesn't exist
        let _ = conn.execute(
            "INSERT OR IGNORE INTO aides (name, aide_type) VALUES ('task_log', 'file')",
            [],
        );
        
        Ok(Database { conn })
    }
    
    fn create_aide(&self, name: &str, aide_type: &str) -> Result<()> {
        if aide_type != "text" && aide_type != "file" {
            println!("Error: aide_type must be 'text' or 'file'");
            return Ok(());
        }
        
        match self.conn.execute(
            "INSERT INTO aides (name, aide_type) VALUES (?1, ?2)",
            [name, aide_type],
        ) {
            Ok(_) => {
                println!("Aide '{}' of type '{}' created successfully", name, aide_type);
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
    
    fn add_data(&self, name: &str, input_text: &str, command_output: Option<String>) -> Result<()> {
        // First, find the aide by name and get its type
        let (aide_id, aide_type): (i64, String) = match self.conn.query_row(
            "SELECT id, aide_type FROM aides WHERE name = ?1",
            [name],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ) {
            Ok((id, aide_type)) => (id, aide_type),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                println!("Aide '{}' not found", name);
                return Ok(());
            }
            Err(e) => return Err(e.into()),
        };
        
        let final_output = if aide_type == "file" {
            // For file type aides, create actual files
            let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let aide_dir = PathBuf::from(&home_dir).join(".aide").join(&name);
            fs::create_dir_all(&aide_dir)?;
            
            // Create the actual file
            let file_path = aide_dir.join(format!("{}.txt", input_text.replace(" ", "_")));
            
            let content = command_output.unwrap_or_else(|| {
                format!("File: {}\nCreated: {}\nAide: {}\n\n--- Content ---\n", 
                       input_text, 
                       chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
                       name)
            });
            
            fs::write(&file_path, &content)?;
            println!("File created: {}", file_path.display());
            
            content
        } else {
            // For text type, use the provided command_output or empty string
            command_output.unwrap_or_else(|| {
                println!("Error: command_output is required for text type");
                String::new()
            })
        };
        
        if final_output.is_empty() {
            println!("No content provided, skipping...");
            return Ok(());
        }
        
        self.conn.execute(
            "INSERT INTO data (aide_id, input_text, command_output) VALUES (?1, ?2, ?3)",
            [&aide_id.to_string(), input_text, &final_output],
        )?;
        
        println!("Data added successfully to aide '{}'", name);
        Ok(())
    }
    
    fn search_by_input(&self, input_text: &str) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "SELECT d.input_text, d.command_output, a.name, a.aide_type 
             FROM data d 
             JOIN aides a ON d.aide_id = a.id"
        )?;
        
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,  // input_text
                row.get::<_, String>(1)?,  // command_output
                row.get::<_, String>(2)?,  // name
                row.get::<_, String>(3)?,  // aide_type
            ))
        })?;
        
        let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
        let mut best_match: Option<(i64, String, String, String)> = None;
        
        for row in rows {
            let (db_input, output, name, _aide_type) = row?;
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
    
    fn search_by_command(&self, input_text: &str) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "SELECT d.input_text, d.command_output, a.name, a.aide_type 
             FROM data d 
             JOIN aides a ON d.aide_id = a.id"
        )?;
        
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,  // input_text
                row.get::<_, String>(1)?,  // command_output
                row.get::<_, String>(2)?,  // name
                row.get::<_, String>(3)?,  // aide_type
            ))
        })?;
        
        let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
        let mut best_match: Option<(i64, String, String, String)> = None;
        
        for row in rows {
            let (db_input, output, name, _aide_type) = row?;
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
    
    fn create_task(&self, task_name: &str) -> Result<()> {
        // Create tasks directory if it doesn't exist
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let tasks_dir = PathBuf::from(&home_dir).join(".aide").join("tasks");
        fs::create_dir_all(&tasks_dir)?;
        
        // Create task log file path
        let task_log_file = tasks_dir.join(format!("{}.txt", task_name));
        
        // Check if task already exists
        let exists = self.conn.query_row(
            "SELECT 1 FROM tasks WHERE name = ?1",
            [task_name],
            |_| Ok(()),
        );
        
        if exists.is_ok() {
            println!("Task '{}' already exists. Opening task log file...", task_name);
        } else {
            // Create new task
            self.conn.execute(
                "INSERT INTO tasks (name, priority, status, task_log_file_path) VALUES (?1, 3, 'created', ?2)",
                [task_name, &task_log_file.to_string_lossy()],
            )?;
            
            // Create initial task log content
            let initial_content = format!(
                "Task: {}\nStatus: created\nPriority: 3\nCreated: {}\n\n--- Task Log ---\n",
                task_name,
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
            );
            
            fs::write(&task_log_file, initial_content)?;
            println!("Task '{}' created successfully!", task_name);
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
    
    fn update_task_status(&self, task_name: &str, status: &str) -> Result<()> {
        let valid_statuses = ["created", "in_progress", "completed"];
        if !valid_statuses.contains(&status) {
            println!("Invalid status. Valid statuses are: created, in_progress, completed");
            return Ok(());
        }
        
        let rows_affected = self.conn.execute(
            "UPDATE tasks SET status = ?1 WHERE name = ?2",
            [status, task_name],
        )?;
        
        if rows_affected == 0 {
            println!("Task '{}' not found", task_name);
        } else {
            println!("Task '{}' status updated to '{}'", task_name, status);
        }
        
        Ok(())
    }
    
    fn update_task_priority(&self, task_name: &str, priority: u8) -> Result<()> {
        if priority < 1 || priority > 5 {
            println!("Invalid priority. Priority must be between 1 (highest) and 5 (lowest)");
            return Ok(());
        }
        
        let rows_affected = self.conn.execute(
            "UPDATE tasks SET priority = ?1 WHERE name = ?2",
            [&priority.to_string(), task_name],
        )?;
        
        if rows_affected == 0 {
            println!("Task '{}' not found", task_name);
        } else {
            println!("Task '{}' priority updated to {}", task_name, priority);
        }
        
        Ok(())
    }
    
    fn list_tasks(&self) -> Result<()> {
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
    
    fn list_aides(&self) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "SELECT a.name, a.aide_type, COUNT(d.id) as data_count 
             FROM aides a 
             LEFT JOIN data d ON a.id = d.aide_id 
             GROUP BY a.name, a.aide_type 
             ORDER BY a.name"
        )?;
        
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,  // name
                row.get::<_, String>(1)?,  // aide_type
                row.get::<_, i32>(2)?,     // data_count
            ))
        })?;
        
        println!("Aides:");
        println!("------");
        for row in rows {
            let (name, aide_type, data_count) = row?;
            println!("{} | Type: {} | Data entries: {}", 
                     name, aide_type, data_count);
        }
        
        Ok(())
    }
    
    fn edit_task(&self, task_name: &str) -> Result<()> {
        let task_log_file: String = match self.conn.query_row(
            "SELECT task_log_file_path FROM tasks WHERE name = ?1",
            [task_name],
            |row| row.get(0),
        ) {
            Ok(path) => path,
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                println!("Task '{}' not found", task_name);
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

    fn update_aide_content(&self, aide_name: &str, new_content: &str) -> Result<()> {
        let rows_affected = self.conn.execute(
            "UPDATE data SET command_output = ?1 WHERE aide_id = (SELECT id FROM aides WHERE name = ?2)",
            [new_content, aide_name],
        )?;
        
        if rows_affected == 0 {
            // If no existing data, create a new entry
            self.add_data(aide_name, "TUI Edit", Some(new_content.to_string()))?;
        }
        
        Ok(())
    }

    fn add_task_log(&self, task_name: &str, log_text: &str) -> Result<()> {
        let task_log_file: String = match self.conn.query_row(
            "SELECT task_log_file_path FROM tasks WHERE name = ?1",
            [task_name],
            |row| row.get(0),
        ) {
            Ok(path) => path,
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                println!("Task '{}' not found", task_name);
                return Ok(());
            }
            Err(e) => return Err(e.into()),
        };
        
        // Read existing content
        let mut content = if PathBuf::from(&task_log_file).exists() {
            fs::read_to_string(&task_log_file)?
        } else {
            format!("Task: {}\n\n--- Task Log ---\n", task_name)
        };
        
        // Add timestamp and new log entry
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S");
        let log_entry = format!("\n[{}] {}", timestamp, log_text);
        content.push_str(&log_entry);
        
        // Write back to file
        fs::write(&task_log_file, content)?;
        println!("Log entry added to task '{}'", task_name);
        
        Ok(())
    }

    fn get_all_tasks(&self) -> Result<Vec<TaskItem>> {
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

    fn get_all_aides(&self) -> Result<Vec<AideItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT a.name, a.aide_type, 
                    GROUP_CONCAT(d.input_text, '|||') as all_inputs,
                    GROUP_CONCAT(d.command_output, '|||') as all_outputs
             FROM aides a 
             LEFT JOIN data d ON a.id = d.aide_id 
             GROUP BY a.name, a.aide_type
             ORDER BY a.name"
        )?;
        
        let rows = stmt.query_map([], |row| {
            Ok(AideItem {
                name: row.get(0)?,
                aide_type: row.get(1)?,
                input_text: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                command_output: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
            })
        })?;
        
        let mut aides = Vec::new();
        for row in rows {
            aides.push(row?);
        }
        
        Ok(aides)
    }
}

fn run_tui(db: Database) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(db)?;
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                // Handle text editor input first
                if app.popup_mode == PopupMode::TextEditor {
                    let _ = app.handle_text_editor_input(key.code, key.modifiers);
                } else if app.popup_mode != PopupMode::None {
                    match key.code {
                        KeyCode::Esc => {
                            app.close_popup();
                        }
                        KeyCode::Char(c) => {
                            let _ = app.handle_popup_input(c);
                        }
                        KeyCode::Backspace => {
                            app.handle_backspace();
                        }
                        _ => {}
                    }
                } else {
                    // Handle normal navigation
                    match key.code {
                        KeyCode::Char('q') => {
                            app.should_quit = true;
                        }
                        KeyCode::Tab => {
                            app.next_tab();
                        }
                        KeyCode::BackTab => {
                            app.previous_tab();
                        }
                        KeyCode::Down => {
                            app.next_item();
                        }
                        KeyCode::Up => {
                            app.previous_item();
                        }
                        KeyCode::Enter => {
                            if app.current_tab == 0 {
                                let _ = app.edit_selected_task();
                            } else if app.current_tab == 1 {
                                let _ = app.edit_selected_aide();
                            }
                        }
                        KeyCode::Char('r') => {
                            let _ = app.refresh_data();
                        }
                        KeyCode::Char('p') => {
                            if app.current_tab == 0 {
                                app.show_priority_popup();
                            }
                        }
                        KeyCode::Char('s') => {
                            if app.current_tab == 0 {
                                app.show_status_popup();
                            }
                        }
                        KeyCode::Char('e') => {
                            if app.current_tab == 1 {
                                app.show_aide_popup();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.area());

    let titles: Vec<Line> = ["Tasks", "Aides"]
        .iter()
        .cloned()
        .map(Line::from)
        .collect();
    
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Aide TUI"))
        .select(app.current_tab)
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
        );
    
    f.render_widget(tabs, chunks[0]);

    match app.current_tab {
        0 => render_tasks(f, app, chunks[1]),
        1 => render_aides(f, app, chunks[1]),
        _ => {}
    }

    // Render popups
    if app.show_priority_popup {
        let popup_area = centered_rect(50, 20, f.area());
        let block = Block::default()
            .title("Change Task Priority")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray));
        let content = Paragraph::new("Enter new priority (1-5):\n\n1 = Highest Priority\n2 = High Priority\n3 = Medium Priority\n4 = Low Priority\n5 = Lowest Priority\n\nPress ESC to cancel")
            .block(block)
            .alignment(ratatui::layout::Alignment::Center)
            .style(Style::default().fg(Color::White));
        
        f.render_widget(content, popup_area);
    }

    if app.show_status_popup {
        let popup_area = centered_rect(50, 20, f.area());
        let block = Block::default()
            .title("Change Task Status")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray));
        let content = Paragraph::new("Select new status:\n\n1. Created\n2. In Progress\n3. Completed\n\nPress ESC to cancel")
            .block(block)
            .alignment(ratatui::layout::Alignment::Center)
            .style(Style::default().fg(Color::White));
        
        f.render_widget(content, popup_area);
    }

    if app.show_aide_popup {
        let popup_area = centered_rect(60, 25, f.area());
        let block = Block::default()
            .title("Quick Edit Aide")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray));
        let content = Paragraph::new(format!("Enter input text for aide:\n\n{}\n\nPress ENTER to save\nPress ESC to cancel", app.input_buffer))
            .block(block)
            .alignment(ratatui::layout::Alignment::Center)
            .style(Style::default().fg(Color::White));
        
        f.render_widget(content, popup_area);
    }

    // Render text editor
    if let Some(editor) = &app.text_editor {
        let editor_area = centered_rect(90, 80, f.area());
        
        // Create the main editor block
        let block = Block::default()
            .title(format!("{} - Ctrl+S: Save & Close | Ctrl+Q: Quit | ESC: Cancel", &editor.title))
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black));
        
        let inner_area = block.inner(editor_area);
        f.render_widget(block, editor_area);
        
        // Calculate visible lines based on scroll offset
        let visible_height = inner_area.height as usize;
        let start_line = editor.scroll_offset;
        let end_line = (start_line + visible_height).min(editor.content.len());
        
        // Render the content
        let content_lines: Vec<Line> = editor.content[start_line..end_line]
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let actual_line = start_line + i;
                let is_cursor_line = actual_line == editor.cursor_row;
                
                if is_cursor_line {
                    // Show cursor position
                    let mut line_spans = Vec::new();
                    let line_chars: Vec<char> = line.chars().collect();
                    
                    // Add characters before cursor
                    if editor.cursor_col > 0 {
                        let before_cursor: String = line_chars[..editor.cursor_col.min(line_chars.len())].iter().collect();
                        line_spans.push(Span::styled(before_cursor, Style::default().fg(Color::White)));
                    }
                    
                    // Add cursor
                    let cursor_char = if editor.cursor_col < line_chars.len() {
                        line_chars[editor.cursor_col].to_string()
                    } else {
                        " ".to_string()
                    };
                    line_spans.push(Span::styled(cursor_char, Style::default().bg(Color::White).fg(Color::Black)));
                    
                    // Add characters after cursor
                    if editor.cursor_col < line_chars.len() {
                        let after_cursor: String = line_chars[editor.cursor_col + 1..].iter().collect();
                        if !after_cursor.is_empty() {
                            line_spans.push(Span::styled(after_cursor, Style::default().fg(Color::White)));
                        }
                    }
                    
                    Line::from(line_spans)
                } else {
                    Line::from(Span::styled(line, Style::default().fg(Color::White)))
                }
            })
            .collect();
        
        let editor_content = Paragraph::new(content_lines)
            .block(Block::default())
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(Color::White));
        
        f.render_widget(editor_content, inner_area);
    }
}

// Helper function to create centered rectangles for popups
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn render_tasks(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    let tasks: Vec<ListItem> = app
        .tasks
        .iter()
        .map(|task| {
            let status_color = match task.status.as_str() {
                "completed" => Color::Green,
                "in_progress" => Color::Yellow,
                "created" => Color::Blue,
                _ => Color::White,
            };
            
            ListItem::new(vec![Line::from(vec![
                Span::styled(
                    format!("{} ", task.name),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("[P{}] ", task.priority),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    format!("[{}]", task.status),
                    Style::default().fg(status_color),
                ),
            ])])
        })
        .collect();

    let tasks_list = List::new(tasks)
        .block(Block::default().borders(Borders::ALL).title("Tasks"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(tasks_list, chunks[0], &mut app.task_list_state);

    let selected_task = app.task_list_state.selected().and_then(|i| app.tasks.get(i));
    let info_text = if let Some(task) = selected_task {
        format!(
            "Task: {}\nPriority: {}\nStatus: {}\nCreated: {}\n\nControls:\n• Enter: Edit task log\n• p: Change priority\n• s: Change status\n• r: Refresh\n• q: Quit",
            task.name, task.priority, task.status, task.created_at
        )
    } else {
        "No task selected\n\nControls:\n• ↑/↓: Navigate\n• Enter: Edit task log\n• p: Change priority\n• s: Change status\n• r: Refresh\n• q: Quit".to_string()
    };

    let info_paragraph = Paragraph::new(info_text)
        .block(Block::default().borders(Borders::ALL).title("Task Info"))
        .style(Style::default().fg(Color::White));

    f.render_widget(info_paragraph, chunks[1]);
}

fn render_aides(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    let aides: Vec<ListItem> = app
        .aides
        .iter()
        .map(|aide| {
            let type_color = match aide.aide_type.as_str() {
                "file" => Color::Green,
                "text" => Color::Blue,
                _ => Color::White,
            };
            
            ListItem::new(vec![Line::from(vec![
                Span::styled(
                    format!("{} ", aide.name),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("[{}]", aide.aide_type),
                    Style::default().fg(type_color),
                ),
            ])])
        })
        .collect();

    let aides_list = List::new(aides)
        .block(Block::default().borders(Borders::ALL).title("Aides"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(aides_list, chunks[0], &mut app.aide_list_state);

    let selected_aide = app.aide_list_state.selected().and_then(|i| app.aides.get(i));
    
    if let Some(aide) = selected_aide {
        let (title, content) = match aide.aide_type.as_str() {
            "text" => {
                let title = format!("Text Entries - {}", aide.name);
                if aide.command_output.is_empty() {
                    let content = "No text entries available\n\nTo add entries:\n• aide add command \"description\" \"command text\"\n\nControls:\n• Enter: Edit content\n• e: Quick edit\n• r: Refresh\n• q: Quit".to_string();
                    (title, content)
                } else {
                    // Split concatenated entries
                    let inputs: Vec<&str> = aide.input_text.split("|||").collect();
                    let outputs: Vec<&str> = aide.command_output.split("|||").collect();
                    
                    let mut content = String::new();
                    content.push_str("All Text Entries:\n");
                    content.push_str("================\n\n");
                    
                    for (i, (input, output)) in inputs.iter().zip(outputs.iter()).enumerate() {
                        if !input.is_empty() && !output.is_empty() {
                            content.push_str(&format!("{}. {}\n", i + 1, input));
                            content.push_str(&format!("   └─ {}\n\n", output));
                        }
                    }
                    
                    content.push_str("Controls:\n• Enter: Edit content\n• e: Quick edit\n• r: Refresh\n• q: Quit");
                    (title, content)
                }
            }
            "file" => {
                let title = format!("File Entries - {}", aide.name);
                if aide.command_output.is_empty() {
                    let content = format!("No file entries available\n\nTo add files:\n• aide add {} \"file_name\"\n\nControls:\n• Enter: Edit file\n• e: Quick edit\n• r: Refresh\n• q: Quit", aide.name);
                    (title, content)
                } else {
                    // Split concatenated entries
                    let inputs: Vec<&str> = aide.input_text.split("|||").collect();
                    let outputs: Vec<&str> = aide.command_output.split("|||").collect();
                    
                    let mut content = String::new();
                    content.push_str("All Files:\n");
                    content.push_str("=========\n\n");
                    
                    for (i, (input, output)) in inputs.iter().zip(outputs.iter()).enumerate() {
                        if !input.is_empty() {
                            content.push_str(&format!("{}. 📄 {}\n", i + 1, input));
                            if !output.is_empty() {
                                // Show preview of file content (first 100 chars)
                                let preview = if output.len() > 100 {
                                    format!("{}...", &output[..100])
                                } else {
                                    output.to_string()
                                };
                                content.push_str(&format!("   Preview: {}\n", preview));
                            }
                            content.push_str("\n");
                        }
                    }
                    
                    content.push_str("Controls:\n• Enter: Edit file\n• e: Quick edit\n• r: Refresh\n• q: Quit");
                    (title, content)
                }
            }
            _ => {
                let title = format!("Unknown Type - {}", aide.name);
                let content = format!("Type: {}\nInput: {}\nOutput: {}\n\nControls:\n• Enter: Edit\n• r: Refresh\n• q: Quit",
                               aide.aide_type, aide.input_text, aide.command_output);
                (title, content)
            }
        };

        let content_paragraph = Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL).title(title))
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(Color::White));

        f.render_widget(content_paragraph, chunks[1]);
    } else {
        let info_text = "No aide selected\n\nControls:\n• ↑/↓: Navigate\n• Enter: Edit aide\n• e: Quick edit\n• r: Refresh\n• q: Quit";
        
        let info_paragraph = Paragraph::new(info_text)
            .block(Block::default().borders(Borders::ALL).title("Aide Content"))
            .style(Style::default().fg(Color::White));

        f.render_widget(info_paragraph, chunks[1]);
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let db = Database::new()?;
    
    match cli.command {
        Some(Commands::Create { name, aide_type }) => {
            db.create_aide(&name, &aide_type)?;
        }
        Some(Commands::Add { name, input_text, command_output }) => {
            db.add_data(&name, &input_text, command_output)?;
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
