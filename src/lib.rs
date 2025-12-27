#![doc = include_str!("../README.md")]
//!
//! ## Modules
//!
//! - [`detection`] - Binary detection utilities
//! - [`error`] - Error types
//! - [`harness`] - Harness discovery and path resolution
//! - [`mcp`] - MCP server type definitions
//! - [`types`] - Core type definitions
//! - [`skill`] - Skill file parsing utilities

pub mod detection;
pub mod error;
pub mod harness;
pub mod mcp;
pub mod platform;
pub mod skill;
pub mod types;

pub use detection::find_binary;
pub use error::{Error, Result};
pub use harness::Harness;
pub use mcp::{
    HttpMcpServer, McpCapabilities, McpServer, OAuthConfig, SseMcpServer, StdioMcpServer,
};
pub use skill::{Frontmatter, parse_frontmatter};
pub use types::{
    ConfigResource, DirectoryResource, DirectoryStructure, EnvValue, FileFormat, HarnessKind,
    InstallationStatus, PathType, Scope,
};
