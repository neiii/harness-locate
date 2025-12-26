//! Core type definitions for harness path resolution.

use std::fmt;
use std::path::PathBuf;

/// Supported AI coding harnesses.
///
/// This enum represents the different AI coding assistants whose
/// configuration paths can be discovered.
///
/// # Extensibility
///
/// This enum is marked `#[non_exhaustive]` to allow adding new
/// harness types in future versions without breaking changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum HarnessKind {
    /// Claude Code (Anthropic's CLI)
    ClaudeCode,
    /// OpenCode
    OpenCode,
    /// Goose (Block's AI coding assistant)
    Goose,
}

impl fmt::Display for HarnessKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ClaudeCode => write!(f, "Claude Code"),
            Self::OpenCode => write!(f, "OpenCode"),
            Self::Goose => write!(f, "Goose"),
        }
    }
}

/// Scope for path resolution.
///
/// Determines whether to look up global (user-level) or
/// project-local configuration paths.
#[derive(Debug, Clone)]
pub enum Scope {
    /// User-level global configuration (e.g., `~/.config/...`)
    Global,
    /// Project-local configuration (e.g., `.claude/` in project root)
    Project(PathBuf),
}

/// Types of paths a harness may provide.
///
/// Each harness can have different configuration directories
/// for different purposes.
///
/// # Extensibility
///
/// This enum is marked `#[non_exhaustive]` to allow adding new
/// path types in future versions without breaking changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PathType {
    /// Main configuration directory
    Config,
    /// Skills/capabilities definitions
    Skills,
    /// Custom commands
    Commands,
    /// MCP (Model Context Protocol) configuration
    Mcp,
    /// Rules and constraints
    Rules,
}
