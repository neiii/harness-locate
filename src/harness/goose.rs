//! Goose harness implementation.
//!
//! Goose stores its configuration in:
//! - **Global**: `~/.config/goose/`
//! - **Project**: `.goose/` in project root (if exists)

use std::path::PathBuf;

use crate::error::Result;
use crate::platform;
use crate::types::Scope;

/// Returns the global Goose configuration directory.
///
/// Returns `~/.config/goose/` on all platforms.
///
/// # Errors
///
/// Returns an error if the home directory cannot be determined.
pub fn global_config_dir() -> Result<PathBuf> {
    Ok(platform::config_dir()?.join("goose"))
}

/// Returns the project-local Goose configuration directory.
///
/// # Arguments
///
/// * `project_root` - Path to the project root directory
#[must_use]
pub fn project_config_dir(project_root: &std::path::Path) -> PathBuf {
    project_root.join(".goose")
}

/// Returns the commands directory for the given scope.
///
/// Goose does not have a dedicated commands directory, so this
/// returns the config directory itself.
pub fn commands_dir(scope: &Scope) -> Result<PathBuf> {
    config_dir(scope)
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
/// Goose stores MCP configuration in the base config directory.
pub fn mcp_dir(scope: &Scope) -> Result<PathBuf> {
    config_dir(scope)
}

/// Returns the skills directory for the given scope.
///
/// Goose does not have a dedicated skills directory, so this
/// returns `None` for both global and project scopes.
#[must_use]
pub fn skills_dir(_scope: &Scope) -> Option<PathBuf> {
    None
}

/// Checks if Goose is installed on this system.
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
        assert!(path.ends_with("goose"));
    }

    #[test]
    fn project_config_dir_is_relative_to_root() {
        let root = PathBuf::from("/some/project");
        let config = project_config_dir(&root);
        assert_eq!(config, PathBuf::from("/some/project/.goose"));
    }

    #[test]
    fn commands_dir_returns_config_dir() {
        if platform::config_dir().is_err() {
            return;
        }

        let result = commands_dir(&Scope::Global);
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.ends_with("goose"));
    }

    #[test]
    fn skills_dir_returns_none() {
        assert!(skills_dir(&Scope::Global).is_none());
        assert!(skills_dir(&Scope::Project(PathBuf::from("/project"))).is_none());
    }

    #[test]
    fn mcp_dir_returns_config_dir() {
        if platform::config_dir().is_err() {
            return;
        }

        let result = mcp_dir(&Scope::Global);
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.ends_with("goose"));
    }
}
