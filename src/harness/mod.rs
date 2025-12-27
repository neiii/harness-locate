//! Harness discovery and path resolution.

use std::path::PathBuf;

use serde_json::json;

use crate::error::{Error, Result};
use crate::mcp::{McpCapabilities, McpServer};
use crate::types::{
    ConfigResource, DirectoryResource, DirectoryStructure, FileFormat, HarnessKind,
    InstallationStatus, Scope,
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
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use get_harness::{Harness, HarnessKind};
    ///
    /// let harness = Harness::locate(HarnessKind::ClaudeCode)?;
    /// println!("Found {} at {:?}", harness.kind(), harness.config(&get_harness::Scope::Global)?);
    /// # Ok::<(), get_harness::Error>(())
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// use get_harness::{Harness, HarnessKind};
    ///
    /// let harness = Harness::new(HarnessKind::Goose);
    /// assert_eq!(harness.kind(), HarnessKind::Goose);
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// use get_harness::{Harness, HarnessKind};
    ///
    /// // Create without checking installation
    /// let harness = Harness::new(HarnessKind::OpenCode);
    /// assert_eq!(harness.kind(), HarnessKind::OpenCode);
    /// ```
    #[must_use]
    pub fn new(kind: HarnessKind) -> Self {
        Self { kind }
    }

    /// Returns `true` if this harness is installed on the current system.
    ///
    /// Installation is determined by checking if the harness's global
    /// configuration directory exists.
    ///
    /// # Examples
    ///
    /// ```
    /// use get_harness::{Harness, HarnessKind};
    ///
    /// let harness = Harness::new(HarnessKind::Goose);
    /// if harness.is_installed() {
    ///     println!("Goose is available");
    /// }
    /// ```
    #[must_use]
    pub fn is_installed(&self) -> bool {
        match self.kind {
            HarnessKind::ClaudeCode => claude_code::is_installed(),
            HarnessKind::OpenCode => opencode::is_installed(),
            HarnessKind::Goose => goose::is_installed(),
        }
    }

    /// Returns detailed installation status for this harness.
    ///
    /// Checks both binary availability in PATH and config directory existence.
    ///
    /// # Errors
    ///
    /// Returns an error if binary detection fails due to a system error.
    pub fn installation_status(&self) -> Result<InstallationStatus> {
        let binary_path = self.find_first_binary()?;

        let config_path = match self.kind {
            HarnessKind::ClaudeCode => claude_code::global_config_dir().ok(),
            HarnessKind::OpenCode => opencode::global_config_dir().ok(),
            HarnessKind::Goose => goose::global_config_dir().ok(),
        }
        .filter(|p| p.exists());

        let status = match (binary_path, config_path) {
            (Some(binary_path), Some(config_path)) => InstallationStatus::FullyInstalled {
                binary_path,
                config_path,
            },
            (Some(binary_path), None) => InstallationStatus::BinaryOnly { binary_path },
            (None, Some(config_path)) => InstallationStatus::ConfigOnly { config_path },
            (None, None) => InstallationStatus::NotInstalled,
        };

        Ok(status)
    }

    fn find_first_binary(&self) -> Result<Option<PathBuf>> {
        for name in self.kind.binary_names() {
            if let Some(path) = crate::detection::find_binary(name)? {
                return Ok(Some(path));
            }
        }
        Ok(None)
    }

    /// Returns all harnesses that are installed on the current system.
    ///
    /// # Errors
    ///
    /// Returns an error if the home directory or config directory cannot
    /// be determined (required to check installation status).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use get_harness::Harness;
    ///
    /// for harness in Harness::installed()? {
    ///     println!("{} is installed", harness.kind());
    /// }
    /// # Ok::<(), get_harness::Error>(())
    /// ```
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
    /// - `Ok(None)` if this harness does not support skills (Goose)
    /// - `Ok(Some(resource))` if skills are supported (Claude Code, OpenCode)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use get_harness::{Harness, HarnessKind, Scope};
    ///
    /// let harness = Harness::new(HarnessKind::ClaudeCode);
    /// if let Some(skills) = harness.skills(&Scope::Global)? {
    ///     println!("Skills directory: {}", skills.path.display());
    /// }
    /// # Ok::<(), get_harness::Error>(())
    /// ```
    pub fn skills(&self, scope: &Scope) -> Result<Option<DirectoryResource>> {
        match self.kind {
            HarnessKind::ClaudeCode => {
                let path = claude_code::skills_dir(scope)
                    .ok_or_else(|| Error::NotFound("skills directory".into()))?;
                Ok(Some(DirectoryResource {
                    exists: path.exists(),
                    path,
                    structure: DirectoryStructure::Nested {
                        subdir_pattern: "*".into(),
                        file_name: "SKILL.md".into(),
                    },
                    file_format: FileFormat::MarkdownWithFrontmatter,
                }))
            }
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
            HarnessKind::Goose => {
                let path = goose::skills_dir(scope)
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
        }
    }

    /// Returns the commands directory resource for the given scope.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration directory cannot be determined.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use get_harness::{Harness, HarnessKind, Scope};
    ///
    /// let harness = Harness::new(HarnessKind::OpenCode);
    /// if let Some(commands) = harness.commands(&Scope::Global)? {
    ///     println!("Commands at: {}", commands.path.display());
    /// }
    /// # Ok::<(), get_harness::Error>(())
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use get_harness::{Harness, HarnessKind, Scope};
    ///
    /// let harness = Harness::new(HarnessKind::OpenCode);
    /// if let Some(plugins) = harness.plugins(&Scope::Global)? {
    ///     println!("Plugins at: {}", plugins.path.display());
    /// }
    /// # Ok::<(), get_harness::Error>(())
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use get_harness::{Harness, HarnessKind, Scope};
    ///
    /// let harness = Harness::new(HarnessKind::OpenCode);
    /// if let Some(agents) = harness.agents(&Scope::Global)? {
    ///     println!("Agents at: {}", agents.path.display());
    /// }
    /// # Ok::<(), get_harness::Error>(())
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use get_harness::{Harness, HarnessKind, Scope};
    /// use std::path::PathBuf;
    ///
    /// let harness = Harness::new(HarnessKind::ClaudeCode);
    ///
    /// // Global config
    /// let global = harness.config(&Scope::Global)?;
    ///
    /// // Project config
    /// let project = harness.config(&Scope::Project(PathBuf::from("/my/project")))?;
    /// # Ok::<(), get_harness::Error>(())
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use get_harness::{Harness, HarnessKind, Scope};
    ///
    /// let harness = Harness::new(HarnessKind::ClaudeCode);
    /// if let Some(mcp) = harness.mcp(&Scope::Global)? {
    ///     println!("MCP config: {}", mcp.file.display());
    ///     println!("Key path: {}", mcp.key_path);
    /// }
    /// # Ok::<(), get_harness::Error>(())
    /// ```
    pub fn mcp(&self, scope: &Scope) -> Result<Option<ConfigResource>> {
        let (file, key_path, format) = match self.kind {
            HarnessKind::ClaudeCode => {
                // Claude Code CLI uses different files than Claude Desktop:
                // - Global: ~/.claude.json (in home dir, NOT in .claude/)
                // - Project: .mcp.json (in project root, NOT in .claude/)
                let file = match scope {
                    Scope::Global => crate::platform::home_dir()?.join(".claude.json"),
                    Scope::Project(root) => root.join(".mcp.json"),
                };
                (file, "/mcpServers".into(), FileFormat::Json)
            }
            HarnessKind::OpenCode => {
                let base = opencode::config_dir(scope)?;
                (base.join("opencode.json"), "/mcp".into(), FileFormat::Json)
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

    /// Returns the MCP capabilities for this harness.
    ///
    /// Describes what MCP features this harness supports, such as transport
    /// types (stdio, SSE, HTTP) and configuration options (OAuth, headers, etc.).
    ///
    /// # Example
    ///
    /// ```
    /// use get_harness::{Harness, HarnessKind};
    ///
    /// let harness = Harness::new(HarnessKind::OpenCode);
    /// let caps = harness.mcp_capabilities();
    /// assert!(caps.oauth);  // OpenCode supports OAuth
    /// ```
    #[must_use]
    pub fn mcp_capabilities(&self) -> McpCapabilities {
        McpCapabilities::for_kind(self.kind)
    }

    /// Checks if this harness supports a specific MCP server configuration.
    ///
    /// This performs field-aware validation, checking not just the transport type
    /// but also whether the harness supports the specific features used by the server:
    ///
    /// - Stdio servers: checks `cwd` and `timeout_ms` if present
    /// - SSE servers: checks `headers` and `timeout_ms` if present
    /// - HTTP servers: checks `headers`, `oauth`, and `timeout_ms` if present
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use get_harness::{Harness, HarnessKind};
    /// use get_harness::mcp::{McpServer, HttpMcpServer, OAuthConfig};
    ///
    /// let server = McpServer::Http(HttpMcpServer {
    ///     url: "https://api.example.com/mcp".to_string(),
    ///     headers: HashMap::new(),
    ///     oauth: Some(OAuthConfig {
    ///         client_id: Some("app".to_string()),
    ///         client_secret: None,
    ///         scope: None,
    ///     }),
    ///     enabled: true,
    ///     timeout_ms: None,
    /// });
    ///
    /// let opencode = Harness::new(HarnessKind::OpenCode);
    /// assert!(opencode.supports_mcp_server(&server));  // OpenCode supports HTTP + OAuth
    ///
    /// let claude = Harness::new(HarnessKind::ClaudeCode);
    /// assert!(claude.supports_mcp_server(&server));   // Claude supports HTTP + OAuth
    /// ```
    #[must_use]
    pub fn supports_mcp_server(&self, server: &McpServer) -> bool {
        let caps = self.mcp_capabilities();

        match server {
            McpServer::Stdio(s) => {
                if !caps.stdio {
                    return false;
                }
                if s.cwd.is_some() && !caps.cwd {
                    return false;
                }
                if s.timeout_ms.is_some() && !caps.timeout {
                    return false;
                }
                true
            }
            McpServer::Sse(s) => {
                if !caps.sse {
                    return false;
                }
                if !s.headers.is_empty() && !caps.headers {
                    return false;
                }
                if s.timeout_ms.is_some() && !caps.timeout {
                    return false;
                }
                true
            }
            McpServer::Http(s) => {
                if !caps.http {
                    return false;
                }
                if !s.headers.is_empty() && !caps.headers {
                    return false;
                }
                if s.oauth.is_some() && !caps.oauth {
                    return false;
                }
                if s.timeout_ms.is_some() && !caps.timeout {
                    return false;
                }
                true
            }
        }
    }

    /// Validates an MCP server configuration for this harness.
    ///
    /// Combines base validation with harness-specific capability checks.
    /// Returns detailed issues explaining any incompatibilities.
    #[must_use]
    pub fn validate_mcp_server(
        &self,
        server: &McpServer,
    ) -> Vec<crate::validation::ValidationIssue> {
        crate::validation::validate_for_harness(server, self.kind)
    }

    /// Returns the rules directory resource for the given scope.
    ///
    /// Rules files contain behavioral instructions for the AI assistant.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration directory cannot be determined.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use get_harness::{Harness, HarnessKind, Scope};
    /// use std::path::PathBuf;
    ///
    /// let harness = Harness::new(HarnessKind::ClaudeCode);
    /// if let Some(rules) = harness.rules(&Scope::Project(PathBuf::from(".")))? {
    ///     println!("Rules directory: {}", rules.path.display());
    /// }
    /// # Ok::<(), get_harness::Error>(())
    /// ```
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

    /// Converts an MCP server configuration to native harness format.
    ///
    /// # Arguments
    ///
    /// * `name` - The server name/identifier
    /// * `server` - The normalized MCP server configuration
    ///
    /// # Errors
    ///
    /// Returns `Error::UnsupportedMcpConfig` if the server uses features
    /// not supported by this harness.
    ///
    /// # Example
    ///
    /// ```
    /// use get_harness::{Harness, HarnessKind};
    /// use get_harness::mcp::{McpServer, StdioMcpServer};
    ///
    /// let harness = Harness::new(HarnessKind::ClaudeCode);
    /// let server = McpServer::Stdio(StdioMcpServer {
    ///     command: "node".to_string(),
    ///     args: vec!["server.js".to_string()],
    ///     env: Default::default(),
    ///     cwd: None,
    ///     enabled: true,
    ///     timeout_ms: None,
    /// });
    ///
    /// let native = harness.mcp_to_native("my-server", &server).unwrap();
    /// ```
    pub fn mcp_to_native(&self, name: &str, server: &McpServer) -> Result<serde_json::Value> {
        if !self.supports_mcp_server(server) {
            return Err(Error::UnsupportedMcpConfig {
                harness: self.kind.to_string(),
                reason: "Server uses unsupported features".into(),
            });
        }

        match self.kind {
            HarnessKind::ClaudeCode => self.mcp_to_native_claude_code(name, server),
            HarnessKind::OpenCode => self.mcp_to_native_opencode(name, server),
            HarnessKind::Goose => self.mcp_to_native_goose(name, server),
        }
    }

    fn mcp_to_native_claude_code(
        &self,
        _name: &str,
        server: &McpServer,
    ) -> Result<serde_json::Value> {
        match server {
            McpServer::Stdio(s) => {
                let mut env_map = serde_json::Map::new();
                for (key, value) in &s.env {
                    env_map.insert(key.clone(), json!(value.to_native(HarnessKind::ClaudeCode)));
                }

                let mut obj = serde_json::Map::new();
                obj.insert("command".into(), json!(s.command));

                if !s.args.is_empty() {
                    obj.insert("args".into(), json!(s.args));
                }

                if !env_map.is_empty() {
                    obj.insert("env".into(), json!(env_map));
                }

                Ok(json!(obj))
            }
            McpServer::Sse(s) => {
                let mut obj = serde_json::Map::new();
                obj.insert("type".into(), json!("sse"));
                obj.insert("url".into(), json!(s.url));

                if !s.headers.is_empty() {
                    let mut headers_map = serde_json::Map::new();
                    for (key, value) in &s.headers {
                        headers_map
                            .insert(key.clone(), json!(value.to_native(HarnessKind::ClaudeCode)));
                    }
                    obj.insert("headers".into(), json!(headers_map));
                }

                Ok(json!(obj))
            }
            McpServer::Http(s) => {
                let mut obj = serde_json::Map::new();
                obj.insert("type".into(), json!("http"));
                obj.insert("url".into(), json!(s.url));

                if !s.headers.is_empty() {
                    let mut headers_map = serde_json::Map::new();
                    for (key, value) in &s.headers {
                        headers_map
                            .insert(key.clone(), json!(value.to_native(HarnessKind::ClaudeCode)));
                    }
                    obj.insert("headers".into(), json!(headers_map));
                }

                Ok(json!(obj))
            }
        }
    }

    fn mcp_to_native_goose(&self, name: &str, server: &McpServer) -> Result<serde_json::Value> {
        match server {
            McpServer::Stdio(s) => {
                let mut obj = serde_json::Map::new();
                obj.insert("name".into(), json!(name));
                obj.insert("type".into(), json!("stdio"));
                obj.insert("cmd".into(), json!(s.command));

                if !s.args.is_empty() {
                    obj.insert("args".into(), json!(s.args));
                }

                if !s.env.is_empty() {
                    let mut envs_map = serde_json::Map::new();
                    for (key, value) in &s.env {
                        envs_map.insert(key.clone(), json!(value.to_native(HarnessKind::Goose)));
                    }
                    obj.insert("envs".into(), json!(envs_map));
                }

                obj.insert("enabled".into(), json!(s.enabled));

                if let Some(timeout_ms) = s.timeout_ms {
                    let timeout_seconds = timeout_ms / 1000;
                    obj.insert("timeout".into(), json!(timeout_seconds));
                }

                obj.insert("description".into(), json!(""));

                Ok(json!(obj))
            }
            McpServer::Sse(s) => {
                let mut obj = serde_json::Map::new();
                obj.insert("name".into(), json!(name));
                obj.insert("type".into(), json!("sse"));
                obj.insert("uri".into(), json!(s.url));
                obj.insert("enabled".into(), json!(s.enabled));

                if let Some(timeout_ms) = s.timeout_ms {
                    let timeout_seconds = timeout_ms / 1000;
                    obj.insert("timeout".into(), json!(timeout_seconds));
                }

                obj.insert("description".into(), json!(""));

                Ok(json!(obj))
            }
            McpServer::Http(s) => {
                let mut obj = serde_json::Map::new();
                obj.insert("name".into(), json!(name));
                obj.insert("type".into(), json!("streamable_http"));
                obj.insert("uri".into(), json!(s.url));

                if !s.headers.is_empty() {
                    let mut headers_map = serde_json::Map::new();
                    for (key, value) in &s.headers {
                        headers_map.insert(key.clone(), json!(value.to_native(HarnessKind::Goose)));
                    }
                    obj.insert("headers".into(), json!(headers_map));
                }

                obj.insert("enabled".into(), json!(s.enabled));

                if let Some(timeout_ms) = s.timeout_ms {
                    let timeout_seconds = timeout_ms / 1000;
                    obj.insert("timeout".into(), json!(timeout_seconds));
                }

                obj.insert("description".into(), json!(""));

                Ok(json!(obj))
            }
        }
    }

    fn mcp_to_native_opencode(&self, _name: &str, server: &McpServer) -> Result<serde_json::Value> {
        match server {
            McpServer::Stdio(stdio) => {
                // Build command array: [command, ...args]
                let mut command = vec![stdio.command.clone()];
                command.extend(stdio.args.clone());

                // Convert environment variables to OpenCode format
                let environment: serde_json::Map<String, serde_json::Value> = stdio
                    .env
                    .iter()
                    .map(|(k, v)| {
                        let value = v.to_native(HarnessKind::OpenCode);
                        (k.clone(), json!(value))
                    })
                    .collect();

                let mut config = json!({
                    "type": "local",
                    "command": command,
                    "enabled": stdio.enabled,
                });

                // Add optional fields
                if !environment.is_empty() {
                    config["environment"] = json!(environment);
                }
                if let Some(timeout) = stdio.timeout_ms {
                    config["timeout"] = json!(timeout);
                }

                Ok(config)
            }
            McpServer::Http(http) => {
                // Convert headers to OpenCode format
                let headers: serde_json::Map<String, serde_json::Value> = http
                    .headers
                    .iter()
                    .map(|(k, v)| {
                        let value = v.to_native(HarnessKind::OpenCode);
                        (k.clone(), json!(value))
                    })
                    .collect();

                let mut config = json!({
                    "type": "remote",
                    "url": http.url,
                    "enabled": http.enabled,
                });

                // Add optional fields
                if !headers.is_empty() {
                    config["headers"] = json!(headers);
                }
                if let Some(timeout) = http.timeout_ms {
                    config["timeout"] = json!(timeout);
                }
                if let Some(oauth) = &http.oauth {
                    let mut oauth_config = serde_json::Map::new();
                    if let Some(client_id) = &oauth.client_id {
                        oauth_config.insert("client_id".to_string(), json!(client_id));
                    }
                    if let Some(client_secret) = &oauth.client_secret {
                        let secret_value = client_secret.to_native(HarnessKind::OpenCode);
                        oauth_config.insert("client_secret".to_string(), json!(secret_value));
                    }
                    if let Some(scope) = &oauth.scope {
                        oauth_config.insert("scope".to_string(), json!(scope));
                    }
                    if !oauth_config.is_empty() {
                        config["oauth"] = json!(oauth_config);
                    }
                }

                Ok(config)
            }
            McpServer::Sse(sse) => {
                let headers: serde_json::Map<String, serde_json::Value> = sse
                    .headers
                    .iter()
                    .map(|(k, v)| {
                        let value = v.to_native(HarnessKind::OpenCode);
                        (k.clone(), json!(value))
                    })
                    .collect();

                let mut config = json!({
                    "type": "remote",
                    "url": sse.url,
                    "enabled": sse.enabled,
                });

                if !headers.is_empty() {
                    config["headers"] = json!(headers);
                }
                if let Some(timeout) = sse.timeout_ms {
                    config["timeout"] = json!(timeout);
                }

                Ok(config)
            }
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
    fn skills_for_claude_code() {
        if !claude_code::is_installed() {
            return;
        }

        let harness = Harness::locate(HarnessKind::ClaudeCode).unwrap();
        let resource = harness.skills(&Scope::Global).unwrap();
        assert!(resource.is_some());
        let dir = resource.unwrap();
        assert!(dir.path.ends_with("skills"));
        assert!(matches!(dir.structure, DirectoryStructure::Nested { .. }));
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
    fn skills_for_goose_global() {
        if crate::platform::config_dir().is_err() {
            return;
        }

        let harness = Harness::new(HarnessKind::Goose);
        let result = harness.skills(&Scope::Global);
        assert!(result.is_ok());
        let resource = result.unwrap();
        assert!(resource.is_some());
        let dir = resource.unwrap();
        assert!(dir.path.ends_with("agents/skills"));
        assert!(matches!(dir.structure, DirectoryStructure::Nested { .. }));
    }

    #[test]
    fn skills_for_goose_project() {
        let harness = Harness::new(HarnessKind::Goose);
        let result = harness.skills(&Scope::Project(PathBuf::from("/some/project")));
        assert!(result.is_ok());
        let resource = result.unwrap();
        assert!(resource.is_some());
        let dir = resource.unwrap();
        assert_eq!(dir.path, PathBuf::from("/some/project/.agents/skills"));
        assert!(matches!(dir.structure, DirectoryStructure::Nested { .. }));
    }

    #[test]
    fn rules_for_claude_code_global() {
        if !claude_code::is_installed() {
            return;
        }

        let harness = Harness::new(HarnessKind::ClaudeCode);
        let result = harness.rules(&Scope::Global);
        assert!(result.is_ok());
        let resource = result.unwrap();
        assert!(resource.is_some());
        assert!(resource.unwrap().path.ends_with(".claude"));
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

    #[test]
    fn mcp_capabilities_returns_correct_for_each_harness() {
        let claude = Harness::new(HarnessKind::ClaudeCode);
        assert!(claude.mcp_capabilities().oauth);

        let opencode = Harness::new(HarnessKind::OpenCode);
        assert!(opencode.mcp_capabilities().oauth);

        let goose = Harness::new(HarnessKind::Goose);
        assert!(goose.mcp_capabilities().oauth);
    }

    #[test]
    fn supports_mcp_server_stdio_basic() {
        use crate::mcp::StdioMcpServer;

        let server = McpServer::Stdio(StdioMcpServer {
            command: "node".to_string(),
            args: vec![],
            env: std::collections::HashMap::new(),
            cwd: None,
            enabled: true,
            timeout_ms: None,
        });

        // All harnesses support basic stdio
        for &kind in HarnessKind::ALL {
            let harness = Harness::new(kind);
            assert!(
                harness.supports_mcp_server(&server),
                "{kind:?} should support basic stdio"
            );
        }
    }

    #[test]
    fn supports_mcp_server_stdio_with_timeout() {
        use crate::mcp::StdioMcpServer;

        let server = McpServer::Stdio(StdioMcpServer {
            command: "node".to_string(),
            args: vec![],
            env: std::collections::HashMap::new(),
            cwd: None,
            enabled: true,
            timeout_ms: Some(30000),
        });

        let claude = Harness::new(HarnessKind::ClaudeCode);
        assert!(claude.supports_mcp_server(&server));

        let opencode = Harness::new(HarnessKind::OpenCode);
        assert!(opencode.supports_mcp_server(&server));
    }

    #[test]
    fn supports_mcp_server_http_with_oauth() {
        use crate::mcp::{HttpMcpServer, OAuthConfig};

        let server = McpServer::Http(HttpMcpServer {
            url: "https://example.com".to_string(),
            headers: std::collections::HashMap::new(),
            oauth: Some(OAuthConfig {
                client_id: Some("app".to_string()),
                client_secret: None,
                scope: None,
            }),
            enabled: true,
            timeout_ms: None,
        });

        let claude = Harness::new(HarnessKind::ClaudeCode);
        assert!(claude.supports_mcp_server(&server));

        let opencode = Harness::new(HarnessKind::OpenCode);
        assert!(opencode.supports_mcp_server(&server));
    }

    #[test]
    fn supports_mcp_server_sse() {
        use crate::mcp::SseMcpServer;

        let server = McpServer::Sse(SseMcpServer {
            url: "https://example.com/sse".to_string(),
            headers: std::collections::HashMap::new(),
            enabled: true,
            timeout_ms: None,
        });

        let opencode = Harness::new(HarnessKind::OpenCode);
        assert!(opencode.supports_mcp_server(&server));

        let claude = Harness::new(HarnessKind::ClaudeCode);
        assert!(claude.supports_mcp_server(&server));
    }

    #[test]
    fn mcp_to_native_goose_stdio() {
        use crate::mcp::StdioMcpServer;

        let harness = Harness::new(HarnessKind::Goose);
        let server = McpServer::Stdio(StdioMcpServer {
            command: "node".to_string(),
            args: vec!["server.js".to_string()],
            env: std::collections::HashMap::new(),
            cwd: None,
            enabled: true,
            timeout_ms: Some(30000),
        });

        let result = harness.mcp_to_native("test-server", &server).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("type").unwrap(), "stdio");
        assert_eq!(obj.get("cmd").unwrap(), "node");
        assert_eq!(obj.get("name").unwrap(), "test-server");
        assert_eq!(obj.get("timeout").unwrap(), 30);
        assert_eq!(obj.get("description").unwrap(), "");
        assert_eq!(obj.get("enabled").unwrap(), true);

        let args = obj.get("args").unwrap().as_array().unwrap();
        assert_eq!(args[0], "server.js");
    }

    #[test]
    fn mcp_to_native_goose_sse() {
        use crate::mcp::SseMcpServer;

        let harness = Harness::new(HarnessKind::Goose);
        let server = McpServer::Sse(SseMcpServer {
            url: "https://example.com/sse".to_string(),
            headers: std::collections::HashMap::new(),
            enabled: true,
            timeout_ms: Some(45000),
        });

        let result = harness.mcp_to_native("sse-server", &server).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("type").unwrap(), "sse");
        assert_eq!(obj.get("uri").unwrap(), "https://example.com/sse");
        assert_eq!(obj.get("name").unwrap(), "sse-server");
        assert_eq!(obj.get("timeout").unwrap(), 45);
        assert_eq!(obj.get("description").unwrap(), "");
        assert_eq!(obj.get("enabled").unwrap(), true);
    }

    #[test]
    fn mcp_to_native_goose_http() {
        use crate::mcp::HttpMcpServer;
        use crate::types::EnvValue;

        let harness = Harness::new(HarnessKind::Goose);
        let mut headers = std::collections::HashMap::new();
        headers.insert("Authorization".to_string(), EnvValue::plain("Bearer token"));

        let server = McpServer::Http(HttpMcpServer {
            url: "https://api.example.com/mcp".to_string(),
            headers,
            oauth: None,
            enabled: true,
            timeout_ms: Some(60000),
        });

        let result = harness.mcp_to_native("http-server", &server).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("type").unwrap(), "streamable_http");
        assert_eq!(obj.get("uri").unwrap(), "https://api.example.com/mcp");
        assert_eq!(obj.get("name").unwrap(), "http-server");
        assert_eq!(obj.get("timeout").unwrap(), 60);
        assert_eq!(obj.get("description").unwrap(), "");
        assert_eq!(obj.get("enabled").unwrap(), true);

        let headers = obj.get("headers").unwrap().as_object().unwrap();
        assert_eq!(headers.get("Authorization").unwrap(), "Bearer token");
    }

    #[test]
    fn mcp_to_native_goose_http_with_oauth() {
        use crate::mcp::{HttpMcpServer, OAuthConfig};

        let harness = Harness::new(HarnessKind::Goose);
        let server = McpServer::Http(HttpMcpServer {
            url: "https://example.com".to_string(),
            headers: std::collections::HashMap::new(),
            oauth: Some(OAuthConfig {
                client_id: Some("app".to_string()),
                client_secret: None,
                scope: None,
            }),
            enabled: true,
            timeout_ms: None,
        });

        let result = harness.mcp_to_native("test", &server);
        assert!(result.is_ok());
    }

    #[test]
    fn mcp_to_native_goose_timeout_converted_to_seconds() {
        use crate::mcp::StdioMcpServer;

        let harness = Harness::new(HarnessKind::Goose);
        let server = McpServer::Stdio(StdioMcpServer {
            command: "test".to_string(),
            args: vec![],
            env: std::collections::HashMap::new(),
            cwd: None,
            enabled: true,
            timeout_ms: Some(30000),
        });

        let result = harness.mcp_to_native("test", &server).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("timeout").unwrap(), 30);
    }

    #[test]
    fn mcp_to_native_goose_env_vars_resolved() {
        use crate::mcp::StdioMcpServer;
        use crate::types::EnvValue;

        // SAFETY: Test runs single-threaded; no concurrent access to this env var
        unsafe { std::env::set_var("TEST_GOOSE_ENV_VAR", "resolved_test_value") };

        let harness = Harness::new(HarnessKind::Goose);
        let mut env = std::collections::HashMap::new();
        env.insert("API_KEY".to_string(), EnvValue::env("TEST_GOOSE_ENV_VAR"));

        let server = McpServer::Stdio(StdioMcpServer {
            command: "test".to_string(),
            args: vec![],
            env,
            cwd: None,
            enabled: true,
            timeout_ms: None,
        });

        let result = harness.mcp_to_native("test", &server).unwrap();
        let obj = result.as_object().unwrap();
        let envs = obj.get("envs").unwrap().as_object().unwrap();

        assert_eq!(envs.get("API_KEY").unwrap(), "resolved_test_value");

        unsafe { std::env::remove_var("TEST_GOOSE_ENV_VAR") };
    }

    #[test]
    fn mcp_to_native_opencode_stdio() {
        use crate::mcp::StdioMcpServer;
        use crate::types::EnvValue;

        let harness = Harness::new(HarnessKind::OpenCode);
        let mut env = std::collections::HashMap::new();
        env.insert("API_KEY".to_string(), EnvValue::env("MY_KEY"));

        let server = McpServer::Stdio(StdioMcpServer {
            command: "node".to_string(),
            args: vec!["server.js".to_string()],
            env,
            cwd: None,
            enabled: true,
            timeout_ms: Some(30000),
        });

        let result = harness.mcp_to_native("test-server", &server).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("type").unwrap(), "local");
        assert_eq!(obj.get("enabled").unwrap(), true);
        assert_eq!(obj.get("timeout").unwrap(), 30000);

        let cmd = obj.get("command").unwrap().as_array().unwrap();
        assert_eq!(cmd.len(), 2);
        assert_eq!(cmd[0], "node");
        assert_eq!(cmd[1], "server.js");

        let environment = obj.get("environment").unwrap().as_object().unwrap();
        assert_eq!(environment.get("API_KEY").unwrap(), "{env:MY_KEY}");
    }

    #[test]
    fn mcp_to_native_opencode_http_with_oauth() {
        use crate::mcp::{HttpMcpServer, OAuthConfig};
        use crate::types::EnvValue;

        let harness = Harness::new(HarnessKind::OpenCode);
        let mut headers = std::collections::HashMap::new();
        headers.insert("X-Custom".to_string(), EnvValue::plain("value"));

        let server = McpServer::Http(HttpMcpServer {
            url: "https://api.example.com/mcp".to_string(),
            headers,
            oauth: Some(OAuthConfig {
                client_id: Some("my-client".to_string()),
                client_secret: Some(EnvValue::env("OAUTH_SECRET")),
                scope: Some("read write".to_string()),
            }),
            enabled: true,
            timeout_ms: Some(60000),
        });

        let result = harness.mcp_to_native("api-server", &server).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("type").unwrap(), "remote");
        assert_eq!(obj.get("url").unwrap(), "https://api.example.com/mcp");
        assert_eq!(obj.get("enabled").unwrap(), true);
        assert_eq!(obj.get("timeout").unwrap(), 60000);

        let headers_obj = obj.get("headers").unwrap().as_object().unwrap();
        assert_eq!(headers_obj.get("X-Custom").unwrap(), "value");

        let oauth = obj.get("oauth").unwrap().as_object().unwrap();
        assert_eq!(oauth.get("client_id").unwrap(), "my-client");
        assert_eq!(oauth.get("client_secret").unwrap(), "{env:OAUTH_SECRET}");
        assert_eq!(oauth.get("scope").unwrap(), "read write");
    }

    #[test]
    fn mcp_to_native_opencode_sse() {
        use crate::mcp::SseMcpServer;

        let harness = Harness::new(HarnessKind::OpenCode);
        let server = McpServer::Sse(SseMcpServer {
            url: "https://example.com/sse".to_string(),
            headers: std::collections::HashMap::new(),
            enabled: true,
            timeout_ms: None,
        });

        let result = harness.mcp_to_native("test", &server);
        assert!(result.is_ok());

        let obj = result.unwrap();
        let obj = obj.as_object().unwrap();
        assert_eq!(obj.get("type").unwrap(), "remote");
        assert_eq!(obj.get("url").unwrap(), "https://example.com/sse");
    }

    #[test]
    fn mcp_to_native_opencode_command_array_format() {
        use crate::mcp::StdioMcpServer;

        let harness = Harness::new(HarnessKind::OpenCode);
        let server = McpServer::Stdio(StdioMcpServer {
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "@modelcontextprotocol/server".to_string()],
            env: std::collections::HashMap::new(),
            cwd: None,
            enabled: true,
            timeout_ms: None,
        });

        let result = harness.mcp_to_native("npx-server", &server).unwrap();
        let obj = result.as_object().unwrap();

        let cmd = obj.get("command").unwrap().as_array().unwrap();
        assert_eq!(cmd.len(), 3);
        assert_eq!(cmd[0], "npx");
        assert_eq!(cmd[1], "-y");
        assert_eq!(cmd[2], "@modelcontextprotocol/server");
    }

    #[test]
    fn mcp_to_native_opencode_http_without_oauth() {
        use crate::mcp::HttpMcpServer;

        let harness = Harness::new(HarnessKind::OpenCode);
        let server = McpServer::Http(HttpMcpServer {
            url: "https://simple.example.com".to_string(),
            headers: std::collections::HashMap::new(),
            oauth: None,
            enabled: false,
            timeout_ms: None,
        });

        let result = harness.mcp_to_native("simple", &server).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("type").unwrap(), "remote");
        assert_eq!(obj.get("url").unwrap(), "https://simple.example.com");
        assert_eq!(obj.get("enabled").unwrap(), false);
        assert!(obj.get("oauth").is_none());
        assert!(obj.get("timeout").is_none());
        assert!(obj.get("headers").is_none());
    }
}
