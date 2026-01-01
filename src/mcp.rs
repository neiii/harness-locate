//! MCP (Model Context Protocol) server type definitions.
//!
//! This module defines normalized types for MCP server configurations
//! that work across all harnesses (Claude Code, OpenCode, Goose).

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::types::{EnvValue, HarnessKind};

/// Returns `true` for serde default.
fn default_true() -> bool {
    true
}

/// A normalized MCP server configuration.
///
/// MCP servers can use different transport mechanisms:
/// - **Stdio**: Local process communication via stdin/stdout
/// - **SSE**: Server-Sent Events for real-time streaming
/// - **HTTP**: Streamable HTTP for request/response patterns
///
/// The enum is tagged by `transport` for clean JSON serialization:
/// ```json
/// { "transport": "stdio", "command": "node", "args": ["server.js"] }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "transport", rename_all = "snake_case")]
pub enum McpServer {
    /// Local stdio-based MCP server.
    Stdio(StdioMcpServer),
    /// SSE (Server-Sent Events) MCP server.
    Sse(SseMcpServer),
    /// HTTP/Streamable HTTP MCP server.
    Http(HttpMcpServer),
}

/// Configuration for a stdio-based MCP server.
///
/// Stdio servers are local processes that communicate via stdin/stdout.
/// This is the most common type of MCP server.
///
/// # Example
///
/// ```
/// use harness_locate::mcp::StdioMcpServer;
///
/// let server = StdioMcpServer {
///     command: "npx".to_string(),
///     args: vec!["-y".to_string(), "@modelcontextprotocol/server-filesystem".to_string()],
///     env: Default::default(),
///     cwd: None,
///     enabled: true,
///     timeout_ms: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StdioMcpServer {
    /// The command to execute (e.g., `"node"`, `"npx"`).
    pub command: String,

    /// Command-line arguments passed to the command.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,

    /// Environment variables for the process.
    ///
    /// Values can be plain strings or environment variable references.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, EnvValue>,

    /// Working directory for the process.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<PathBuf>,

    /// Whether this server is enabled.
    ///
    /// Defaults to `true`. Disabled servers are skipped during loading.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Connection timeout in milliseconds.
    ///
    /// If not specified, harness-specific defaults apply.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
}

/// Configuration for an SSE (Server-Sent Events) MCP server.
///
/// SSE servers connect to a remote endpoint that streams events.
///
/// # Example
///
/// ```
/// use harness_locate::mcp::SseMcpServer;
///
/// let server = SseMcpServer {
///     url: "https://api.example.com/mcp/sse".to_string(),
///     headers: Default::default(),
///     enabled: true,
///     timeout_ms: Some(30000),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseMcpServer {
    /// The SSE endpoint URL.
    pub url: String,

    /// HTTP headers to include in requests.
    ///
    /// Values can be plain strings or environment variable references,
    /// useful for authentication tokens.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, EnvValue>,

    /// Whether this server is enabled.
    ///
    /// Defaults to `true`. Disabled servers are skipped during loading.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Connection timeout in milliseconds.
    ///
    /// If not specified, harness-specific defaults apply.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
}

/// Configuration for an HTTP/Streamable HTTP MCP server.
///
/// HTTP servers use standard HTTP requests with optional OAuth authentication.
/// This transport supports the "Streamable HTTP" variant of MCP.
///
/// # Example
///
/// ```
/// use harness_locate::mcp::{HttpMcpServer, OAuthConfig};
/// use harness_locate::types::EnvValue;
///
/// let server = HttpMcpServer {
///     url: "https://api.example.com/mcp".to_string(),
///     headers: Default::default(),
///     oauth: Some(OAuthConfig {
///         client_id: Some("my-app".to_string()),
///         client_secret: Some(EnvValue::env("OAUTH_SECRET")),
///         scope: Some("read write".to_string()),
///     }),
///     enabled: true,
///     timeout_ms: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpMcpServer {
    /// The HTTP endpoint URL.
    pub url: String,

    /// HTTP headers to include in requests.
    ///
    /// Values can be plain strings or environment variable references.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, EnvValue>,

    /// OAuth configuration for authentication.
    ///
    /// If provided, the harness will handle OAuth token acquisition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oauth: Option<OAuthConfig>,

    /// Whether this server is enabled.
    ///
    /// Defaults to `true`. Disabled servers are skipped during loading.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Connection timeout in milliseconds.
    ///
    /// If not specified, harness-specific defaults apply.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
}

/// OAuth configuration for HTTP MCP servers.
///
/// All fields are optional to support different OAuth flows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    /// OAuth client ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,

    /// OAuth client secret.
    ///
    /// Can be an environment variable reference for security.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<EnvValue>,

    /// OAuth scope(s) to request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

/// Describes what MCP features a harness supports.
///
/// Different harnesses support different subsets of MCP configuration options.
/// Use [`McpCapabilities::for_kind`] to get capabilities for a specific harness.
///
/// # Extensibility
///
/// This struct is marked `#[non_exhaustive]` to allow adding new capability
/// fields in future versions without breaking changes. Use [`for_kind`] or
/// struct update syntax (`..Default::default()`) when constructing.
///
/// [`for_kind`]: McpCapabilities::for_kind
///
/// # Example
///
/// ```
/// use harness_locate::mcp::McpCapabilities;
/// use harness_locate::types::HarnessKind;
///
/// let caps = McpCapabilities::for_kind(HarnessKind::OpenCode);
/// assert!(caps.stdio);
/// assert!(caps.oauth);  // OpenCode supports OAuth
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[non_exhaustive]
pub struct McpCapabilities {
    /// Supports local stdio servers.
    pub stdio: bool,

    /// Supports SSE (Server-Sent Events) remote servers.
    pub sse: bool,

    /// Supports HTTP/Streamable HTTP remote servers.
    pub http: bool,

    /// Supports OAuth authentication for remote servers.
    pub oauth: bool,

    /// Supports per-server timeout configuration.
    pub timeout: bool,

    /// Supports enable/disable toggle per server.
    pub toggle: bool,

    /// Supports custom HTTP headers for remote servers.
    pub headers: bool,

    /// Supports working directory (cwd) for stdio servers.
    pub cwd: bool,
}

impl McpCapabilities {
    /// Returns the MCP capabilities for a specific harness kind.
    ///
    /// # Example
    ///
    /// ```
    /// use harness_locate::mcp::McpCapabilities;
    /// use harness_locate::types::HarnessKind;
    ///
    /// let caps = McpCapabilities::for_kind(HarnessKind::ClaudeCode);
    /// assert!(caps.stdio);
    /// assert!(caps.oauth);   // Claude Code supports OAuth via /mcp command
    /// ```
    #[must_use]
    pub fn for_kind(kind: HarnessKind) -> Self {
        match kind {
            HarnessKind::ClaudeCode => Self {
                stdio: true,
                sse: true,
                http: true,
                oauth: true,
                timeout: true,
                toggle: false,
                headers: true,
                cwd: false,
            },
            HarnessKind::OpenCode => Self {
                stdio: true,
                sse: true,
                http: true,
                oauth: true,
                timeout: true,
                toggle: true,
                headers: true,
                cwd: false,
            },
            HarnessKind::Goose => Self {
                stdio: true,
                sse: true,
                http: true,
                oauth: true,
                timeout: true,
                toggle: true,
                headers: true,
                cwd: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stdio_server_serialization_roundtrip() {
        let server = McpServer::Stdio(StdioMcpServer {
            command: "node".to_string(),
            args: vec!["server.js".to_string()],
            env: HashMap::new(),
            cwd: None,
            enabled: true,
            timeout_ms: None,
        });

        let json = serde_json::to_string(&server).unwrap();
        let parsed: McpServer = serde_json::from_str(&json).unwrap();

        if let McpServer::Stdio(s) = parsed {
            assert_eq!(s.command, "node");
            assert_eq!(s.args, vec!["server.js"]);
            assert!(s.enabled);
        } else {
            panic!("Expected Stdio variant");
        }
    }

    #[test]
    fn sse_server_serialization_roundtrip() {
        let server = McpServer::Sse(SseMcpServer {
            url: "https://example.com/sse".to_string(),
            headers: HashMap::new(),
            enabled: true,
            timeout_ms: Some(30000),
        });

        let json = serde_json::to_string(&server).unwrap();
        let parsed: McpServer = serde_json::from_str(&json).unwrap();

        if let McpServer::Sse(s) = parsed {
            assert_eq!(s.url, "https://example.com/sse");
            assert_eq!(s.timeout_ms, Some(30000));
        } else {
            panic!("Expected Sse variant");
        }
    }

    #[test]
    fn http_server_serialization_roundtrip() {
        let server = McpServer::Http(HttpMcpServer {
            url: "https://api.example.com/mcp".to_string(),
            headers: HashMap::new(),
            oauth: Some(OAuthConfig {
                client_id: Some("my-app".to_string()),
                client_secret: None,
                scope: Some("read".to_string()),
            }),
            enabled: true,
            timeout_ms: None,
        });

        let json = serde_json::to_string(&server).unwrap();
        let parsed: McpServer = serde_json::from_str(&json).unwrap();

        if let McpServer::Http(s) = parsed {
            assert_eq!(s.url, "https://api.example.com/mcp");
            assert!(s.oauth.is_some());
            let oauth = s.oauth.unwrap();
            assert_eq!(oauth.client_id, Some("my-app".to_string()));
            assert_eq!(oauth.scope, Some("read".to_string()));
        } else {
            panic!("Expected Http variant");
        }
    }

    #[test]
    fn oauth_config_serialization_roundtrip() {
        let config = OAuthConfig {
            client_id: Some("test-client".to_string()),
            client_secret: Some(EnvValue::env("SECRET_VAR")),
            scope: Some("read write".to_string()),
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: OAuthConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.client_id, Some("test-client".to_string()));
        assert_eq!(parsed.client_secret, Some(EnvValue::env("SECRET_VAR")));
        assert_eq!(parsed.scope, Some("read write".to_string()));
    }

    #[test]
    fn enabled_defaults_to_true() {
        // JSON without 'enabled' field
        let json = r#"{"transport":"stdio","command":"test"}"#;
        let parsed: McpServer = serde_json::from_str(json).unwrap();

        if let McpServer::Stdio(s) = parsed {
            assert!(s.enabled, "enabled should default to true");
        } else {
            panic!("Expected Stdio variant");
        }
    }

    #[test]
    fn empty_collections_not_serialized() {
        let server = StdioMcpServer {
            command: "test".to_string(),
            args: vec![],
            env: HashMap::new(),
            cwd: None,
            enabled: true,
            timeout_ms: None,
        };

        let json = serde_json::to_string(&server).unwrap();

        // Empty args, env, cwd, timeout_ms should not appear in JSON
        assert!(!json.contains("args"));
        assert!(!json.contains("env"));
        assert!(!json.contains("cwd"));
        assert!(!json.contains("timeout_ms"));
    }

    #[test]
    fn env_values_in_server_config() {
        let mut env = HashMap::new();
        env.insert("API_KEY".to_string(), EnvValue::env("MY_API_KEY"));
        env.insert("DEBUG".to_string(), EnvValue::plain("true"));

        let server = StdioMcpServer {
            command: "server".to_string(),
            args: vec![],
            env,
            cwd: None,
            enabled: true,
            timeout_ms: None,
        };

        let json = serde_json::to_string(&server).unwrap();
        let parsed: StdioMcpServer = serde_json::from_str(&json).unwrap();

        assert_eq!(
            parsed.env.get("API_KEY"),
            Some(&EnvValue::env("MY_API_KEY"))
        );
        assert_eq!(parsed.env.get("DEBUG"), Some(&EnvValue::plain("true")));
    }

    #[test]
    fn transport_tag_in_json() {
        let stdio = McpServer::Stdio(StdioMcpServer {
            command: "test".to_string(),
            args: vec![],
            env: HashMap::new(),
            cwd: None,
            enabled: true,
            timeout_ms: None,
        });

        let sse = McpServer::Sse(SseMcpServer {
            url: "http://example.com".to_string(),
            headers: HashMap::new(),
            enabled: true,
            timeout_ms: None,
        });

        let http = McpServer::Http(HttpMcpServer {
            url: "http://example.com".to_string(),
            headers: HashMap::new(),
            oauth: None,
            enabled: true,
            timeout_ms: None,
        });

        let stdio_json = serde_json::to_string(&stdio).unwrap();
        let sse_json = serde_json::to_string(&sse).unwrap();
        let http_json = serde_json::to_string(&http).unwrap();

        assert!(stdio_json.contains(r#""transport":"stdio""#));
        assert!(sse_json.contains(r#""transport":"sse""#));
        assert!(http_json.contains(r#""transport":"http""#));
    }

    #[test]
    fn mcp_capabilities_for_claude_code() {
        let caps = McpCapabilities::for_kind(HarnessKind::ClaudeCode);
        assert!(caps.stdio);
        assert!(caps.sse);
        assert!(caps.http);
        assert!(caps.oauth);
        assert!(caps.timeout);
        assert!(!caps.toggle);
        assert!(caps.headers);
        assert!(!caps.cwd);
    }

    #[test]
    fn mcp_capabilities_for_opencode() {
        let caps = McpCapabilities::for_kind(HarnessKind::OpenCode);
        assert!(caps.stdio);
        assert!(caps.sse);
        assert!(caps.http);
        assert!(caps.oauth);
        assert!(caps.timeout);
        assert!(caps.toggle);
        assert!(caps.headers);
        assert!(!caps.cwd);
    }

    #[test]
    fn mcp_capabilities_for_goose() {
        let caps = McpCapabilities::for_kind(HarnessKind::Goose);
        assert!(caps.stdio);
        assert!(caps.sse);
        assert!(caps.http);
        assert!(caps.oauth);
        assert!(caps.timeout);
        assert!(caps.toggle);
        assert!(caps.headers);
        assert!(!caps.cwd);
    }

    #[test]
    fn mcp_capabilities_serialization() {
        let caps = McpCapabilities::for_kind(HarnessKind::OpenCode);
        let json = serde_json::to_string(&caps).unwrap();
        assert!(json.contains(r#""oauth":true"#));
        assert!(json.contains(r#""stdio":true"#));
    }

    #[test]
    fn mcp_capabilities_default_is_all_false() {
        let caps = McpCapabilities::default();
        assert!(!caps.stdio);
        assert!(!caps.sse);
        assert!(!caps.http);
        assert!(!caps.oauth);
        assert!(!caps.timeout);
        assert!(!caps.toggle);
        assert!(!caps.headers);
        assert!(!caps.cwd);
    }
}
