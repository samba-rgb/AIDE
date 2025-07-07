# Aide - Terminal-Based Knowledge Management Tool

A powerful, database-driven terminal tool for managing tasks, commands, and file-based knowledge with an intuitive TUI interface.

## Features

### 📋 Task Management
- Create and manage tasks with priority levels (1-5) and status tracking
- Built-in task log files with timestamped entries
- Interactive priority and status updates
- Task log quick updates without opening editor

### 📝 Aide System
- **Text Aides**: Store commands, snippets, and quick references
- **File Aides**: Create and manage actual files on your filesystem
- Fuzzy search across all stored content
- Built-in text editor with full cursor navigation

### 🖥️ Terminal User Interface (TUI)
- Interactive interface built with ratatui
- Tabbed navigation between Tasks and Aides
- Real-time content preview in right panel
- Built-in text editor (no external dependencies)
- Popup dialogs for quick actions

## Installation

### Prerequisites
- Rust 1.70+ (for building from source)
- Terminal with UTF-8 support

### Build from Source
```bash
git clone https://github.com/samba-rgb/AIDE.git
cd AIDE
cargo build --release
```

The binary will be available at `target/release/aide`.

### Adding to PATH (Linux/macOS)

After building the release version, you have several options to make `aide` available system-wide:

#### Option 1: Copy to /usr/local/bin (Recommended)
```bash
# Copy the binary to a directory that's already in PATH
sudo cp target/release/aide /usr/local/bin/

# Make it executable (usually already set)
sudo chmod +x /usr/local/bin/aide

# Verify installation
aide --help
```

#### Option 2: Create a symlink to /usr/local/bin
```bash
# Create a symbolic link instead of copying
sudo ln -s $(pwd)/target/release/aide /usr/local/bin/aide

# Verify installation
aide --help
```

#### Option 3: Install to ~/.local/bin
```bash
# Create the directory if it doesn't exist
mkdir -p ~/.local/bin

# Copy the binary
cp target/release/aide ~/.local/bin/

# Add to PATH if not already there
echo 'export PATH="$PATH:$HOME/.local/bin"' >> ~/.bashrc
source ~/.bashrc
```

#### Option 4: Add project directory to PATH
Add the following line to your shell configuration file:

**For bash:**
```bash
echo 'export PATH="$PATH:$(pwd)/target/release"' >> ~/.bashrc
source ~/.bashrc
```

**For zsh:**
```bash
echo 'export PATH="$PATH:$(pwd)/target/release"' >> ~/.zshrc
source ~/.zshrc
```

**For fish shell:**
```bash
echo 'set -gx PATH $PATH $(pwd)/target/release' >> ~/.config/fish/config.fish
```

### Verification
After installation, verify aide is working:
```bash
# Check if aide is in PATH
which aide

# Test the tool
aide --help

# List available commands
aide --help
```

### Uninstallation
To remove aide from your system:

**If installed to /usr/local/bin:**
```bash
sudo rm /usr/local/bin/aide
```

**If using symlink:**
```bash
sudo unlink /usr/local/bin/aide
```

**If added to PATH via shell config:**
Remove the export line from your shell configuration file and restart terminal.

### Troubleshooting Installation

**Permission denied errors:**
```bash
# Make sure the binary has execute permissions
chmod +x target/release/aide
```

**Command not found after installation:**
```bash
# Reload your shell configuration
source ~/.bashrc  # or ~/.zshrc

# Or restart your terminal

# Check if the directory is in PATH
echo $PATH
```

## Quick Start

### 1. Create Your First Aide
```bash
# Create a text-based aide for storing commands
aide create commands text

# Create a file-based aide for work logs
aide create work_log file
```

### 2. Add Content
```bash
# Add content to text aide (stores as timestamped entry)
aide add commands "ssh user@server.com"

# Add content to file aide (appends to ~/.aide/work_log.txt)
aide add work_log "Today's standup notes and meeting updates"
```

### 3. Create Tasks
```bash
# Create a new task (opens editor)
aide task "implement_authentication"

# Add quick log entry to task
aide task-log-update "implement_authentication" "Completed user login flow"
```

### 4. Launch TUI
```bash
# Launch interactive interface
aide
# or explicitly
aide tui
```

## Command Reference

### Task Commands
```bash
# Task management
aide task <task_name>                    # Create/edit task
aide task-list                          # List all tasks
aide task-edit <task_name>               # Edit task log file
aide task-status <task_name> <status>    # Update status (created/in_progress/completed)
aide task-priority <task_name> <1-5>     # Update priority (1=highest, 5=lowest)
aide task-log-update <task_name> <text>  # Add timestamped log entry
```

### Aide Commands
```bash
# Aide management
aide create <name> <type>                # Create aide (type: text|file)
aide add <name> <content>                # Add content to aide
aide add <name> -p <file_path>           # Add content from file to aide
aide write <name>                        # Open file aide in vim/vi/nano editor
aide aide-list                          # List all aides
aide search <text>                       # Fuzzy search content
aide command <text>                      # Search by aide name + content
```

### System Commands
```bash
aide reset                               # Reset all data (WARNING: Deletes all tasks and aides)
aide clear                               # Clear all data (same as reset)
```

### TUI Commands
```bash
aide tui                                 # Launch TUI interface
aide                                     # Default: launch TUI
```

## TUI Navigation

### Main Interface
- **Tab/Shift+Tab**: Switch between Tasks and Aides tabs
- **↑/↓**: Navigate items in current tab
- **Enter**: Edit selected item
- **r**: Refresh data
- **q**: Quit

### Tasks Tab
- **p**: Change priority (popup with options 1-5)
- **s**: Change status (popup with options)
- **Enter**: Edit task log file in built-in editor

### Aides Tab
- **e**: Quick edit aide content
- **Enter**: Full edit in built-in editor

### Built-in Text Editor
- **Ctrl+S**: Save and close
- **Ctrl+Q**: Quit without saving
- **ESC**: Cancel editing
- **Arrow keys**: Navigate cursor
- **Enter**: New line
- **Backspace**: Delete character

## How Aides Work

### Text Aides
Text aides store content as timestamped entries in the database:
- Each `aide add` command creates a new timestamped entry
- Perfect for storing commands, snippets, and quick references
- Search through all entries with `aide search`

### File Aides
File aides create actual files on your filesystem:
- Content is stored in `~/.aide/{aide_name}.txt`
- Each `aide add` command appends timestamped content to the file
- Files can be edited with any external editor
- Perfect for logs, notes, and documentation

## File Structure

Aide creates the following directory structure in your home directory:

```
~/.aide/
├── .aide.db              # SQLite database
├── tasks/                # Task log files
│   ├── task1.txt
│   └── task2.txt
├── work_log.txt          # File aide content
└── commands.txt          # Another file aide
```

### Task Files
Located in `~/.aide/tasks/`, each task gets its own log file:
```
Task: implement_authentication
Status: in_progress
Priority: 2
Created: 2025-07-07 10:30:24

--- Task Log ---
[2025-07-07 11:29:17] Completed user login flow
[2025-07-07 12:15:32] Working on password validation
```

### File Aide Files
File aides create single files in `~/.aide/`:
```
Aide: work_log
Type: file
Created: 2025-07-07 10:30:24

--- Entries ---

[2025-07-07 11:30:00] Today's standup notes and meeting updates
[2025-07-07 14:15:22] Completed sprint planning session
```

## Database Schema

Aide uses SQLite with the following tables:

### `aides`
- `id`: Primary key
- `name`: Aide name (unique)
- `aide_type`: "text" or "file"

### `data`
- `id`: Primary key
- `aide_id`: Foreign key to aides
- `input_text`: Entry content
- `command_output`: Timestamped content

### `tasks`
- `id`: Primary key
- `name`: Task name (unique)
- `priority`: 1-5 (1=highest)
- `status`: "created", "in_progress", "completed"
- `task_log_file_path`: Path to log file
- `created_at`: Timestamp

## Examples

### Daily Workflow
```bash
# Morning standup
aide task-log-update "current_sprint" "Daily standup: working on API endpoints"

# Store useful commands
aide add commands "docker logs -f container_name"

# Create meeting notes
aide add work_log "Team meeting 2025-07-07: Discussed Q3 roadmap and priorities"

# Update task status
aide task-status "api_development" "in_progress"

# Quick search
aide search "docker"
# Output: Found match in aide 'commands': docker logs -f container_name
#         Output: [2025-07-07 10:30:24] docker logs -f container_name
```

### Text Aide Usage
```bash
# Create and populate a commands aide
aide create commands text
aide add commands "ssh user@production-server.com"
aide add commands "docker ps -a"
aide add commands "tail -f /var/log/nginx/access.log"

# Search for specific commands
aide search "ssh"
aide search "docker"
```

### File Aide Usage
```bash
# Create and populate a meeting notes aide
aide create meeting_notes file
aide add meeting_notes "Weekly team sync - discussed new features"
aide add meeting_notes "Sprint planning - estimated 45 story points"

# The content is stored in ~/.aide/meeting_notes.txt
# You can also edit it directly: vim ~/.aide/meeting_notes.txt
```

### TUI Workflow
1. Launch: `aide`
2. Navigate to Tasks tab
3. Select task and press `p` to change priority
4. Switch to Aides tab with Tab
5. Select aide and press Enter to edit content
6. Use built-in editor to modify content
7. Save with Ctrl+S

## Configuration

### Default Aides
Aide automatically creates:
- `task_log` (file type): For task-related files

### File Locations
- Database: `~/.aide/.aide.db`
- Task files: `~/.aide/tasks/`
- File aide content: `~/.aide/{aide_name}.txt`

## Troubleshooting

### Common Issues

**Q: TUI doesn't display properly**
A: Ensure your terminal supports UTF-8 and has adequate size (minimum 80x24)

**Q: Files not being created**
A: Check write permissions in your home directory

**Q: Search not finding content**
A: Search uses fuzzy matching - try partial keywords or different terms

**Q: Task log updates not appearing**
A: Run `aide task-list` to verify task exists, or use TUI refresh (r key)

**Q: "Aide not found" errors**
A: Aide uses fuzzy matching - if you get a suggestion, type 'y' to accept it

### Debug Mode
View raw data:
```bash
aide aide-list    # See all aides and entry counts
aide task-list    # See all tasks with metadata
```

## Advanced Features

### Fuzzy Matching
Aide uses TF-IDF based fuzzy matching for task and aide names:
- Typos are automatically corrected with user confirmation
- Similar names are suggested when exact matches aren't found
- Threshold-based matching ensures relevant suggestions

### Timestamped Entries
All content added to aides is automatically timestamped:
- Helps track when entries were created
- Useful for logs and historical data
- Searchable by content and timestamp

## Contributing

Aide is built with:
- **Rust**: Core language
- **clap**: Command-line argument parsing
- **rusqlite**: SQLite database interface
- **ratatui**: Terminal user interface
- **crossterm**: Cross-platform terminal manipulation
- **fuzzy-matcher**: Fuzzy text searching
- **chrono**: Date and time handling

## License

[Add your license here]

## Changelog

### Version 0.1.0 (Current)
- Initial release
- Task management with priority and status
- Text and file aide types
- Built-in TUI with ratatui
- Fuzzy search functionality
- File creation for file-type aides
- Task log updates with timestamps
- Built-in text editor
- TF-IDF based fuzzy matching
- Reset/clear functionality