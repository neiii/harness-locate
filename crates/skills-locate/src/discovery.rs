//! Plugin discovery from GitHub repositories.

use crate::component::parse_skill_descriptor;
use crate::error::{Error, Result};
use crate::fetch::{extract_file, fetch_bytes, list_files};
use crate::github::GitHubRef;
use crate::types::{PluginDescriptor, PluginSource};

#[derive(Debug, Clone, serde::Deserialize)]
struct Marketplace {
    plugins: Vec<MarketplaceEntry>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct MarketplaceEntry {
    source: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct PluginJson {
    name: String,
    #[serde(default)]
    description: Option<String>,
}

pub fn discover_plugins(repo_url: &str) -> Result<Vec<PluginDescriptor>> {
    let github_ref = GitHubRef::parse(repo_url)?;
    let archive_url = github_ref.archive_url();
    let archive_bytes = fetch_bytes(&archive_url)?;

    let marketplace_path = find_marketplace_json(&archive_bytes)?;
    let marketplace_content = extract_file(&archive_bytes, &marketplace_path)?;
    let marketplace: Marketplace = serde_json::from_str(&marketplace_content)?;

    let mut plugins = Vec::new();
    let prefix = extract_archive_prefix(&archive_bytes)?;

    for entry in marketplace.plugins {
        let plugin_path = resolve_plugin_path(&entry.source);

        if let Ok(plugin) = discover_single_plugin(&archive_bytes, &prefix, &plugin_path) {
            plugins.push(plugin);
        }
    }

    Ok(plugins)
}

fn find_marketplace_json(archive: &[u8]) -> Result<String> {
    let candidates = list_files(archive, "marketplace.json")?;

    for path in candidates {
        if path.contains(".claude-plugin/marketplace.json") {
            return Ok(path);
        }
    }

    Err(Error::NotFound(
        ".claude-plugin/marketplace.json".to_string(),
    ))
}

fn extract_archive_prefix(archive: &[u8]) -> Result<String> {
    let files = list_files(archive, "")?;
    if let Some(first) = files.first()
        && let Some(slash_pos) = first.find('/')
    {
        return Ok(first[..=slash_pos].to_string());
    }
    Ok(String::new())
}

fn resolve_plugin_path(source: &str) -> String {
    source.strip_prefix("./").unwrap_or(source).to_string()
}

fn discover_single_plugin(
    archive: &[u8],
    prefix: &str,
    plugin_path: &str,
) -> Result<PluginDescriptor> {
    let plugin_json_path = format!("{prefix}{plugin_path}/.claude-plugin/plugin.json");
    let alt_plugin_json_path = format!("{prefix}{plugin_path}/plugin.json");

    let plugin_content = extract_file(archive, &plugin_json_path)
        .or_else(|_| extract_file(archive, &alt_plugin_json_path))?;

    let plugin_json: PluginJson = serde_json::from_str(&plugin_content)?;

    let skills_prefix = format!("{prefix}{plugin_path}/skills/");
    let skill_files = list_files(archive, "SKILL.md")?;

    let mut skills = Vec::new();
    for skill_path in skill_files {
        if skill_path.starts_with(&skills_prefix)
            && let Ok(skill_content) = extract_file(archive, &skill_path)
            && let Ok(skill) = parse_skill_descriptor(&skill_content)
        {
            skills.push(skill);
        }
    }

    Ok(PluginDescriptor {
        name: plugin_json.name,
        description: plugin_json.description,
        skills,
    })
}

pub fn discover_from_source(source: &PluginSource) -> Result<Vec<PluginDescriptor>> {
    match source {
        PluginSource::GitHub { github } => discover_plugins(github),
        PluginSource::Url { url } => discover_plugins(url),
        PluginSource::Relative(_) => Err(Error::NotFound(
            "Cannot discover from relative path without base URL".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_plugin_path_strips_prefix() {
        assert_eq!(resolve_plugin_path("./plugins/foo"), "plugins/foo");
        assert_eq!(resolve_plugin_path("plugins/bar"), "plugins/bar");
    }

    #[test]
    #[ignore = "requires network"]
    fn discover_anthropics_claude_code() {
        let plugins = discover_plugins("https://github.com/anthropics/claude-code").unwrap();
        assert!(
            plugins.len() >= 13,
            "Expected at least 13 plugins, got {}",
            plugins.len()
        );

        let names: Vec<_> = plugins.iter().map(|p| p.name.as_str()).collect();
        assert!(
            names.contains(&"Code Review"),
            "Should contain Code Review plugin"
        );
    }
}
