use anyhow::Result;
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
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::database::Database;
use crate::editor::TextEditor;
use crate::models::{TaskItem, AideItem, ConfigItem, PopupMode, EditorCallback};

pub struct App {
    db: Database,
    pub current_tab: usize,
    pub tasks: Vec<TaskItem>,
    pub aides: Vec<AideItem>,
    pub configs: Vec<ConfigItem>,
    pub task_list_state: ListState,
    pub aide_list_state: ListState,
    pub config_list_state: ListState,
    pub should_quit: bool,
    // UI state
    pub show_priority_popup: bool,
    pub show_status_popup: bool,
    pub show_aide_popup: bool,
    pub show_config_popup: bool,
    pub input_buffer: String,
    pub config_value_buffer: String,
    pub popup_mode: PopupMode,
    // Text editor
    pub text_editor: Option<TextEditor>,
    pub editor_save_callback: Option<EditorCallback>,
}

impl App {
    pub fn new(db: Database) -> Result<Self> {
        let mut app = App {
            db,
            current_tab: 0,
            tasks: Vec::new(),
            aides: Vec::new(),
            configs: Vec::new(),
            task_list_state: ListState::default(),
            aide_list_state: ListState::default(),
            config_list_state: ListState::default(),
            should_quit: false,
            show_priority_popup: false,
            show_status_popup: false,
            show_aide_popup: false,
            show_config_popup: false,
            input_buffer: String::new(),
            config_value_buffer: String::new(),
            popup_mode: PopupMode::None,
            text_editor: None,
            editor_save_callback: None,
        };
        app.refresh_data()?;
        Ok(app)
    }

    pub fn refresh_data(&mut self) -> Result<()> {
        self.tasks = self.db.get_all_tasks()?;
        self.aides = self.db.get_all_aides()?;
        self.configs = self.db.get_all_configs()?;
        Ok(())
    }

    pub fn next_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % 3;
    }

    pub fn previous_tab(&mut self) {
        self.current_tab = if self.current_tab == 0 { 2 } else { self.current_tab - 1 };
    }

    pub fn next_item(&mut self) {
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
            2 => {
                let i = match self.config_list_state.selected() {
                    Some(i) => {
                        if i >= self.configs.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.config_list_state.select(Some(i));
            }
            _ => {}
        }
    }

    pub fn previous_item(&mut self) {
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
            2 => {
                let i = match self.config_list_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            self.configs.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.config_list_state.select(Some(i));
            }
            _ => {}
        }
    }

    pub fn show_priority_popup(&mut self) {
        self.show_priority_popup = true;
        self.popup_mode = PopupMode::TaskPriority;
        self.input_buffer.clear();
    }

    pub fn show_status_popup(&mut self) {
        self.show_status_popup = true;
        self.popup_mode = PopupMode::TaskStatus;
        self.input_buffer.clear();
    }

    pub fn show_aide_popup(&mut self) {
        self.show_aide_popup = true;
        self.popup_mode = PopupMode::AideEdit;
        self.input_buffer.clear();
    }

    pub fn show_config_popup(&mut self) {
        if let Some(i) = self.config_list_state.selected() {
            if let Some(config) = self.configs.get(i) {
                self.show_config_popup = true;
                self.popup_mode = PopupMode::ConfigEdit;
                self.input_buffer.clear();
                // Initialize with current config value
                self.config_value_buffer = config.value.clone();
            }
        }
    }

    pub fn close_popup(&mut self) {
        self.show_priority_popup = false;
        self.show_status_popup = false;
        self.show_aide_popup = false;
        self.show_config_popup = false;
        self.popup_mode = PopupMode::None;
        self.input_buffer.clear();
        self.config_value_buffer.clear();
    }

    pub fn handle_popup_input(&mut self, c: char) -> Result<()> {
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
            PopupMode::ConfigEdit => {
                if c == '\n' || c == '\r' {
                    self.handle_config_edit()?;
                } else if c.is_ascii() && c != '\x08' {
                    self.config_value_buffer.push(c);
                }
            }
            PopupMode::TextEditor => {
                // Text editor input is handled separately in handle_text_editor_input
            }
            PopupMode::None => {}
        }
        Ok(())
    }

    pub fn handle_backspace(&mut self) {
        if matches!(self.popup_mode, PopupMode::AideEdit) {
            self.input_buffer.pop();
        } else if matches!(self.popup_mode, PopupMode::ConfigEdit) {
            self.config_value_buffer.pop();
        }
    }

    pub fn open_text_editor(&mut self, title: String, content: String, callback: EditorCallback) {
        self.text_editor = Some(TextEditor::new(title, content));
        self.editor_save_callback = Some(callback);
        self.popup_mode = PopupMode::TextEditor;
    }

    pub fn close_text_editor(&mut self, save: bool) -> Result<()> {
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

    pub fn handle_text_editor_input(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<()> {
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
                KeyCode::PageUp => {
                    editor.page_up(20); // Use default visible height for now
                }
                KeyCode::PageDown => {
                    editor.page_down(20); // Use default visible height for now
                }
                KeyCode::Home => {
                    editor.move_to_start_of_line();
                }
                KeyCode::End => {
                    editor.move_to_end_of_line();
                }
                KeyCode::Esc => {
                    self.close_text_editor(false)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn edit_selected_task(&mut self) -> Result<()> {
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

    pub fn edit_selected_aide(&mut self) -> Result<()> {
        if let Some(i) = self.aide_list_state.selected() {
            if let Some(aide) = self.aides.get(i) {
                // All aides are now files, so read the actual file content
                let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                let aide_dir = PathBuf::from(&home_dir).join(".aide");
                let file_path = aide_dir.join(format!("{}.txt", aide.name));
                
                let formatted_content = if file_path.exists() {
                    fs::read_to_string(&file_path).unwrap_or_else(|_| {
                        format!("# {}\n\nCreated: {}\n\n", 
                               aide.name, 
                               chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"))
                    })
                } else {
                    format!("# {}\n\nCreated: {}\n\n", 
                           aide.name, 
                           chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"))
                };
                
                self.open_text_editor(
                    format!("Edit Aide: {}", aide.name),
                    formatted_content,
                    EditorCallback::SaveAide(aide.name.clone())
                );
            }
        }
        Ok(())
    }

    pub fn handle_aide_edit(&mut self) -> Result<()> {
        if let Some(i) = self.aide_list_state.selected() {
            if let Some(aide) = self.aides.get(i) {
                self.db.update_aide_content(&aide.name, &self.input_buffer)?;
                self.refresh_data()?;
            }
        }
        self.close_popup();
        Ok(())
    }

    pub fn handle_config_edit(&mut self) -> Result<()> {
        if let Some(i) = self.config_list_state.selected() {
            if let Some(config) = self.configs.get(i) {
                self.db.update_config_value(&config.key_name, &self.config_value_buffer)?;
                self.refresh_data()?;
            }
        }
        self.close_popup();
        Ok(())
    }
}

pub fn run_tui(db: Database) -> Result<()> {
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
                            } else if app.current_tab == 2 {
                                app.show_config_popup();
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
                        KeyCode::Char('c') => {
                            if app.current_tab == 2 {
                                app.show_config_popup();
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

    let titles: Vec<Line> = ["Tasks", "Aides", "Configs"]
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
        2 => render_configs(f, app, chunks[1]),
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

    if app.show_config_popup {
        let popup_area = centered_rect(50, 20, f.area());
        let block = Block::default()
            .title("Edit Config Value")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray));
        let content = Paragraph::new(format!("Enter new value for config:\n\n{}\n\nPress ENTER to save\nPress ESC to cancel", app.config_value_buffer))
            .block(block)
            .alignment(ratatui::layout::Alignment::Center)
            .style(Style::default().fg(Color::White));
        
        f.render_widget(content, popup_area);
    }

    // Render text editor with complete background coverage
    if let Some(editor) = &mut app.text_editor {
        // Create a completely opaque full-screen background using Clear
        f.render_widget(
            Block::default().style(Style::default().bg(Color::Black)),
            f.area()
        );
        
        // Fill the entire screen with black background characters
        let full_bg_lines: Vec<Line> = (0..f.area().height)
            .map(|_| Line::from(Span::styled(" ".repeat(f.area().width as usize), Style::default().bg(Color::Black))))
            .collect();
        
        let full_bg = Paragraph::new(full_bg_lines)
            .style(Style::default().bg(Color::Black));
        f.render_widget(full_bg, f.area());
        
        let editor_area = centered_rect(90, 80, f.area());
        
        // Create the main editor block
        let block = Block::default()
            .title(format!("{} - Ctrl+S: Save | Ctrl+Q: Quit | ESC: Cancel | PgUp/PgDn: Scroll | Home/End: Line Nav", &editor.title))
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray).fg(Color::White));
        
        let inner_area = block.inner(editor_area);
        f.render_widget(block, editor_area);
        
        // Calculate visible lines and update scroll in one go
        let visible_height = inner_area.height as usize;
        editor.adjust_scroll_with_height(visible_height);
        
        let start_line = editor.scroll_offset;
        let end_line = (start_line + visible_height).min(editor.content.len());
        
        // Create content lines with explicit background
        let mut content_lines: Vec<Line> = Vec::new();
        
        // Add content lines
        for i in start_line..end_line {
            if i < editor.content.len() {
                let line = &editor.content[i];
                let is_cursor_line = i == editor.cursor_row;
                
                if is_cursor_line {
                    let mut line_spans = Vec::new();
                    let line_chars: Vec<char> = line.chars().collect();
                    
                    // Before cursor
                    if editor.cursor_col > 0 && editor.cursor_col <= line_chars.len() {
                        let before_cursor: String = line_chars[..editor.cursor_col].iter().collect();
                        line_spans.push(Span::styled(before_cursor, Style::default().fg(Color::White).bg(Color::Black)));
                    }
                    
                    // Cursor
                    let cursor_char = if editor.cursor_col < line_chars.len() {
                        line_chars[editor.cursor_col].to_string()
                    } else {
                        " ".to_string()
                    };
                    line_spans.push(Span::styled(cursor_char, Style::default().bg(Color::Cyan).fg(Color::Black)));
                    
                    // After cursor
                    if editor.cursor_col < line_chars.len() {
                        let after_cursor: String = line_chars[editor.cursor_col + 1..].iter().collect();
                        if !after_cursor.is_empty() {
                            line_spans.push(Span::styled(after_cursor, Style::default().fg(Color::White).bg(Color::Black)));
                        }
                    }
                    
                    // Fill the rest of the line with spaces to ensure full width coverage
                    let current_width: usize = line_chars.len();
                    if current_width < inner_area.width as usize {
                        let padding = " ".repeat(inner_area.width as usize - current_width);
                        line_spans.push(Span::styled(padding, Style::default().bg(Color::Black)));
                    }
                    
                    content_lines.push(Line::from(line_spans));
                } else {
                    // Regular line - pad to full width
                    let padded_line = if line.len() < inner_area.width as usize {
                        format!("{}{}", line, " ".repeat(inner_area.width as usize - line.len()))
                    } else {
                        line.clone()
                    };
                    content_lines.push(Line::from(Span::styled(padded_line, Style::default().fg(Color::White).bg(Color::Black))));
                }
            }
        }
        
        // Fill remaining space with full-width empty lines
        while content_lines.len() < visible_height {
            content_lines.push(Line::from(Span::styled(" ".repeat(inner_area.width as usize), Style::default().bg(Color::Black))));
        }
        
        // Render the editor content
        let editor_content = Paragraph::new(content_lines)
            .style(Style::default().fg(Color::White).bg(Color::Black));
        
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
            ListItem::new(vec![Line::from(vec![
                Span::styled(
                    format!("{} ", aide.name),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    "[file]",
                    Style::default().fg(Color::Green),
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
        let title = format!("File Aide - {}", aide.name);
        let content = if aide.command_output.is_empty() {
            format!("No entries available\n\nTo add content:\n• aide add {} \"your content\"\n• aide add {} -p /path/to/file\n\nControls:\n• Enter: Edit file\n• e: Quick edit\n• r: Refresh\n• q: Quit", aide.name, aide.name)
        } else {
            // Split concatenated entries and show preview
            let inputs: Vec<&str> = aide.input_text.split("|||").collect();
            let outputs: Vec<&str> = aide.command_output.split("|||").collect();
            
            let mut content = String::new();
            content.push_str("File Entries:\n");
            content.push_str("=============\n\n");
            
            for (i, (input, output)) in inputs.iter().zip(outputs.iter()).enumerate() {
                if !input.is_empty() {
                    content.push_str(&format!("{}. {}\n", i + 1, input));
                    if !output.is_empty() {
                        // Show preview of content (first 100 chars)
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
            content
        };

        let content_paragraph = Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL).title(title))
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(Color::White));

        f.render_widget(content_paragraph, chunks[1]);
    } else {
        let info_text = "No aide selected\n\nControls:\n• ↑/↓: Navigate\n• Enter: Edit aide file\n• e: Quick edit\n• r: Refresh\n• q: Quit";
        
        let info_paragraph = Paragraph::new(info_text)
            .block(Block::default().borders(Borders::ALL).title("Aide Content"))
            .style(Style::default().fg(Color::White));

        f.render_widget(info_paragraph, chunks[1]);
    }
}

fn render_configs(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    let configs: Vec<ListItem> = app
        .configs
        .iter()
        .map(|config| {
            ListItem::new(vec![Line::from(vec![
                Span::styled(
                    format!("{} ", config.key_name),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("={}", config.value),
                    Style::default().fg(Color::Cyan),
                ),
            ])])
        })
        .collect();

    let configs_list = List::new(configs)
        .block(Block::default().borders(Borders::ALL).title("Configs"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(configs_list, chunks[0], &mut app.config_list_state);

    let selected_config = app.config_list_state.selected().and_then(|i| app.configs.get(i));
    let info_text = if let Some(config) = selected_config {
        format!(
            "Config: {}\nValue: {}\n\nControls:\n• Enter: Edit config value\n• r: Refresh\n• q: Quit",
            config.key_name, config.value
        )
    } else {
        "No config selected\n\nControls:\n• ↑/↓: Navigate\n• Enter: Edit config value\n• r: Refresh\n• q: Quit".to_string()
    };

    let info_paragraph = Paragraph::new(info_text)
        .block(Block::default().borders(Borders::ALL).title("Config Info"))
        .style(Style::default().fg(Color::White));

    f.render_widget(info_paragraph, chunks[1]);
}