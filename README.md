# Aide - Terminal-Based Knowledge Management Tool

A powerful, database-driven terminal tool for managing tasks, commands, configurations, and file-based knowledge with an intuitive TUI interface and intelligent fuzzy matching.

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

### ⚙️ Configuration Management
- Store and manage application configurations with key-value pairs
- TF-IDF powered fuzzy matching for config keys
- Interactive config editing through TUI
- Automatic config indexing with incremental updates

### 🖥️ Terminal User Interface (TUI)
- Interactive interface built with ratatui
- **Three-tab navigation**: Tasks, Aides, and Configs
- Real-time content preview in right panel
- Built-in text editor (no external dependencies)
- Popup dialogs for quick actions

### 🧠 Intelligent Fuzzy Matching
- **TF-IDF Algorithm**: Advanced text similarity scoring
- **Incremental Indexing**: Efficient updates without full rebuilds
- **Smart Suggestions**: Context-aware typo correction
- **Universal Search**: Works across tasks, aides, and configs

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
aide create <name>                       # Create aide
aide add <name> <content>                # Add content to aide
aide add <name> -p <file_path>           # Add content from file to aide
aide write <name>                        # Open aide in editor
aide aide-list                          # List all aides
aide search <text>                       # Fuzzy search content
```

### Configuration Commands
```bash
# Configuration management
aide set <key> <value>                   # Set configuration value
aide get <key>                           # Get configuration value
aide config-list                        # List all configurations
aide config-delete <key>                # Delete configuration key
```

### System Commands
```bash
aide reset                               # Reset all data (WARNING: Deletes everything)
aide clear                               # Clear all data (same as reset)
```

### TUI Commands
```bash
aide tui                                 # Launch TUI interface
aide                                     # Default: launch TUI
```

## TUI Navigation

### Main Interface
- **Tab/Shift+Tab**: Switch between Tasks, Aides, and Configs tabs
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

### Configs Tab
- **Enter**: Edit config value (popup editor)
- **c**: Quick edit config value
- **r**: Refresh config list

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

## Database Schema

Aide uses SQLite with the following tables:

### `aides`
- `id`: Primary key
- `name`: Aide name (unique)

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

### `config_data`
- `id`: Primary key
- `key_name`: Configuration key (unique)
- `value`: Configuration value
- `description`: Optional description
- `created_at`: Creation timestamp
- `updated_at`: Last update timestamp

## Examples

### Configuration Management
```bash
# Set application configurations
aide set database_url "sqlite:///app.db"
aide set api_endpoint "https://api.example.com"
aide set debug_mode "true"

# Get configuration values (with fuzzy matching)
aide get db_url          # Suggests "database_url"
aide get api             # Suggests "api_endpoint"

# List all configurations
aide config-list

# Delete configuration
aide config-delete debug_mode
```

### Daily Workflow with All Features
```bash
# Morning standup
aide task-log-update "current_sprint" "Daily standup: working on API endpoints"

# Store useful commands
aide add commands "docker logs -f container_name"

# Set project configurations
aide set project_name "awesome_app"
aide set git_remote "origin"

# Create meeting notes
aide add work_log "Team meeting 2025-07-09: Discussed Q3 roadmap and priorities"

# Update task status
aide task-status "api_development" "in_progress"

# Quick search across everything
aide search "docker"
aide get project    # Fuzzy matches "project_name"
```

## Advanced Features

### TF-IDF Fuzzy Matching System
Aide implements a sophisticated fuzzy matching system:

#### **Algorithm Features**
- **TF-IDF Scoring**: Term Frequency-Inverse Document Frequency for semantic similarity
- **String Similarity**: Character-based matching for typos and abbreviations
- **Combined Scoring**: Weighted average of TF-IDF (30%) and string similarity (70%)
- **Threshold-based Matching**: Configurable similarity threshold (default: 0.3)

#### **Incremental Indexing**
- **O(1) Additions**: New items added without rebuilding entire index
- **Smart Vocabulary**: Dynamic vocabulary expansion for new terms
- **IDF Recalculation**: Efficient updates only when necessary
- **Memory Efficient**: Existing vectors remain untouched during updates

#### **Universal Application**
- **Tasks**: Fuzzy matching for task names
- **Aides**: Intelligent aide name resolution
- **Configs**: Smart config key suggestions
- **Cross-Entity**: Consistent behavior across all data types

#### **User Experience**
```bash
# Typo correction
aide get databse_url
# Output: 'databse_url' not found. Did you mean 'database_url'? (y/n):

# Partial matching
aide task "auth"
# Output: 'auth' not found. Did you mean 'implement_authentication'? (y/n):

# Abbreviation support
aide add cmds "new command"
# Output: 'cmds' not found. Did you mean 'commands'? (y/n):
```

### Performance Optimizations
- **Incremental Updates**: No full index rebuilds on insertions
- **Lazy Loading**: TF-IDF indexes built on demand
- **Memory Caching**: In-memory indexes for fast lookups
- **Database Efficiency**: SQLite with proper indexing

## Configuration

### Default Data
Aide automatically creates:
- `task_log` aide: For task-related files
- Empty TF-IDF indexes for tasks, aides, and configs

### File Locations
- Database: `~/.aide/.aide.db`
- Task files: `~/.aide/tasks/`
- Aide content: `~/.aide/{aide_name}.txt`

### TF-IDF Settings
- **Fuzzy Match Threshold**: 0.3 (30% similarity required)
- **String Weight**: 70% (character-based similarity)
- **TF-IDF Weight**: 30% (semantic similarity)
- **Vocabulary Growth**: Dynamic expansion

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

## LLM Model Configuration

Aide uses an LLM (Large Language Model) for command generation. You can configure the model and endpoint using environment variables:

- `OLLAMA_MODEL_NAME`: Set the model name (default: `qwen2.5-coder:0.5b`)
- `OLLAMA_BASE_URL`: Set the Ollama API base URL (default: `http://localhost:11434`)

### Example: Change Model

```bash
export OLLAMA_MODEL_NAME="your-model-name"
export OLLAMA_BASE_URL="http://localhost:11434"
./aide ask "your question"
```

If these variables are not set, Aide will use the default values.

## LLM Model Environment Variables

Aide uses environment variables to configure the LLM model for command generation. You can change these variables in your terminal before running aide:

```bash
export OLLAMA_MODEL_NAME="your-model-name"
export OLLAMA_BASE_URL="http://localhost:11434"
./aide ask "your question"
```

- `OLLAMA_MODEL_NAME`: Sets the model name (default: `qwen2.5-coder:0.5b`)
- `OLLAMA_BASE_URL`: Sets the Ollama API base URL (default: `http://localhost:11434`)

If these variables are not set, Aide will use the default values. This allows you to easily switch models or endpoints for different use cases.

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

### Version 0.2.0 (Current)
- **NEW**: Configuration management system with `set`, `get`, `config-list`, `config-delete` commands
- **NEW**: Third "Configs" tab in TUI interface
- **NEW**: TF-IDF powered fuzzy matching for all entity types
- **NEW**: Incremental indexing system for performance
- **IMPROVED**: Universal fuzzy matching across tasks, aides, and configs
- **IMPROVED**: Better typo correction and suggestion system
- **ENHANCED**: More efficient database operations

### Version 0.1.0
- Initial release
- Task management with priority and status
- Text and file aide types
- Built-in TUI with ratatui
- Fuzzy search functionality
- File creation for file-type aides
- Task log updates with timestamps
- Built-in text editor
- Reset/clear functionality

## Build Optimization Tips

To optimize your Rust build for performance and faster compilation:

1. **Release Mode**: Always build with release mode for production binaries:
   ```bash
   cargo build --release
   ```

2. **Enable LTO (Link Time Optimization)**: Add this to your `Cargo.toml`:
   ```toml
   [profile.release]
   lto = true
   ```

3. **Incremental Compilation**: For faster development builds, enable incremental compilation:
   ```toml
   [profile.dev]
   incremental = true
   ```

4. **Codegen Units**: For maximum optimization, set codegen-units to 1 (may slow build):
   ```toml
   [profile.release]
   codegen-units = 1
   ```

5. **Keep Dependencies Lean**: Remove unused dependencies and keep them updated.

6. **Clean Old Artifacts**: Occasionally run:
   ```bash
   cargo clean
   ```

7. **Use Rustup Toolchain**: Ensure you are using the latest stable toolchain:
   ```bash
   rustup update
   ```

These steps will help you achieve faster and more efficient builds on Linux.

### Shell Completion Scripts

You can generate shell completion scripts for aide using the following command:

```bash
aide completions <shell>
```

Replace `<shell>` with your shell type (e.g., `bash`, `zsh`, `fish`, `elvish`, `powershell`).

Example for bash:
```bash
aide completions bash > /etc/bash_completion.d/aide
```

This enables tab-completion for aide commands in your shell.