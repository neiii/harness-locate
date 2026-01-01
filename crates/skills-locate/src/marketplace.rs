use serde::Deserialize;

use crate::{Error, Result};

#[derive(Debug, Clone, Deserialize)]
pub struct PluginEntry {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Marketplace {
    pub version: u32,
    pub plugins: Vec<PluginEntry>,
}

pub fn parse_marketplace(json: &str) -> Result<Marketplace> {
    serde_json::from_str(json).map_err(Error::Json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_marketplace() {
        let json = r#"{
            "version": 1,
            "plugins": [
                {
                    "name": "test-plugin",
                    "source": "owner/repo",
                    "description": "A test plugin",
                    "tags": ["test", "example"]
                },
                {
                    "name": "minimal",
                    "source": "https://github.com/foo/bar"
                }
            ]
        }"#;

        let result = parse_marketplace(json).unwrap();
        assert_eq!(result.version, 1);
        assert_eq!(result.plugins.len(), 2);
        assert_eq!(result.plugins[0].name, "test-plugin");
        assert_eq!(result.plugins[0].source, "owner/repo");
        assert_eq!(
            result.plugins[0].description,
            Some("A test plugin".to_string())
        );
        assert_eq!(result.plugins[0].tags, vec!["test", "example"]);
        assert_eq!(result.plugins[1].name, "minimal");
        assert!(result.plugins[1].description.is_none());
        assert!(result.plugins[1].tags.is_empty());
    }

    #[test]
    fn parse_with_path() {
        let json = r#"{
            "version": 1,
            "plugins": [{
                "name": "nested",
                "source": "owner/repo",
                "path": "skills/my-skill"
            }]
        }"#;

        let result = parse_marketplace(json).unwrap();
        assert_eq!(result.plugins[0].path, Some("skills/my-skill".to_string()));
    }

    #[test]
    fn parse_missing_required_field() {
        let json = r#"{"version": 1, "plugins": [{"name": "no-source"}]}"#;
        assert!(parse_marketplace(json).is_err());
    }

    #[test]
    fn parse_empty_plugins() {
        let json = r#"{"version": 1, "plugins": []}"#;
        let result = parse_marketplace(json).unwrap();
        assert!(result.plugins.is_empty());
    }
}
