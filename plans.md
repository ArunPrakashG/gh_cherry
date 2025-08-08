# Project Plan: Testing and Quality for gh_cherry

## Problem Statement

We need a structured plan and initial test coverage for the gh_cherry TUI app (Rust). Focus on safe behavior, config correctness, and small pure components, while avoiding brittle tests that require live GitHub or a real git repo.

## Research Phase

- [x] Fetch project structure and dependencies
- [x] Identify testable units without external services
- [x] Note modules that need light refactor to enable testing (helper functions, pub(crate) exports)

## Implementation Phase

- [x] Expose a small library crate interface for testing (lib.rs)
- [x] Add util::short_sha tests
- [x] Add ListState behavior tests (selection, wrapping)
- [x] Add Config env override tests using a temp working directory (serialized)
- [x] Add PR label matching helper and tests (no network)
- [x] Consider minimal git operations test with a temporary repo (if stable)
      // New: distribution and docs
- [x] Polish README with standard sections and emojis
- [x] Add GitHub Actions workflow to build multi-platform releases and publish assets

## Testing Phase

- [x] Unit tests for util
- [x] Unit tests for ui::state::ListState
- [x] Integration tests for config env overrides
- [x] Unit tests for PR label matching logic
- [x] (Optional) Integration test for git ops with temp repo

## Validation Phase

- [x] Build and clippy clean
- [x] All tests pass locally
- [x] Document how to run tests
- [x] Add Windows setup script and docs
- [x] Release workflow validated for tag pushes and manual dispatch

## Progress Log

### [Init] - Plan created

Outlined scope, quick wins, and initial test targets.

### [Progress] - Initial tests added

- Exposed lib interface for tests
- Added util, ListState, PR label matching, config env tests
- Added optional git ops test using a temp repo
- All tests passing locally

### [2025-08-08] - Windows setup script

- Added `scripts/setup.ps1` to install `gh_cherry.exe` to a per-user directory and update PATH.
- Updated README with Windows setup instructions and options.
- Added `-Scope Machine` option (requires Admin) to install under Program Files and update HKLM PATH.

### [2025-08-08] - Unix/macOS setup script

- Added `scripts/setup_unix.sh` to install `gh_cherry` to a per-user bin and update PATH via shell profiles.
- Updated README with Unix/macOS setup and a short Run section.

### [2025-08-08] - README polish and Release workflow

- Rewrote README with clearer structure, emojis, correct repo links, and a concise CI section.
- Added `.github/workflows/release.yml` building assets for Windows, Linux (Ubuntu), macOS (x86_64 and ARM64) and attaching them to releases via `softprops/action-gh-release`.
- Artifacts include the binary plus setup scripts and README.

## Notes and Observations

- Avoid network: octocrab-dependent functions should be abstracted or tested via pure helpers.
- Config::load reads cherry.env from CWD; tests must isolate working dirs (serialize tests).
- git2 tests can be flaky on CI; defer unless needed.
- GitHub Actions uses `dtolnay/rust-toolchain@stable` and `Swatinem/rust-cache@v2` which are the current community-preferred options.
- Release assets are packaged as zip on macOS/Windows and tar.gz on Linux; names follow `gh_cherry-<platform>.{zip|tar.gz}`.
