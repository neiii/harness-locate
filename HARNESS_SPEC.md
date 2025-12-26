# Harness Specification

This document defines the required paths and behaviors for adding a new harness to `get-harness`.

## Required Functions

When adding a new harness, create a module at `src/harness/{name}.rs` with these functions:

| Function | Signature | Required | Notes |
|----------|-----------|----------|-------|
| `global_config_dir` | `fn() -> Result<PathBuf>` | Yes | Base global config directory |
| `project_config_dir` | `fn(&Path) -> PathBuf` | Yes | Base project config directory |
| `config_dir` | `fn(&Scope) -> Result<PathBuf>` | Yes | Dispatches to global/project |
| `commands_dir` | `fn(&Scope) -> Result<PathBuf>` | Yes | Custom commands location |
| `skills_dir` | `fn(&Scope) -> Option<PathBuf>` | Yes | Skills/capabilities (None if unsupported) |
| `mcp_dir` | `fn(&Scope) -> Result<PathBuf>` | Yes | MCP configuration location |
| `rules_dir` | `fn(&Scope) -> Option<PathBuf>` | Yes | Behavioral rules/instructions |
| `is_installed` | `fn() -> bool` | Yes | Check if harness exists on system |

## Path Types Reference

### Config (`config_dir`)
Base configuration directory for the harness.

| Harness | Global | Project |
|---------|--------|---------|
| Claude Code | `~/.claude/` or `$CLAUDE_CONFIG_DIR` | `.claude/` |
| OpenCode | `~/.config/opencode/` | `.opencode/` |
| Goose | `~/.config/goose/` | `.goose/` |

### Commands (`commands_dir`)
Custom slash commands or scripts.

| Harness | Global | Project |
|---------|--------|---------|
| Claude Code | `~/.claude/commands/` | `.claude/commands/` |
| OpenCode | `~/.config/opencode/command/` | `.opencode/command/` |
| Goose | Same as config | Same as config |

### Skills (`skills_dir`)
Reusable capability definitions. Return `None` if the harness doesn't support skills.

| Harness | Global | Project |
|---------|--------|---------|
| Claude Code | `~/.claude/skills/` | `.claude/skills/` |
| OpenCode | `~/.config/opencode/skill/` | `.opencode/skill/` |
| Goose | None | None |

### MCP (`mcp_dir`)
Model Context Protocol server configuration.

| Harness | Global | Project |
|---------|--------|---------|
| Claude Code | `~/.claude.json` | `.mcp.json` |
| OpenCode | Same as config (`opencode.json`) | Same as config (`opencode.json`) |
| Goose | Same as config | Same as config |

### Rules (`rules_dir`)
Behavioral instructions/rules files. **Note**: Project rules typically live at project root, not in a subdirectory.

| Harness | Global | Project | Files |
|---------|--------|---------|-------|
| Claude Code | `~/.claude/` | Project root | `CLAUDE.md`, `CLAUDE.local.md` |
| OpenCode | None | Project root | `AGENTS.md` |
| Goose | `~/.config/goose/` | Project root | `.goosehints`, `AGENTS.md` |

## Integration Checklist

When adding a new harness:

- [ ] Create `src/harness/{name}.rs` with all required functions
- [ ] Add `pub mod {name};` to `src/harness/mod.rs`
- [ ] Add variant to `HarnessKind` enum in `src/types.rs`
- [ ] Add `Display` impl for the new variant
- [ ] Update `Harness::locate()` match arm
- [ ] Update all `Harness::*_path()` methods
- [ ] Add unit tests for each function
- [ ] Add integration tests in `mod.rs`
- [ ] Update this spec with the new harness paths
- [ ] Run `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`

## Design Principles

1. **Return directories, not files**: All path functions return directories. File names are documented but not hard-coded in return values.

2. **Option for optional features**: Use `Option<PathBuf>` for features a harness may not support (e.g., skills).

3. **Result for fallible paths**: Use `Result<PathBuf>` when path resolution can fail (e.g., home directory not found).

4. **Project root for rules**: Rules files conventionally live at project root, not in config subdirectories.

5. **Graceful degradation in tests**: Tests should skip (not fail) when a harness isn't installed.
