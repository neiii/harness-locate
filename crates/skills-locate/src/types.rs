//! Core type definitions for skills discovery.

use serde::{Deserialize, Serialize};

/// Source location for a plugin.
///
/// Plugins can be sourced from GitHub repositories, direct URLs,
/// or relative paths within a marketplace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum PluginSource {
    /// GitHub repository reference.
    GitHub {
        /// GitHub URL or owner/repo shorthand.
        github: String,
    },
    /// Direct URL to plugin.
    Url {
        /// Full URL to the plugin location.
        url: String,
    },
    /// Relative path within a marketplace repository.
    Relative(String),
}

/// Plugin descriptor containing metadata and skills.
///
/// Represents a plugin as discovered from a repository,
/// including its name, description, and contained skills.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PluginDescriptor {
    /// Plugin name.
    pub name: String,

    /// Optional description of the plugin.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Skills contained in this plugin.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<SkillDescriptor>,
}

/// Skill metadata descriptor.
///
/// Contains metadata extracted from SKILL.md frontmatter,
/// without the full skill body content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SkillDescriptor {
    /// Skill name (required).
    pub name: String,

    /// Optional description of the skill.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Trigger patterns that invoke this skill.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub triggers: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_source_github_serde_roundtrip() {
        let source = PluginSource::GitHub {
            github: "anthropics/claude-code".to_string(),
        };
        let json = serde_json::to_string(&source).unwrap();
        assert_eq!(json, r#"{"github":"anthropics/claude-code"}"#);
        let parsed: PluginSource = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, source);
    }

    #[test]
    fn plugin_source_url_serde_roundtrip() {
        let source = PluginSource::Url {
            url: "https://example.com/plugin".to_string(),
        };
        let json = serde_json::to_string(&source).unwrap();
        assert_eq!(json, r#"{"url":"https://example.com/plugin"}"#);
        let parsed: PluginSource = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, source);
    }

    #[test]
    fn plugin_source_relative_serde_roundtrip() {
        let source = PluginSource::Relative("./plugins/my-plugin".to_string());
        let json = serde_json::to_string(&source).unwrap();
        assert_eq!(json, r#""./plugins/my-plugin""#);
        let parsed: PluginSource = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, source);
    }

    #[test]
    fn plugin_descriptor_full_serde_roundtrip() {
        let plugin = PluginDescriptor {
            name: "test-plugin".to_string(),
            description: Some("A test plugin".to_string()),
            skills: vec![SkillDescriptor {
                name: "test-skill".to_string(),
                description: Some("A test skill".to_string()),
                triggers: vec!["/test".to_string()],
            }],
        };
        let json = serde_json::to_string(&plugin).unwrap();
        let parsed: PluginDescriptor = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, plugin);
    }

    #[test]
    fn plugin_descriptor_minimal_serde_roundtrip() {
        let plugin = PluginDescriptor {
            name: "minimal".to_string(),
            description: None,
            skills: vec![],
        };
        let json = serde_json::to_string(&plugin).unwrap();
        // Optional fields should be omitted
        assert_eq!(json, r#"{"name":"minimal"}"#);
        let parsed: PluginDescriptor = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, plugin);
    }

    #[test]
    fn skill_descriptor_full_serde_roundtrip() {
        let skill = SkillDescriptor {
            name: "code-review".to_string(),
            description: Some("Reviews code for issues".to_string()),
            triggers: vec!["/review".to_string(), "/cr".to_string()],
        };
        let json = serde_json::to_string(&skill).unwrap();
        let parsed: SkillDescriptor = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, skill);
    }

    #[test]
    fn skill_descriptor_minimal_serde_roundtrip() {
        let skill = SkillDescriptor {
            name: "minimal-skill".to_string(),
            description: None,
            triggers: vec![],
        };
        let json = serde_json::to_string(&skill).unwrap();
        assert_eq!(json, r#"{"name":"minimal-skill"}"#);
        let parsed: SkillDescriptor = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, skill);
    }

    #[test]
    fn plugin_descriptor_deserialize_with_defaults() {
        // JSON with only required field
        let json = r#"{"name":"test"}"#;
        let plugin: PluginDescriptor = serde_json::from_str(json).unwrap();
        assert_eq!(plugin.name, "test");
        assert_eq!(plugin.description, None);
        assert!(plugin.skills.is_empty());
    }

    #[test]
    fn skill_descriptor_deserialize_with_defaults() {
        // JSON with only required field
        let json = r#"{"name":"test-skill"}"#;
        let skill: SkillDescriptor = serde_json::from_str(json).unwrap();
        assert_eq!(skill.name, "test-skill");
        assert_eq!(skill.description, None);
        assert!(skill.triggers.is_empty());
    }
}
