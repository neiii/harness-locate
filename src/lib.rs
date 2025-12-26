//! Cross-platform harness path discovery for AI coding CLI tools.
//!
//! This library provides a unified way to discover configuration paths
//! for various AI coding assistants (Claude Code, OpenCode, Goose).

pub mod error;
pub mod harness;
pub mod types;

pub use error::{Error, Result};
pub use harness::Harness;
pub use types::{HarnessKind, PathType, Scope};
