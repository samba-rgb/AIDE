#[derive(Debug, Clone)]
pub struct TextEditor {
    pub content: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub scroll_offset: usize,
    pub title: String,
    pub is_dirty: bool,
}

impl TextEditor {
    pub fn new(title: String, content: String) -> Self {
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
    
    pub fn insert_char(&mut self, c: char) {
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
    
    pub fn insert_newline(&mut self) {
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
    
    pub fn delete_char(&mut self) {
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
    
    pub fn move_cursor_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.content[self.cursor_row].len();
        }
    }
    
    pub fn move_cursor_right(&mut self) {
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
    
    pub fn move_cursor_up(&mut self) {
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            let line_len = self.content[self.cursor_row].len();
            if self.cursor_col > line_len {
                self.cursor_col = line_len;
            }
            self.adjust_scroll();
        }
    }
    
    pub fn move_cursor_down(&mut self) {
        if self.cursor_row < self.content.len() - 1 {
            self.cursor_row += 1;
            let line_len = self.content[self.cursor_row].len();
            if self.cursor_col > line_len {
                self.cursor_col = line_len;
            }
            self.adjust_scroll();
        }
    }
    
    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }
    
    pub fn scroll_down(&mut self, visible_height: usize) {
        let max_scroll = if self.content.len() > visible_height {
            self.content.len() - visible_height
        } else {
            0
        };
        
        if self.scroll_offset < max_scroll {
            self.scroll_offset += 1;
        }
    }
    
    pub fn page_up(&mut self, visible_height: usize) {
        if self.cursor_row >= visible_height {
            self.cursor_row -= visible_height;
        } else {
            self.cursor_row = 0;
        }
        
        let line_len = self.content[self.cursor_row].len();
        if self.cursor_col > line_len {
            self.cursor_col = line_len;
        }
        self.adjust_scroll();
    }
    
    pub fn page_down(&mut self, visible_height: usize) {
        if self.cursor_row + visible_height < self.content.len() {
            self.cursor_row += visible_height;
        } else {
            self.cursor_row = self.content.len() - 1;
        }
        
        let line_len = self.content[self.cursor_row].len();
        if self.cursor_col > line_len {
            self.cursor_col = line_len;
        }
        self.adjust_scroll();
    }
    
    pub fn move_to_start_of_line(&mut self) {
        self.cursor_col = 0;
    }
    
    pub fn move_to_end_of_line(&mut self) {
        if self.cursor_row < self.content.len() {
            self.cursor_col = self.content[self.cursor_row].len();
        }
    }
    
    // Adjust scroll to keep cursor in view
    fn adjust_scroll(&mut self) {
        // This will be called with visible_height from the UI
        // For now, we'll use a default of 20 lines
        let visible_height = 20;
        
        if self.cursor_row < self.scroll_offset {
            self.scroll_offset = self.cursor_row;
        } else if self.cursor_row >= self.scroll_offset + visible_height {
            self.scroll_offset = self.cursor_row - visible_height + 1;
        }
    }
    
    pub fn adjust_scroll_with_height(&mut self, visible_height: usize) {
        if self.cursor_row < self.scroll_offset {
            self.scroll_offset = self.cursor_row;
        } else if self.cursor_row >= self.scroll_offset + visible_height {
            self.scroll_offset = self.cursor_row - visible_height + 1;
        }
    }
    
    pub fn get_content(&self) -> String {
        self.content.join("\n")
    }
}