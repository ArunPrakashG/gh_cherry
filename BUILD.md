# Build Instructions for gh_cherry

This document describes how to build `gh_cherry` for different platforms.

## Prerequisites

- **Rust 1.70+**: Install from [rustup.rs](https://rustup.rs/)
- **Git**: For cloning the repository
- **GitHub CLI (optional)**: For authentication (`gh auth login`)

## Quick Build (Current Platform)

```bash
# Clone the repository
git clone https://github.com/yourusername/gh_cherry.git
cd gh_cherry

# Build for current platform
cargo build --release

# Run
./target/release/gh_cherry
```

## Platform-Specific Builds

### Windows

**Requirements:**

- Windows 10/11 or Windows Server 2019+
- PowerShell 5.0+ (included with Windows)
- Visual Studio Build Tools or Visual Studio Community

**Build:**

```powershell
# Using PowerShell
.\build-windows.ps1
```

**Output:**

- `dist/gh_cherry-windows-x64.exe` - Single executable file

### macOS

**Requirements:**

- macOS 10.15+ (Catalina)
- Xcode Command Line Tools: `xcode-select --install`

**Build:**

```bash
# Make script executable and run
chmod +x build-macos.sh
./build-macos.sh
```

**Outputs:**

- `dist/gh_cherry-macos-intel` - Intel Macs (x86_64)
- `dist/gh_cherry-macos-arm64` - Apple Silicon (ARM64)
- `dist/gh_cherry-macos-universal` - Universal binary (both architectures)
- `dist/gh_cherry.app` - macOS app bundle (optional)

### Cross-Platform Build

Build for multiple platforms from one machine:

```bash
chmod +x build-cross.sh
./build-cross.sh
```

**Note:** Cross-compilation limitations:

- **From Windows:** Can build Windows binaries only
- **From macOS:** Can build both Windows and macOS binaries
- **From Linux:** Can build Windows and Linux binaries

## Advanced Build Options

### Optimized Release Build

```bash
# Maximum optimizations (slower build, faster runtime)
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Minimize binary size
cargo build --release --features=small
```

### Static Linking (Linux/Windows)

```bash
# Static binary (no system dependencies)
cargo build --release --target x86_64-unknown-linux-musl  # Linux
cargo build --release --target x86_64-pc-windows-gnu      # Windows
```

### Debug Build with Symbols

```bash
# Debug build for development
cargo build

# Release with debug info
cargo build --release --config profile.release.debug=true
```

## Binary Size Optimization

The build scripts automatically apply these optimizations:

1. **Strip debug symbols**: Reduces size by ~30-50%
2. **Release mode**: Full optimizations enabled
3. **LTO (Link Time Optimization)**: Further optimizations

### Manual Size Reduction

```bash
# Additional size reduction (Linux/macOS)
strip target/release/gh_cherry

# Ultimate size reduction with UPX (optional)
upx --best target/release/gh_cherry
```

## Troubleshooting

### Windows Build Issues

**Error: "MSVC not found"**

```powershell
# Install Visual Studio Build Tools
winget install Microsoft.VisualStudio.2022.BuildTools

# Or use MinGW
rustup toolchain install stable-x86_64-pc-windows-gnu
rustup default stable-x86_64-pc-windows-gnu
```

**Error: "PowerShell execution policy"**

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### macOS Build Issues

**Error: "Xcode Command Line Tools not found"**

```bash
xcode-select --install
```

**Error: "Target not found"**

```bash
rustup update
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
```

### Cross-Compilation Issues

**Windows from Linux:**

```bash
# Install MinGW
sudo apt install mingw-w64  # Ubuntu/Debian
sudo dnf install mingw64-gcc  # Fedora

# Add target
rustup target add x86_64-pc-windows-gnu
```

**macOS from Linux (requires osxcross):**

```bash
# This is complex - use GitHub Actions instead
# See .github/workflows/build.yml for CI setup
```

## GitHub Actions CI/CD

For automated builds on push/PR, the repository includes:

- `.github/workflows/build.yml` - Builds for all platforms
- `.github/workflows/release.yml` - Creates releases with binaries

**Outputs:**

- Windows: `gh_cherry-windows-x64.exe`
- macOS Intel: `gh_cherry-macos-intel`
- macOS ARM64: `gh_cherry-macos-arm64`
- macOS Universal: `gh_cherry-macos-universal`
- Linux: `gh_cherry-linux-x64`

## Installation

### From Release

1. Download the appropriate binary from [Releases](https://github.com/yourusername/gh_cherry/releases)
2. Make executable: `chmod +x gh_cherry-*` (macOS/Linux)
3. Move to PATH: `mv gh_cherry-* /usr/local/bin/gh_cherry`

### Using Cargo

```bash
cargo install gh_cherry
```

### Using Homebrew (macOS)

```bash
brew tap yourusername/gh_cherry
brew install gh_cherry
```

## Build Configurations

### Cargo.toml Features

```toml
[features]
default = ["tui"]
tui = ["ratatui", "crossterm"]
small = []  # Smaller binary size
```

Build with specific features:

```bash
cargo build --release --features=small --no-default-features
```

## Performance Notes

- **Binary Size**: ~8-15MB (depending on platform and optimizations)
- **Memory Usage**: ~10-50MB (depending on repository size)
- **Startup Time**: ~100-500ms (depending on GitHub API response)

## Security Notes

- Binaries are **not signed** by default
- macOS may show "unidentified developer" warning
- Windows SmartScreen may block execution
- Use `--allow-unsigned` flags during development

For production, consider:

1. Code signing certificates
2. Notarization (macOS)
3. Windows Authenticode signing
