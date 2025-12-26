//! Core type definitions for harness path resolution.

use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

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

/// File formats used by harness configuration files.
///
/// Different harnesses use different formats for their configuration,
/// commands, and other resources.
///
/// # Extensibility
///
/// This enum is marked `#[non_exhaustive]` to allow adding new
/// formats in future versions without breaking changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FileFormat {
    /// Standard JSON format.
    Json,
    /// JSON with comments (JSONC).
    Jsonc,
    /// YAML format.
    Yaml,
    /// Plain Markdown.
    Markdown,
    /// Markdown with YAML frontmatter.
    MarkdownWithFrontmatter,
}

/// Directory layout structure for resource directories.
///
/// Harnesses organize their resources in different ways:
/// - Flat: Files directly in the directory (e.g., `commands/foo.md`)
/// - Nested: Subdirectory per resource (e.g., `skills/foo/SKILL.md`)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DirectoryStructure {
    /// Files directly in the directory.
    ///
    /// Example: `commands/foo.md`, `commands/bar.md`
    Flat {
        /// Glob pattern for matching files (e.g., `"*.md"`).
        file_pattern: String,
    },
    /// Subdirectory per resource with a fixed filename inside.
    ///
    /// Example: `skills/foo/SKILL.md`, `skills/bar/SKILL.md`
    Nested {
        /// Pattern for subdirectory names (e.g., `"*"`).
        subdir_pattern: String,
        /// Fixed filename within each subdirectory (e.g., `"SKILL.md"`).
        file_name: String,
    },
}

/// A directory-based resource location.
///
/// Represents a directory that contains multiple resource files,
/// such as commands or skills directories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryResource {
    /// Path to the directory.
    pub path: PathBuf,
    /// Whether the directory currently exists on the filesystem.
    pub exists: bool,
    /// How resources are organized within the directory.
    pub structure: DirectoryStructure,
    /// Format of files within the directory.
    pub file_format: FileFormat,
}

/// A configuration file resource location.
///
/// Represents a single configuration file that may contain
/// multiple configuration entries, accessed via a key path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigResource {
    /// Path to the configuration file.
    pub file: PathBuf,
    /// Whether the file currently exists on the filesystem.
    pub file_exists: bool,
    /// JSON pointer path to the relevant section (e.g., `"/mcpServers"`).
    pub key_path: String,
    /// Format of the configuration file.
    pub format: FileFormat,
    /// Optional JSON Schema URL for validation.
    pub schema_url: Option<String>,
}
