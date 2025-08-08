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

## Testing Phase

- [x] Unit tests for util
- [x] Unit tests for ui::state::ListState
- [x] Integration tests for config env overrides
- [x] Unit tests for PR label matching logic
- [x] (Optional) Integration test for git ops with temp repo

## Validation Phase

- [x] Build and clippy clean
- [x] All tests pass locally
- [ ] Document how to run tests

## Progress Log

### [Init] - Plan created

Outlined scope, quick wins, and initial test targets.

### [Progress] - Initial tests added

- Exposed lib interface for tests
- Added util, ListState, PR label matching, config env tests
- Added optional git ops test using a temp repo
- All tests passing locally

## Notes and Observations

- Avoid network: octocrab-dependent functions should be abstracted or tested via pure helpers.
- Config::load reads cherry.env from CWD; tests must isolate working dirs (serialize tests).
- git2 tests can be flaky on CI; defer unless needed.
