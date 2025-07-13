#[derive(Debug, Clone)]
pub struct TaskItem {
    pub name: String,
    pub priority: i32,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct AideItem {
    pub name: String,
    pub input_text: String,
    pub command_output: String,
}

#[derive(Debug, Clone)]
pub struct ConfigItem {
    pub key_name: String,
    pub value: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PopupMode {
    None,
    TaskPriority,
    TaskStatus,
    AideEdit,
    ConfigEdit,
    TextEditor,
}

#[derive(Debug, Clone)]
pub enum EditorCallback {
    SaveTask(String),
    SaveAide(String),
}