#!/usr/bin/env bash
set -euo pipefail

# gh_cherry Unix/macOS setup script
# - Installs gh_cherry to a per-user bin directory (default: ~/.local/bin)
# - Optionally appends that directory to your shell PATH in a profile file
#
# Usage (from the extracted release folder containing gh_cherry and scripts/):
#   bash ./scripts/setup_unix.sh [--install-dir DIR] [--force] [--no-path-update]

INSTALL_DIR="$HOME/.local/bin"
FORCE=0
NO_PATH_UPDATE=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --install-dir)
      INSTALL_DIR="$2"; shift 2 ;;
    --force)
      FORCE=1; shift ;;
    --no-path-update)
      NO_PATH_UPDATE=1; shift ;;
    -h|--help)
      cat <<EOF
gh_cherry setup (Unix/macOS)
Usage: $0 [--install-dir DIR] [--force] [--no-path-update]
  --install-dir     Target install directory (default: $HOME/.local/bin)
  --force           Overwrite existing binary if present
  --no-path-update  Skip updating shell profile PATH
EOF
      exit 0 ;;
    *) echo "Unknown option: $1"; exit 1 ;;
  esac
done

info() { printf '\033[36m[INFO]\033[0m %s\n' "$*"; }
warn() { printf '\033[33m[WARN]\033[0m %s\n' "$*"; }
err()  { printf '\033[31m[ERROR]\033[0m %s\n' "$*"; }

# Locate source binary relative to this script
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
SOURCE_EXE="${SCRIPT_DIR}/../gh_cherry"

if [[ ! -f "$SOURCE_EXE" ]]; then
  err "Could not locate gh_cherry next to the setup script. Ensure the binary and scripts folder are together."
  exit 1
fi

info "Installing gh_cherry from: $SOURCE_EXE"
info "Target install directory: $INSTALL_DIR"

mkdir -p "$INSTALL_DIR"
DEST_EXE="$INSTALL_DIR/gh_cherry"

if [[ -f "$DEST_EXE" && $FORCE -eq 0 ]]; then
  warn "gh_cherry already exists at destination. Use --force to overwrite. Skipping copy."
else
  install -m 0755 "$SOURCE_EXE" "$DEST_EXE"
  info "Copied to: $DEST_EXE"
fi

# PATH update
if [[ $NO_PATH_UPDATE -eq 0 ]]; then
  # Is INSTALL_DIR already in PATH?
  case ":$PATH:" in
    *:"$INSTALL_DIR":*) ALREADY=1 ;;
    *) ALREADY=0 ;;
  esac

  if [[ $ALREADY -eq 1 ]]; then
    info "Install directory already present in PATH."
  else
    # Try to append to a profile file
    CANDIDATES=("$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.profile" "$HOME/.bash_profile" "$HOME/.zprofile")
    UPDATED=0
    for file in "${CANDIDATES[@]}"; do
      # Create the file if it doesn't exist
      if [[ ! -e "$file" ]]; then
        # Only create bashrc/profile files; skip zsh files unless in zsh
        touch "$file" || true
      fi
      if [[ -w "$file" ]]; then
        if ! grep -Fq "$INSTALL_DIR" "$file" 2>/dev/null; then
          printf '\n# Added by gh_cherry setup: add install dir to PATH\n' >>"$file"
          printf 'export PATH="%s:$PATH"\n' "$INSTALL_DIR" >>"$file"
          UPDATED=1
          info "Updated PATH in: $file"
          break
        fi
      fi
    done
    if [[ $UPDATED -eq 0 ]]; then
      warn "Could not update PATH automatically. Please add to your shell profile:\n    export PATH=\"$INSTALL_DIR:$PATH\""
    fi
  fi
else
  warn "Skipping PATH update due to --no-path-update."
fi

printf '\n\033[32mInstallation complete.\033[0m\n'
printf 'Run: gh_cherry\n'
printf 'If not recognized, restart your terminal or source your profile file.\n'
