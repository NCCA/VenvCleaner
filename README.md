# VenvCleaner

## Warning!

This is 100% AI written / Vibe Coded. This initial snapshot has been developed as an expermiment. I have not looked at any of the code! I will post write ups about this at a later date with links etc. What follows is also AI genrated and I've not read it yet!

See [VenvCleaner](VenvCleaner.md) for more info and [AIChat1](AIChat1.md) for the full AI conversation.

## About

A multi-mode Rust application to help manage and clean up Python virtual environment folders (.venv) on Mac and Linux systems.

## Features

- **Three Operation Modes:**
  - **CLI Mode**: Command-line interface with various flags for automated operations
  - **TUI Mode**: Terminal User Interface for interactive management (coming soon)
  - **GUI Mode**: Graphical User Interface using Qt6 (coming soon)

- **Comprehensive .venv Management:**
  - Find .venv directories recursively or in current directory
  - Display detailed information including size, creation date, and last used date
  - Safe deletion with confirmation prompts
  - Force mode for automated cleanup
  - Dry-run mode to preview operations without making changes

- **Smart Analysis:**
  - Color-coded output based on directory age and size
  - Recommendations for cleanup based on usage patterns
  - Human-readable size formatting (KB, MB, GB)
  - Permission checking before deletion attempts

## Installation

### From Source

```bash
git clone <repository-url>
cd VenvCleaner
cargo build --release
```

The binary will be available at `target/release/venv_cleaner`.

### Using Cargo

```bash
cargo install venv_cleaner
```

## Usage

### Basic Commands

```bash
# Search for .venv in current directory
venv_cleaner

# Search in specific directory
venv_cleaner /path/to/search

# Recursive search from current directory
venv_cleaner -r

# Query mode - show information without deleting
venv_cleaner -q -r

# Force mode - delete without prompting
venv_cleaner -r -f

# Dry run - show what would be deleted
venv_cleaner -r --dry-run
```

### Command Line Options

- `DIR` - Directory to search for .venv folders (defaults to current directory)
- `-r, --recursive` - Recursively search from the specified directory
- `-f, --force` - Force delete without prompting for confirmation
- `-q, --query` - Query and display .venv folders with their sizes (no deletion)
- `--dry-run` - Show what would be deleted without actually deleting
- `-v, --verbose` - Enable verbose output (can be used multiple times)
- `--tui` - Launch in Terminal User Interface mode (coming soon)
- `--gui` - Launch in Graphical User Interface mode (coming soon)
- `-h, --help` - Show help information
- `-V, --version` - Show version information

### Examples

#### Query Mode
```bash
# Find all .venv directories and show their information
venv_cleaner -q -r ~/projects

# Output example:
# Found .venv directories:
# ================================================================================
# Location                                                     Size        Created              Last Used
# ------------------------------------------------------------
# /home/user/projects/webapp                                   245.67 MB   2023-08-15 14:30:25  2024-01-10 09:15:42
# /home/user/projects/data-analysis                           1.23 GB     2023-09-01 11:20:10  2023-12-05 16:45:30
# /home/user/projects/old-prototype                           156.45 MB   2023-06-10 08:45:15  2023-07-15 12:30:25
```

#### Interactive Cleanup
```bash
# Interactive cleanup with confirmation prompts
venv_cleaner -r ~/projects

# Output example:
# ────────────────────────────────────────────────────────────
# 📁 /home/user/projects/old-prototype
# 📏 Size: 156.45 MB
# 📅 Last used: 2023-07-15 12:30:25 (180 days ago)
# ⚠️  This .venv hasn't been used in over 90 days
#
# Delete this .venv directory? (y/N): y
# 🗑️  Deleting...
# ✅ Deleted successfully
```

#### Force Mode
```bash
# Delete all old .venv directories without prompting
venv_cleaner -r -f ~/projects
```

#### Dry Run
```bash
# See what would be deleted without actually deleting
venv_cleaner -r --dry-run ~/projects
```

## Output Information

### Query Mode Display
- **Location**: Parent directory containing the .venv
- **Size**: Formatted size (KB/MB/GB) with color coding:
  - 🔴 Red: > 1GB
  - 🟡 Yellow: > 100MB
  - ⚪ Normal: < 100MB
- **Created**: When the .venv was created
- **Last Used**: When the .venv was last modified, with color coding:
  - 🟢 Green: Used within last 30 days
  - 🔴 Red: Not used in over 90 days
  - ⚪ Normal: Used 30-90 days ago

### Recommendations
The tool provides intelligent recommendations based on:
- Age of .venv directories (suggests cleanup for >90 days old)
- Size of .venv directories (highlights large directories >500MB)
- Usage patterns

## Safety Features

- **Permission Checking**: Verifies write permissions before attempting deletion
- **Confirmation Prompts**: Interactive confirmation unless in force mode
- **Dry Run Mode**: Preview operations without making changes
- **Detailed Logging**: Comprehensive logging with multiple verbosity levels
- **Error Handling**: Graceful error handling and reporting

## Future Modes

### TUI Mode (Coming Soon)
- Interactive terminal interface
- Navigate through directories with keyboard shortcuts
- Sort by various criteria (size, date, location)
- Bulk selection and operations

### GUI Mode (Coming Soon)
- Modern Qt6-based graphical interface
- Visual directory tree
- Drag-and-drop operations
- Advanced filtering and search

## Development

### Building
```bash
# Debug build
cargo build

# Release build
cargo build --release

# With specific features
cargo build --features tui
cargo build --features gui
```

### Testing
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test cli::tests
```

### Features
- `cli` (default): Command-line interface
- `tui`: Terminal User Interface (requires ratatui, crossterm)
- `gui`: Graphical User Interface (requires cxx-qt)
- `async`: Async runtime support (requires tokio)

## Platform Support

- ✅ Linux
- ✅ macOS
- ⚠️ Windows (basic support, some features may vary)

## Requirements

- Rust 1.70.0 or later
- For GUI mode: Qt6 development libraries

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes following the Rust style guide
4. Add tests for new functionality
5. Ensure all tests pass (`cargo test`)
6. Commit your changes (`git commit -m 'Add amazing feature'`)
7. Push to the branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Changelog

### v0.1.0
- Initial release with CLI mode
- Basic .venv detection and cleanup
- Query mode for information display
- Interactive and force deletion modes
- Dry-run support
- Comprehensive error handling and logging

## Security

This tool modifies your filesystem by deleting directories. While it includes safety features:
- Always use `--dry-run` first to preview operations
- Be cautious with `-f` (force) mode
- Regularly backup important projects
- The tool only deletes directories named `.venv`

## Support

- Report bugs and request features via GitHub Issues
- Check the documentation for detailed usage examples
- Use `venv_cleaner --help` for quick reference
