//! Harness discovery and path resolution.

use std::path::PathBuf;

use crate::error::Result;
use crate::types::{HarnessKind, Scope};

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
    pub fn locate(_kind: HarnessKind) -> Result<Self> {
        todo!("Implementation in get-harness-bhv.2")
    }

    /// Returns the kind of harness.
    #[must_use]
    pub fn kind(&self) -> HarnessKind {
        self.kind
    }

    /// Returns the path to the skills directory for the given scope.
    #[must_use]
    pub fn skills_path(&self, _scope: Scope) -> Option<PathBuf> {
        todo!("Implementation in get-harness-bhv.2")
    }

    /// Returns the path to the commands directory for the given scope.
    #[must_use]
    pub fn commands_path(&self, _scope: Scope) -> Option<PathBuf> {
        todo!("Implementation in get-harness-bhv.2")
    }

    /// Returns the path to the config directory for the given scope.
    #[must_use]
    pub fn config_path(&self, _scope: Scope) -> Option<PathBuf> {
        todo!("Implementation in get-harness-bhv.2")
    }

    /// Returns the path to the MCP configuration directory for the given scope.
    #[must_use]
    pub fn mcp_path(&self, _scope: Scope) -> Option<PathBuf> {
        todo!("Implementation in get-harness-bhv.2")
    }
}
