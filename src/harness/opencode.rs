//! OpenCode harness implementation.
//!
//! OpenCode stores its configuration in:
//! - **Global**: `~/.config/opencode/`
//! - **Project**: `.opencode/` in project root

use std::path::PathBuf;

use crate::error::Result;
use crate::platform;
use crate::types::Scope;

/// Returns the global OpenCode configuration directory.
///
/// Returns `~/.config/opencode/` on all platforms.
///
/// # Errors
///
/// Returns an error if the home directory cannot be determined.
pub fn global_config_dir() -> Result<PathBuf> {
    Ok(platform::config_dir()?.join("opencode"))
}

/// Returns the project-local OpenCode configuration directory.
///
/// # Arguments
///
/// * `project_root` - Path to the project root directory
#[must_use]
pub fn project_config_dir(project_root: &std::path::Path) -> PathBuf {
    project_root.join(".opencode")
}

/// Returns the commands directory for the given scope.
///
/// - **Global**: `~/.config/opencode/command/`
/// - **Project**: `.opencode/command/`
pub fn commands_dir(scope: &Scope) -> Result<PathBuf> {
    match scope {
        Scope::Global => Ok(global_config_dir()?.join("command")),
        Scope::Project(root) => Ok(project_config_dir(root).join("command")),
    }
}

/// Returns the config directory for the given scope.
///
/// This is the base configuration directory.
pub fn config_dir(scope: &Scope) -> Result<PathBuf> {
    match scope {
        Scope::Global => global_config_dir(),
        Scope::Project(root) => Ok(project_config_dir(root)),
    }
}

/// Returns the MCP configuration directory for the given scope.
///
/// OpenCode stores MCP configuration in a `plugin/` subdirectory.
pub fn mcp_dir(scope: &Scope) -> Result<PathBuf> {
    match scope {
        Scope::Global => Ok(global_config_dir()?.join("plugin")),
        Scope::Project(root) => Ok(project_config_dir(root).join("plugin")),
    }
}

/// Returns the skills directory for the given scope.
///
/// - **Global**: `~/.config/opencode/skill/`
/// - **Project**: `.opencode/skill/`
#[must_use]
pub fn skills_dir(scope: &Scope) -> Option<PathBuf> {
    match scope {
        Scope::Global => global_config_dir().ok().map(|p| p.join("skill")),
        Scope::Project(root) => Some(project_config_dir(root).join("skill")),
    }
}

/// Checks if OpenCode is installed on this system.
///
/// Currently checks if the global config directory exists.
pub fn is_installed() -> bool {
    global_config_dir().map(|p| p.exists()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_config_dir_is_absolute() {
        // Skip if config dir cannot be determined (CI environments)
        if platform::config_dir().is_err() {
            return;
        }

        let result = global_config_dir();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.is_absolute());
        assert!(path.ends_with("opencode"));
    }

    #[test]
    fn project_config_dir_is_relative_to_root() {
        let root = PathBuf::from("/some/project");
        let config = project_config_dir(&root);
        assert_eq!(config, PathBuf::from("/some/project/.opencode"));
    }

    #[test]
    fn commands_dir_global() {
        if platform::config_dir().is_err() {
            return;
        }

        let result = commands_dir(&Scope::Global);
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.ends_with("command"));
    }

    #[test]
    fn commands_dir_project() {
        let root = PathBuf::from("/some/project");
        let result = commands_dir(&Scope::Project(root));
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path, PathBuf::from("/some/project/.opencode/command"));
    }

    #[test]
    fn skills_dir_global() {
        if platform::config_dir().is_err() {
            return;
        }

        let result = skills_dir(&Scope::Global);
        assert!(result.is_some());
        let path = result.unwrap();
        assert!(path.ends_with("skill"));
    }

    #[test]
    fn skills_dir_project() {
        let root = PathBuf::from("/some/project");
        let result = skills_dir(&Scope::Project(root));
        assert!(result.is_some());
        let path = result.unwrap();
        assert_eq!(path, PathBuf::from("/some/project/.opencode/skill"));
    }

    #[test]
    fn mcp_dir_global() {
        if platform::config_dir().is_err() {
            return;
        }

        let result = mcp_dir(&Scope::Global);
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.ends_with("plugin"));
    }
}
