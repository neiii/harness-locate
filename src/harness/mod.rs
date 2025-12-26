//! Harness discovery and path resolution.

use std::path::PathBuf;

use crate::error::{Error, Result};
use crate::types::{
    ConfigResource, DirectoryResource, DirectoryStructure, FileFormat, HarnessKind, Scope,
};

pub mod claude_code;
pub mod goose;
pub mod opencode;

/// A discovered harness with resolved base paths.
///
/// Use [`Harness::locate`] to find a harness on the current system.
#[derive(Debug)]
pub struct Harness {
    kind: HarnessKind,
}

impl Harness {
    /// Locate a harness on the current system.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] if the harness is not installed.
    /// Returns [`Error::UnsupportedPlatform`] if the platform is not supported.
    ///
    /// [`Error::NotFound`]: crate::error::Error::NotFound
    /// [`Error::UnsupportedPlatform`]: crate::error::Error::UnsupportedPlatform
    pub fn locate(kind: HarnessKind) -> Result<Self> {
        let is_installed = match kind {
            HarnessKind::ClaudeCode => claude_code::is_installed(),
            HarnessKind::OpenCode => opencode::is_installed(),
            HarnessKind::Goose => goose::is_installed(),
        };

        if is_installed {
            Ok(Self { kind })
        } else {
            Err(Error::NotFound(kind.to_string()))
        }
    }

    /// Returns the kind of harness.
    #[must_use]
    pub fn kind(&self) -> HarnessKind {
        self.kind
    }

    /// Creates a new harness instance for the given kind.
    ///
    /// This does not check if the harness is installed. Use [`is_installed`]
    /// to check installation status, or [`installed`] to get all installed harnesses.
    ///
    /// [`is_installed`]: Harness::is_installed
    /// [`installed`]: Harness::installed
    #[must_use]
    pub fn new(kind: HarnessKind) -> Self {
        Self { kind }
    }

    /// Returns `true` if this harness is installed on the current system.
    ///
    /// Installation is determined by checking if the harness's global
    /// configuration directory exists.
    #[must_use]
    pub fn is_installed(&self) -> bool {
        match self.kind {
            HarnessKind::ClaudeCode => claude_code::is_installed(),
            HarnessKind::OpenCode => opencode::is_installed(),
            HarnessKind::Goose => goose::is_installed(),
        }
    }

    /// Returns all harnesses that are installed on the current system.
    ///
    /// # Errors
    ///
    /// Returns an error if the home directory or config directory cannot
    /// be determined (required to check installation status).
    pub fn installed() -> Result<Vec<Harness>> {
        let mut result = Vec::new();
        for &kind in HarnessKind::ALL {
            let harness = Self::new(kind);
            if harness.is_installed() {
                result.push(harness);
            }
        }
        Ok(result)
    }

    /// Returns the skills directory resource for the given scope.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration directory cannot be determined.
    ///
    /// # Returns
    ///
    /// - `Ok(None)` if this harness does not support skills (Claude Code, Goose)
    /// - `Ok(Some(resource))` if skills are supported (OpenCode)
    pub fn skills(&self, scope: &Scope) -> Result<Option<DirectoryResource>> {
        match self.kind {
            HarnessKind::OpenCode => {
                let path = opencode::skills_dir(scope)
                    .ok_or_else(|| Error::NotFound("skills directory".into()))?;
                Ok(Some(DirectoryResource {
                    exists: path.exists(),
                    path,
                    structure: DirectoryStructure::Nested {
                        subdir_pattern: "*".into(),
                        file_name: "SKILL.md".into(),
                    },
                    file_format: FileFormat::Markdown,
                }))
            }
            HarnessKind::ClaudeCode | HarnessKind::Goose => Ok(None),
        }
    }

    /// Returns the commands directory resource for the given scope.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration directory cannot be determined.
    pub fn commands(&self, scope: &Scope) -> Result<Option<DirectoryResource>> {
        let path = match self.kind {
            HarnessKind::ClaudeCode => claude_code::commands_dir(scope)?,
            HarnessKind::OpenCode => opencode::commands_dir(scope)?,
            HarnessKind::Goose => return Ok(None), // Goose doesn't have commands
        };
        Ok(Some(DirectoryResource {
            exists: path.exists(),
            path,
            structure: DirectoryStructure::Flat {
                file_pattern: "*.md".into(),
            },
            file_format: FileFormat::MarkdownWithFrontmatter,
        }))
    }

    /// Returns the plugins directory resource for the given scope.
    ///
    /// Only OpenCode supports plugins. Claude Code and Goose return `Ok(None)`.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration directory cannot be determined.
    pub fn plugins(&self, scope: &Scope) -> Result<Option<DirectoryResource>> {
        match self.kind {
            HarnessKind::OpenCode => {
                let path = opencode::config_dir(scope)?.join("plugin");
                Ok(Some(DirectoryResource {
                    exists: path.exists(),
                    path,
                    structure: DirectoryStructure::Flat {
                        file_pattern: "*.{js,ts}".into(),
                    },
                    file_format: FileFormat::Json,
                }))
            }
            HarnessKind::ClaudeCode | HarnessKind::Goose => Ok(None),
        }
    }

    /// Returns the agents directory resource for the given scope.
    ///
    /// Only OpenCode supports custom agents. Claude Code and Goose return `Ok(None)`.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration directory cannot be determined.
    pub fn agents(&self, scope: &Scope) -> Result<Option<DirectoryResource>> {
        match self.kind {
            HarnessKind::OpenCode => {
                let path = opencode::config_dir(scope)?.join("agent");
                Ok(Some(DirectoryResource {
                    exists: path.exists(),
                    path,
                    structure: DirectoryStructure::Flat {
                        file_pattern: "*.{yaml,json}".into(),
                    },
                    file_format: FileFormat::Yaml,
                }))
            }
            HarnessKind::ClaudeCode | HarnessKind::Goose => Ok(None),
        }
    }

    /// Returns the base configuration directory path for the given scope.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration directory cannot be determined.
    pub fn config(&self, scope: &Scope) -> Result<PathBuf> {
        match self.kind {
            HarnessKind::ClaudeCode => claude_code::config_dir(scope),
            HarnessKind::OpenCode => opencode::config_dir(scope),
            HarnessKind::Goose => goose::config_dir(scope),
        }
    }

    /// Returns the MCP configuration resource for the given scope.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration directory cannot be determined.
    pub fn mcp(&self, scope: &Scope) -> Result<Option<ConfigResource>> {
        let (file, key_path, format) = match self.kind {
            HarnessKind::ClaudeCode => {
                let base = claude_code::config_dir(scope)?;
                (
                    base.join("claude_desktop_config.json"),
                    "/mcpServers".into(),
                    FileFormat::Json,
                )
            }
            HarnessKind::OpenCode => {
                let base = opencode::config_dir(scope)?;
                (
                    base.join("config.json"),
                    "/mcpServers".into(),
                    FileFormat::Json,
                )
            }
            HarnessKind::Goose => {
                let base = goose::config_dir(scope)?;
                (
                    base.join("config.yaml"),
                    "/extensions".into(),
                    FileFormat::Yaml,
                )
            }
        };
        Ok(Some(ConfigResource {
            file_exists: file.exists(),
            file,
            key_path,
            format,
            schema_url: None,
        }))
    }

    /// Returns the rules directory resource for the given scope.
    ///
    /// Rules files contain behavioral instructions for the AI assistant.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration directory cannot be determined.
    pub fn rules(&self, scope: &Scope) -> Result<Option<DirectoryResource>> {
        let path = match self.kind {
            HarnessKind::ClaudeCode => claude_code::rules_dir(scope),
            HarnessKind::OpenCode => opencode::rules_dir(scope),
            HarnessKind::Goose => goose::rules_dir(scope),
        };
        match path {
            Some(p) => Ok(Some(DirectoryResource {
                exists: p.exists(),
                path: p,
                structure: DirectoryStructure::Flat {
                    file_pattern: "*.md".into(),
                },
                file_format: FileFormat::Markdown,
            })),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locate_claude_code_when_installed() {
        if !claude_code::is_installed() {
            return;
        }

        let result = Harness::locate(HarnessKind::ClaudeCode);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().kind(), HarnessKind::ClaudeCode);
    }

    #[test]
    fn config_global_for_claude_code() {
        if !claude_code::is_installed() {
            return;
        }

        let harness = Harness::locate(HarnessKind::ClaudeCode).unwrap();
        let path = harness.config(&Scope::Global).unwrap();
        assert!(path.is_absolute());
        assert!(path.ends_with(".claude"));
    }

    #[test]
    fn config_project_for_claude_code() {
        if !claude_code::is_installed() {
            return;
        }

        let harness = Harness::locate(HarnessKind::ClaudeCode).unwrap();
        let path = harness
            .config(&Scope::Project(PathBuf::from("/some/project")))
            .unwrap();
        assert_eq!(path, PathBuf::from("/some/project/.claude"));
    }

    #[test]
    fn commands_for_claude_code() {
        if !claude_code::is_installed() {
            return;
        }

        let harness = Harness::locate(HarnessKind::ClaudeCode).unwrap();
        let resource = harness.commands(&Scope::Global).unwrap();
        assert!(resource.is_some());
        let dir = resource.unwrap();
        assert!(dir.path.ends_with("commands"));
        assert!(matches!(dir.structure, DirectoryStructure::Flat { .. }));
    }

    #[test]
    fn skills_none_for_claude_code() {
        let harness = Harness::new(HarnessKind::ClaudeCode);
        let result = harness.skills(&Scope::Global);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn locate_opencode_when_installed() {
        if !opencode::is_installed() {
            return;
        }

        let result = Harness::locate(HarnessKind::OpenCode);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().kind(), HarnessKind::OpenCode);
    }

    #[test]
    fn config_global_for_opencode() {
        if !opencode::is_installed() {
            return;
        }

        let harness = Harness::locate(HarnessKind::OpenCode).unwrap();
        let path = harness.config(&Scope::Global).unwrap();
        assert!(path.is_absolute());
        assert!(path.ends_with("opencode"));
    }

    #[test]
    fn config_project_for_opencode() {
        if !opencode::is_installed() {
            return;
        }

        let harness = Harness::locate(HarnessKind::OpenCode).unwrap();
        let path = harness
            .config(&Scope::Project(PathBuf::from("/some/project")))
            .unwrap();
        assert_eq!(path, PathBuf::from("/some/project/.opencode"));
    }

    #[test]
    fn skills_for_opencode() {
        if !opencode::is_installed() {
            return;
        }

        let harness = Harness::locate(HarnessKind::OpenCode).unwrap();
        let resource = harness.skills(&Scope::Global).unwrap();
        assert!(resource.is_some());
        let dir = resource.unwrap();
        assert!(dir.path.ends_with("skill"));
    }

    #[test]
    fn locate_goose_when_installed() {
        if !goose::is_installed() {
            return;
        }

        let result = Harness::locate(HarnessKind::Goose);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().kind(), HarnessKind::Goose);
    }

    #[test]
    fn config_global_for_goose() {
        if !goose::is_installed() {
            return;
        }

        let harness = Harness::locate(HarnessKind::Goose).unwrap();
        let path = harness.config(&Scope::Global).unwrap();
        assert!(path.is_absolute());
        assert!(path.ends_with("goose"));
    }

    #[test]
    fn config_project_for_goose() {
        if !goose::is_installed() {
            return;
        }

        let harness = Harness::locate(HarnessKind::Goose).unwrap();
        let path = harness
            .config(&Scope::Project(PathBuf::from("/some/project")))
            .unwrap();
        assert_eq!(path, PathBuf::from("/some/project/.goose"));
    }

    #[test]
    fn skills_none_for_goose() {
        let harness = Harness::new(HarnessKind::Goose);
        let result = harness.skills(&Scope::Global);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn rules_none_for_claude_code_global() {
        let harness = Harness::new(HarnessKind::ClaudeCode);
        let result = harness.rules(&Scope::Global);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn rules_project_root_for_claude_code() {
        let harness = Harness::new(HarnessKind::ClaudeCode);
        let resource = harness
            .rules(&Scope::Project(PathBuf::from("/some/project")))
            .unwrap();
        assert!(resource.is_some());
        assert_eq!(resource.unwrap().path, PathBuf::from("/some/project"));
    }

    #[test]
    fn rules_global_for_goose() {
        if !goose::is_installed() {
            return;
        }

        let harness = Harness::locate(HarnessKind::Goose).unwrap();
        let resource = harness.rules(&Scope::Global).unwrap();
        assert!(resource.is_some());
        assert!(resource.unwrap().path.ends_with("goose"));
    }

    #[test]
    fn rules_project_root_for_goose() {
        let harness = Harness::new(HarnessKind::Goose);
        let resource = harness
            .rules(&Scope::Project(PathBuf::from("/some/project")))
            .unwrap();
        assert!(resource.is_some());
        assert_eq!(resource.unwrap().path, PathBuf::from("/some/project"));
    }

    #[test]
    fn plugins_none_for_non_opencode() {
        let harness = Harness::new(HarnessKind::ClaudeCode);
        assert!(harness.plugins(&Scope::Global).unwrap().is_none());

        let harness = Harness::new(HarnessKind::Goose);
        assert!(harness.plugins(&Scope::Global).unwrap().is_none());
    }

    #[test]
    fn agents_none_for_non_opencode() {
        let harness = Harness::new(HarnessKind::ClaudeCode);
        assert!(harness.agents(&Scope::Global).unwrap().is_none());

        let harness = Harness::new(HarnessKind::Goose);
        assert!(harness.agents(&Scope::Global).unwrap().is_none());
    }

    #[test]
    fn mcp_returns_config_resource() {
        if !claude_code::is_installed() {
            return;
        }

        let harness = Harness::new(HarnessKind::ClaudeCode);
        let resource = harness.mcp(&Scope::Global).unwrap();
        assert!(resource.is_some());
        let config = resource.unwrap();
        assert_eq!(config.key_path, "/mcpServers");
    }

    #[test]
    fn commands_none_for_goose() {
        let harness = Harness::new(HarnessKind::Goose);
        let result = harness.commands(&Scope::Global).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn harness_kind_all_contains_all_variants() {
        assert_eq!(HarnessKind::ALL.len(), 3);
        assert!(HarnessKind::ALL.contains(&HarnessKind::ClaudeCode));
        assert!(HarnessKind::ALL.contains(&HarnessKind::OpenCode));
        assert!(HarnessKind::ALL.contains(&HarnessKind::Goose));
    }

    #[test]
    fn new_creates_harness_without_installation_check() {
        let harness = Harness::new(HarnessKind::ClaudeCode);
        assert_eq!(harness.kind(), HarnessKind::ClaudeCode);

        let harness = Harness::new(HarnessKind::OpenCode);
        assert_eq!(harness.kind(), HarnessKind::OpenCode);

        let harness = Harness::new(HarnessKind::Goose);
        assert_eq!(harness.kind(), HarnessKind::Goose);
    }

    #[test]
    fn is_installed_matches_locate() {
        for &kind in HarnessKind::ALL {
            let harness = Harness::new(kind);
            let is_installed = harness.is_installed();
            let locate_result = Harness::locate(kind);
            assert_eq!(is_installed, locate_result.is_ok());
        }
    }

    #[test]
    fn installed_returns_only_installed_harnesses() {
        let installed = Harness::installed().unwrap();
        for harness in &installed {
            assert!(harness.is_installed());
        }
    }
}
