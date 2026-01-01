//! Skills discovery and fetching for AI coding agents.

mod component;
mod discovery;
mod error;
mod fetch;
mod github;
mod types;

pub use component::parse_skill_descriptor;
pub use discovery::{discover_from_source, discover_plugins};
pub use error::{Error, Result};
pub use fetch::{extract_file, fetch_bytes, fetch_json, list_files};
pub use github::GitHubRef;
pub use types::{PluginDescriptor, PluginSource, SkillDescriptor};
