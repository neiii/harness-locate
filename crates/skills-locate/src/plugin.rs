use serde::Deserialize;

use crate::{Error, Result};

#[derive(Debug, Clone, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

pub fn parse_plugin_manifest(json: &str) -> Result<PluginManifest> {
    serde_json::from_str(json).map_err(Error::Json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_manifest() {
        let json = r#"{
            "name": "my-skill",
            "version": "1.0.0",
            "description": "A useful skill",
            "author": "Developer",
            "homepage": "https://example.com",
            "license": "MIT",
            "keywords": ["ai", "coding"],
            "files": ["skill.md", "prompts/"],
            "dependencies": ["other-skill"]
        }"#;

        let result = parse_plugin_manifest(json).unwrap();
        assert_eq!(result.name, "my-skill");
        assert_eq!(result.version, "1.0.0");
        assert_eq!(result.description, Some("A useful skill".to_string()));
        assert_eq!(result.keywords, vec!["ai", "coding"]);
        assert_eq!(result.files, vec!["skill.md", "prompts/"]);
    }

    #[test]
    fn parse_minimal_manifest() {
        let json = r#"{"name": "minimal", "version": "0.1.0"}"#;
        let result = parse_plugin_manifest(json).unwrap();
        assert_eq!(result.name, "minimal");
        assert_eq!(result.version, "0.1.0");
        assert!(result.description.is_none());
        assert!(result.keywords.is_empty());
    }

    #[test]
    fn parse_missing_required() {
        let json = r#"{"name": "no-version"}"#;
        assert!(parse_plugin_manifest(json).is_err());
    }
}
