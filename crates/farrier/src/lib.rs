mod discovery;
mod error;
mod fetch;
mod github;
mod marketplace;
mod plugin;

pub use discovery::{discover_skills, discover_skills_ok, SkillDescriptor};
pub use error::Error;
pub use fetch::{extract_zip_to_buffer, fetch_github_archive, fetch_single_file, FileBuffer};
pub use github::{resolve_github_source, GitHubRef};
pub use marketplace::{parse_marketplace, Marketplace, PluginEntry};
pub use plugin::{parse_plugin_manifest, PluginManifest};

pub type Result<T> = std::result::Result<T, Error>;
