# 🍒 gh_cherry — GitHub Cherry‑Pick TUI

[![Built With Ratatui](https://ratatui.rs/built-with-ratatui/badge.svg)](https://ratatui.rs/)

A fast, cross‑platform Terminal User Interface (TUI) written in Rust to help teams cherry‑pick Pull Requests into target branches during release workflows.

## ✨ Features

- 🎯 Smart PR detection by labels/tags (e.g., sprint and environment)
- 🔍 Auto‑discovery of owners/orgs and repositories when not specified
- 🔐 GitHub auth via `gh` or Personal Access Token
- 🍒 Interactive cherry‑picking with clear progress and results
- ⚡ Merge‑conflict handling and guided resolution
- 🏷️ Tag workflow support (e.g., move from “pending cherrypick” to “cherry picked”)
- 📊 Modern TUI powered by Ratatui

## 🎯 Use case

Great for teams that:

- Merge into a `develop` branch and need selective backports to release branches
- Use sprint/environment tags (e.g., `S28`, `DEV`) to mark PRs that need cherrypicking
- Want a fast TUI to pick PRs and push results with consistent labeling

## 🚀 Installation

### Prerequisites

- Rust 1.70+ and Git
- GitHub CLI (`gh`) authenticated, or a GitHub Personal Access Token

### From source

```bash
git clone https://github.com/ArunPrakashG/gh_cherry.git
cd gh_cherry
cargo build --release
cargo install --path .
```

### Prebuilt binaries

Download from GitHub Releases. Each release includes assets for Windows, macOS, and Linux.

- Windows: unzip and run `scripts/setup.ps1` (adds `gh_cherry.exe` to PATH)
- macOS/Linux: extract and run `scripts/setup_unix.sh` (installs to `~/.local/bin` by default)

## 🔧 Setup

### Authentication

Option 1 — GitHub CLI (recommended):

```bash
gh auth login
```

Option 2 — Personal Access Token:

1. Create a token at https://github.com/settings/tokens with `repo` and `read:org`
2. Export it: `export GITHUB_TOKEN=your_token`

### Configuration

Create `~/.config/gh_cherry/config.toml`:

```toml
[github]
owner = "your-org"
repo = "your-repo"
base_branch = "develop"
target_branch = "main"

[tags]
sprint_pattern = "S\\d+"
environment = "DEV"
pending_tag = "pending cherrypick"
completed_tag = "cherry picked"

[ui]
days_back = 28
page_size = 20
```

## 🧭 Usage

Quick start:

```bash
gh_cherry              # auto-discover owner/repo
gh_cherry -o myorg -r myrepo
gh_cherry --config examples/dev-config.toml
```

Keyboard shortcuts: `↑/↓` or `j/k` navigate • `Enter` select • `Space` multi‑select • `Tab` switch • `Esc` back • `q` quit • `r` refresh • `h` help • `/` search

## 🧪 Development

```bash
git clone https://github.com/ArunPrakashG/gh_cherry.git
cd gh_cherry
cargo build
cargo test
RUST_LOG=debug cargo run -- --config examples/dev-config.toml
```

Main crates: ratatui, octocrab, git2, tokio, clap, serde, toml

## 📦 Releases (CI)

This repo ships a GitHub Actions workflow that builds binaries for Windows, macOS, and Linux and attaches them to a GitHub Release when you push a tag like `v1.2.3` (or run the workflow manually). See `.github/workflows/release.yml`.

## ❓ Troubleshooting

- Auth errors: `gh auth status`, or set `GITHUB_TOKEN` with `repo` and `read:org`
- Repo not found: check owner/name and access; ensure auth is configured
- Git failures: run from a git repo; ensure you have write permissions; stash local changes
- No PRs found: adjust tag patterns or `days_back`; verify base branch

Debug logging: `RUST_LOG=debug gh_cherry`

## 📜 License

MIT — see [LICENSE](LICENSE).

## 🙌 Acknowledgments

- [Ratatui](https://github.com/ratatui/ratatui)
- [Octocrab](https://github.com/XAMPPRocky/octocrab)
- [git2-rs](https://github.com/rust-lang/git2-rs)
- The Rust community
