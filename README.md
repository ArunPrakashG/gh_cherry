# GitHub Cherry-Pick TUI

[![Built With Ratatui](https://ratatui.rs/built-with-ratatui/badge.svg)](https://ratatui.rs/)

A Terminal User Interface (TUI) application built in Rust to assist with cherry-picking Pull Requests to target branches, specifically designed for release branch workflows.

## Features

- 🎯 **Smart PR Detection**: Automatically finds PRs with specific tags (e.g., "S28 DEV pending cherrypick")
- 🔐 **GitHub Authentication**: Seamlessly integrates with GitHub CLI auth or supports Personal Access Tokens
- 🍒 **Interactive Cherry-Picking**: Select and cherry-pick individual PRs or commits with visual feedback
- ⚡ **Conflict Resolution**: Graceful handling of merge conflicts with interactive resolution
- 🏷️ **Automatic Tag Management**: Updates PR tags from "pending cherrypick" to "cherry picked" after successful operations
- 📊 **Rich TUI**: Modern terminal interface built with Ratatui for an excellent user experience

## Use Case

This tool is designed for development teams using a Git workflow where:
- Developers create PRs to a `develop` branch
- PRs are tagged with sprint identifiers (e.g., "S28") and environment tags ("DEV")
- PRs include a "pending cherrypick" tag when they need to be cherry-picked to release branches
- After successful cherry-picking, tags are updated to "cherry picked"

## Installation

### Prerequisites

- Rust 1.70.0 or later
- Git installed and configured
- GitHub CLI (`gh`) installed and authenticated (recommended)
- Or a GitHub Personal Access Token

### From Source

```bash
git clone https://github.com/yourusername/gh_cherry.git
cd gh_cherry
cargo build --release
cargo install --path .
```

### Using Cargo

```bash
cargo install gh_cherry
```

## Setup

### Authentication

#### Option 1: GitHub CLI (Recommended)
```bash
# Install GitHub CLI if not already installed
# On Windows: winget install GitHub.cli
# On macOS: brew install gh
# On Linux: See https://github.com/cli/cli#installation

# Authenticate with GitHub
gh auth login
```

#### Option 2: Personal Access Token
1. Create a Personal Access Token at https://github.com/settings/tokens
2. Grant the following permissions:
   - `repo` (Full control of private repositories)
   - `read:org` (Read org and team membership)
3. Set the token as an environment variable:
```bash
export GITHUB_TOKEN=your_token_here
```

### Configuration

Create a configuration file at `~/.config/gh_cherry/config.toml`:

```toml
[github]
# Organization or username
owner = "your-org"
# Repository name
repo = "your-repo"
# Base branch to cherry-pick from (usually develop)
base_branch = "develop"
# Default target branch (can be changed in UI)
target_branch = "main"

[tags]
# Sprint tag pattern (e.g., S28, S29, etc.)
sprint_pattern = "S\\d+"
# Environment tag
environment = "DEV"
# Tag indicating PR needs cherry-picking
pending_tag = "pending cherrypick"
# Tag to set after successful cherry-pick
completed_tag = "cherry picked"

[ui]
# Number of days to look back for PRs (default: 28)
days_back = 28
# Items per page in PR list (default: 20)
page_size = 20
```

## Usage

### Basic Usage

```bash
# Start the TUI application
gh_cherry

# Or specify a different repository
gh_cherry --owner myorg --repo myrepo

# Or specify a config file
gh_cherry --config /path/to/config.toml
```

### Command Line Options

```bash
gh_cherry [OPTIONS]

Options:
    -o, --owner <OWNER>          GitHub repository owner
    -r, --repo <REPO>            GitHub repository name
    -c, --config <CONFIG>        Path to configuration file
    -b, --base-branch <BRANCH>   Base branch to cherry-pick from
    -t, --target-branch <BRANCH> Target branch to cherry-pick to
    -d, --days <DAYS>            Number of days to look back for PRs
    -h, --help                   Print help information
    -V, --version                Print version information
```

### TUI Interface

The application provides an intuitive terminal interface with the following screens:

1. **Main Menu**: Choose between different operations
2. **PR List**: Browse PRs matching your criteria
3. **PR Details**: View detailed information about selected PRs
4. **Cherry-Pick Confirmation**: Review changes before applying
5. **Progress**: Real-time feedback during operations
6. **Conflict Resolution**: Interactive conflict resolution interface

### Keyboard Shortcuts

- `↑/↓` or `j/k`: Navigate up/down
- `Enter`: Select/Confirm
- `Space`: Toggle selection (multi-select mode)
- `Tab`: Switch between panels
- `Esc`: Go back/Cancel
- `q`: Quit application
- `r`: Refresh current view
- `h`: Show help
- `/`: Search/Filter

## Development Environment Setup

### Prerequisites

- Rust 1.70.0+
- Git
- GitHub CLI or Personal Access Token
- A test repository with PRs tagged according to the expected pattern

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/gh_cherry.git
cd gh_cherry

# Install dependencies and build
cargo build

# Run tests
cargo test

# Run with development settings
cargo run -- --config examples/dev-config.toml

# Run with debug logging
RUST_LOG=debug cargo run
```

### Development Dependencies

The project uses the following main dependencies:

- `ratatui`: Terminal user interface framework
- `octocrab`: GitHub API client
- `git2`: Git operations
- `tokio`: Async runtime
- `clap`: Command line argument parsing
- `serde`: Serialization/deserialization
- `toml`: Configuration file parsing

### Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run integration tests (requires test repository)
cargo test --features integration-tests

# Run specific test
cargo test test_name
```

### Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass (`cargo test`)
6. Format code (`cargo fmt`)
7. Check with clippy (`cargo clippy`)
8. Commit your changes (`git commit -m 'Add amazing feature'`)
9. Push to the branch (`git push origin feature/amazing-feature`)
10. Open a Pull Request

## Troubleshooting

### Common Issues

#### Authentication Errors
```
Error: GitHub authentication failed
```
- Ensure GitHub CLI is installed and authenticated: `gh auth status`
- Or set a valid Personal Access Token: `export GITHUB_TOKEN=your_token`
- Check token permissions include `repo` and `read:org`

#### Repository Not Found
```
Error: Repository not found or access denied
```
- Verify repository owner and name are correct
- Ensure you have access to the repository
- Check if repository is private and authentication is properly configured

#### Git Operations Failed
```
Error: Failed to perform git operation
```
- Ensure you're running from within a Git repository
- Verify you have write permissions to the repository
- Check if there are uncommitted changes that need to be stashed

#### No PRs Found
```
No PRs found matching criteria
```
- Verify tag patterns in configuration match your repository's tagging scheme
- Check the date range (increase `days_back` in config)
- Ensure PRs exist on the specified base branch

### Debug Mode

Enable debug logging to troubleshoot issues:

```bash
RUST_LOG=debug gh_cherry
```

### Getting Help

- Create an issue on GitHub: [Issues](https://github.com/yourusername/gh_cherry/issues)
- Check existing documentation and FAQ
- Join discussions: [Discussions](https://github.com/yourusername/gh_cherry/discussions)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Ratatui](https://github.com/ratatui/ratatui) for the excellent TUI framework
- [Octocrab](https://github.com/XAMPPRocky/octocrab) for GitHub API integration
- [git2-rs](https://github.com/rust-lang/git2-rs) for Git operations
- The Rust community for excellent tooling and libraries
