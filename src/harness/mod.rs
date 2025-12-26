//! Harness discovery and path resolution.

use std::path::PathBuf;

use crate::error::{Error, Result};
use crate::types::{HarnessKind, Scope};

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

    /// Returns the path to the skills directory for the given scope.
    #[must_use]
    pub fn skills_path(&self, scope: Scope) -> Option<PathBuf> {
        match self.kind {
            HarnessKind::ClaudeCode => claude_code::skills_dir(&scope),
            HarnessKind::OpenCode => opencode::skills_dir(&scope),
            HarnessKind::Goose => goose::skills_dir(&scope),
        }
    }

    /// Returns the path to the commands directory for the given scope.
    #[must_use]
    pub fn commands_path(&self, scope: Scope) -> Option<PathBuf> {
        match self.kind {
            HarnessKind::ClaudeCode => claude_code::commands_dir(&scope).ok(),
            HarnessKind::OpenCode => opencode::commands_dir(&scope).ok(),
            HarnessKind::Goose => goose::commands_dir(&scope).ok(),
        }
    }

    /// Returns the path to the config directory for the given scope.
    #[must_use]
    pub fn config_path(&self, scope: Scope) -> Option<PathBuf> {
        match self.kind {
            HarnessKind::ClaudeCode => claude_code::config_dir(&scope).ok(),
            HarnessKind::OpenCode => opencode::config_dir(&scope).ok(),
            HarnessKind::Goose => goose::config_dir(&scope).ok(),
        }
    }

    /// Returns the path to the MCP configuration directory for the given scope.
    #[must_use]
    pub fn mcp_path(&self, scope: Scope) -> Option<PathBuf> {
        match self.kind {
            HarnessKind::ClaudeCode => claude_code::mcp_dir(&scope).ok(),
            HarnessKind::OpenCode => opencode::mcp_dir(&scope).ok(),
            HarnessKind::Goose => goose::mcp_dir(&scope).ok(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locate_claude_code_when_installed() {
        // This test only passes if Claude Code is installed
        if !claude_code::is_installed() {
            return;
        }

        let result = Harness::locate(HarnessKind::ClaudeCode);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().kind(), HarnessKind::ClaudeCode);
    }

    #[test]
    fn config_path_global_for_claude_code() {
        if !claude_code::is_installed() {
            return;
        }

        let harness = Harness::locate(HarnessKind::ClaudeCode).unwrap();
        let path = harness.config_path(Scope::Global);
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.is_absolute());
        assert!(path.ends_with(".claude"));
    }

    #[test]
    fn config_path_project_for_claude_code() {
        if !claude_code::is_installed() {
            return;
        }

        let harness = Harness::locate(HarnessKind::ClaudeCode).unwrap();
        let path = harness.config_path(Scope::Project(PathBuf::from("/some/project")));
        assert!(path.is_some());
        assert_eq!(path.unwrap(), PathBuf::from("/some/project/.claude"));
    }

    #[test]
    fn commands_path_for_claude_code() {
        if !claude_code::is_installed() {
            return;
        }

        let harness = Harness::locate(HarnessKind::ClaudeCode).unwrap();
        let path = harness.commands_path(Scope::Global);
        assert!(path.is_some());
        assert!(path.unwrap().ends_with("commands"));
    }

    #[test]
    fn skills_path_none_for_claude_code() {
        if !claude_code::is_installed() {
            return;
        }

        let harness = Harness::locate(HarnessKind::ClaudeCode).unwrap();
        assert!(harness.skills_path(Scope::Global).is_none());
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
    fn config_path_global_for_opencode() {
        if !opencode::is_installed() {
            return;
        }

        let harness = Harness::locate(HarnessKind::OpenCode).unwrap();
        let path = harness.config_path(Scope::Global);
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.is_absolute());
        assert!(path.ends_with("opencode"));
    }

    #[test]
    fn config_path_project_for_opencode() {
        if !opencode::is_installed() {
            return;
        }

        let harness = Harness::locate(HarnessKind::OpenCode).unwrap();
        let path = harness.config_path(Scope::Project(PathBuf::from("/some/project")));
        assert!(path.is_some());
        assert_eq!(path.unwrap(), PathBuf::from("/some/project/.opencode"));
    }

    #[test]
    fn skills_path_for_opencode() {
        if !opencode::is_installed() {
            return;
        }

        let harness = Harness::locate(HarnessKind::OpenCode).unwrap();
        let path = harness.skills_path(Scope::Global);
        assert!(path.is_some());
        assert!(path.unwrap().ends_with("skill"));
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
    fn config_path_global_for_goose() {
        if !goose::is_installed() {
            return;
        }

        let harness = Harness::locate(HarnessKind::Goose).unwrap();
        let path = harness.config_path(Scope::Global);
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.is_absolute());
        assert!(path.ends_with("goose"));
    }

    #[test]
    fn config_path_project_for_goose() {
        if !goose::is_installed() {
            return;
        }

        let harness = Harness::locate(HarnessKind::Goose).unwrap();
        let path = harness.config_path(Scope::Project(PathBuf::from("/some/project")));
        assert!(path.is_some());
        assert_eq!(path.unwrap(), PathBuf::from("/some/project/.goose"));
    }

    #[test]
    fn skills_path_none_for_goose() {
        if !goose::is_installed() {
            return;
        }

        let harness = Harness::locate(HarnessKind::Goose).unwrap();
        assert!(harness.skills_path(Scope::Global).is_none());
    }
}
