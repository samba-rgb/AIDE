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

### Adding to PATH (macOS)

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

#### Option 3: Add project directory to PATH
Add the following line to your shell configuration file:

**For zsh (default on macOS Catalina+):**
```bash
echo 'export PATH="$PATH:$(pwd)/target/release"' >> ~/.zshrc
source ~/.zshrc
```

**For bash:**
```bash
echo 'export PATH="$PATH:$(pwd)/target/release"' >> ~/.bash_profile
source ~/.bash_profile
```

**For fish shell:**
```bash
echo 'set -gx PATH $PATH $(pwd)/target/release' >> ~/.config/fish/config.fish
```

#### Option 4: Install to ~/.local/bin
```bash
# Create the directory if it doesn't exist
mkdir -p ~/.local/bin

# Copy the binary
cp target/release/aide ~/.local/bin/

# Add to PATH if not already there
echo 'export PATH="$PATH:$HOME/.local/bin"' >> ~/.zshrc
source ~/.zshrc
```

#### Option 5: Using Homebrew (for distribution)
If you plan to distribute via Homebrew later:
```bash
# Create a formula (advanced users)
# This would be for when you create a Homebrew tap
brew install your-username/tap/aide
```

### Verification
After installation, verify aide is working:
```bash
# Check if aide is in PATH
which aide

# Test the tool
aide --help

# Check version info
aide --version  # (if you add version info later)
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
source ~/.zshrc  # or ~/.bash_profile

# Or restart your terminal

# Check if the directory is in PATH
echo $PATH
```

**macOS security warning:**
If macOS shows "aide cannot be opened because it is from an unidentified developer":
```bash
# Remove quarantine attribute
xattr -d com.apple.quarantine target/release/aide

# Or allow in System Preferences > Security & Privacy
```

## Quick Start

### 1. Create Your First Aide
```bash
# Create a text-based aide for storing commands
aide create command text

# Create a file-based aide for work logs
aide create work_log file
```

### 2. Add Content
```bash
# Add a command to text aide
aide add command "ssh to server" "ssh user@server.com"

# Add a file to file aide (creates actual file)
aide add work_log "daily_standup" "Today's standup notes"
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
aide add <name> <input> [output]         # Add content to aide
aide aide-list                          # List all aides
aide search <text>                       # Fuzzy search content
aide command <text>                      # Search by aide name + content
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

## File Structure

Aide creates the following directory structure in your home directory:

```
~/.aide/
├── .aide.db              # SQLite database
├── tasks/                # Task log files
│   ├── task1.txt
│   └── task2.txt
└── work_log/            # File aide directories
    ├── file1.txt
    └── file2.txt
```

### Task Files
Located in `~/.aide/tasks/`, each task gets its own log file:
```
Task: implement_authentication
Status: in_progress
Priority: 2
Created: 2025-07-06 16:30:24

--- Task Log ---
[2025-07-06 17:29:17] Completed user login flow
[2025-07-06 18:15:32] Working on password validation
```

### File Aides
Each file aide creates a directory in `~/.aide/` containing actual files:
- `aide add work_log "meeting_notes" "content"` → `~/.aide/work_log/meeting_notes.txt`
- Files are real filesystem files you can access with any editor

## Database Schema

Aide uses SQLite with the following tables:

### `aides`
- `id`: Primary key
- `name`: Aide name (unique)
- `aide_type`: "text" or "file"

### `data`
- `id`: Primary key
- `aide_id`: Foreign key to aides
- `input_text`: Entry name/description
- `command_output`: Content/command

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
aide add command "check docker logs" "docker logs -f container_name"

# Create meeting notes
aide add work_log "team_meeting_2025_07_06" "Discussed Q3 roadmap and priorities"

# Update task status
aide task-status "api_development" "in_progress"

# Quick search
aide search "docker"
# Output: Found match in aide 'command': check docker logs
#         Output: docker logs -f container_name
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
- File aide directories: `~/.aide/{aide_name}/`

## Troubleshooting

### Common Issues

**Q: TUI doesn't display properly**
A: Ensure your terminal supports UTF-8 and has adequate size (minimum 80x24)

**Q: Files not being created**
A: Check write permissions in your home directory

**Q: Search not finding content**
A: Search uses fuzzy matching - try partial keywords

**Q: Task log updates not appearing**
A: Run `aide task-list` to verify task exists, or use TUI refresh (r key)

### Debug Mode
View raw data:
```bash
aide aide-list    # See all aides and entry counts
aide task-list    # See all tasks with metadata
```

## Contributing

Aide is built with:
- **Rust**: Core language
- **clap**: Command-line argument parsing
- **rusqlite**: SQLite database interface
- **ratatui**: Terminal user interface
- **crossterm**: Cross-platform terminal manipulation
- **fuzzy-matcher**: Fuzzy text searching

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