use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::plugin::{parse_plugin_manifest, PluginManifest};
use crate::Result;

#[derive(Debug, Clone, PartialEq)]
pub struct SkillDescriptor {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub path: PathBuf,
}

impl From<(PathBuf, PluginManifest)> for SkillDescriptor {
    fn from((path, manifest): (PathBuf, PluginManifest)) -> Self {
        Self {
            name: manifest.name,
            version: manifest.version,
            description: manifest.description,
            path,
        }
    }
}

pub fn discover_skills(files: &HashMap<PathBuf, Vec<u8>>) -> Vec<Result<SkillDescriptor>> {
    files
        .iter()
        .filter(|(path, _)| is_plugin_manifest(path))
        .map(|(path, content)| {
            let manifest_str = std::str::from_utf8(content)?;
            let manifest = parse_plugin_manifest(manifest_str)?;
            let skill_path = path.parent().unwrap_or(Path::new("")).to_path_buf();
            Ok(SkillDescriptor::from((skill_path, manifest)))
        })
        .collect()
}

pub fn discover_skills_ok(files: &HashMap<PathBuf, Vec<u8>>) -> Vec<SkillDescriptor> {
    discover_skills(files)
        .into_iter()
        .filter_map(|r| r.ok())
        .collect()
}

fn is_plugin_manifest(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n == "plugin.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_single_skill() {
        let mut files = HashMap::new();
        files.insert(
            PathBuf::from("skill-a/plugin.json"),
            br#"{"name": "skill-a", "version": "1.0.0"}"#.to_vec(),
        );

        let skills = discover_skills_ok(&files);
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "skill-a");
        assert_eq!(skills[0].path, PathBuf::from("skill-a"));
    }

    #[test]
    fn discovers_multiple_nested_skills() {
        let mut files = HashMap::new();
        files.insert(
            PathBuf::from("skills/auth/plugin.json"),
            br#"{"name": "auth-skill", "version": "1.0.0", "description": "Auth"}"#.to_vec(),
        );
        files.insert(
            PathBuf::from("skills/db/plugin.json"),
            br#"{"name": "db-skill", "version": "2.0.0"}"#.to_vec(),
        );
        files.insert(PathBuf::from("README.md"), b"# Skills repo".to_vec());

        let skills = discover_skills_ok(&files);
        assert_eq!(skills.len(), 2);

        let names: Vec<_> = skills.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"auth-skill"));
        assert!(names.contains(&"db-skill"));
    }

    #[test]
    fn skips_invalid_manifests() {
        let mut files = HashMap::new();
        files.insert(
            PathBuf::from("valid/plugin.json"),
            br#"{"name": "valid", "version": "1.0.0"}"#.to_vec(),
        );
        files.insert(PathBuf::from("invalid/plugin.json"), b"not json".to_vec());

        let skills = discover_skills_ok(&files);
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "valid");
    }
}
