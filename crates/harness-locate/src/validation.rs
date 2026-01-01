//! MCP server configuration validation.
//!
//! This module provides validation for [`McpServer`] configurations,
//! checking for structural issues like empty commands, invalid URLs,
//! excessive timeouts, and suspicious environment variable names.
//!
//! Unlike the fail-fast error handling elsewhere in this crate,
//! validation collects all issues found, allowing callers to see
//! the complete picture rather than stopping at the first problem.
//!
//! # Example
//!
//! ```
//! use harness_locate::mcp::{McpServer, StdioMcpServer};
//! use harness_locate::validation::{validate_mcp_server, Severity};
//!
//! let server = McpServer::Stdio(StdioMcpServer {
//!     command: String::new(), // Empty command - will be flagged
//!     args: vec![],
//!     env: std::collections::HashMap::new(),
//!     cwd: None,
//!     enabled: true,
//!     timeout_ms: None,
//! });
//!
//! let issues = validate_mcp_server(&server);
//! assert!(!issues.is_empty());
//! assert!(issues.iter().any(|i| i.severity == Severity::Error));
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::mcp::{HttpMcpServer, McpCapabilities, McpServer, SseMcpServer, StdioMcpServer};
use crate::types::{EnvValue, HarnessKind};

// Issue code constants for machine-readable classification.

/// Empty command in stdio transport.
pub const CODE_EMPTY_COMMAND: &str = "stdio.command.empty";

/// URL failed to parse.
pub const CODE_INVALID_URL: &str = "url.invalid";

/// URL has non-http(s) scheme.
pub const CODE_INVALID_SCHEME: &str = "url.scheme.invalid";

/// Timeout exceeds recommended maximum.
pub const CODE_TIMEOUT_EXCESSIVE: &str = "timeout.excessive";

/// Environment variable name suggests sensitive data.
pub const CODE_SUSPICIOUS_ENV: &str = "env.suspicious_name";

/// Working directory (cwd) not supported by harness.
pub const CODE_CWD_UNSUPPORTED: &str = "harness.cwd.unsupported";

/// Toggle (enabled field) not supported by harness.
pub const CODE_TOGGLE_UNSUPPORTED: &str = "harness.toggle.unsupported";

/// SSE transport deprecated for this harness (prefer HTTP).
pub const CODE_SSE_DEPRECATED: &str = "harness.transport.sse_deprecated";

/// Severity level for validation issues.
///
/// Determines how the issue should be treated by callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Severity {
    /// Critical issue that will likely cause the server to fail.
    ///
    /// Examples: empty command, unparseable URL.
    Error,

    /// Non-critical issue that may cause problems or is worth reviewing.
    ///
    /// Examples: very long timeout, suspicious environment variable name.
    Warning,
}

/// A validation issue found in an MCP server configuration.
///
/// Issues are collected by [`validate_mcp_server`] and returned as a `Vec`.
/// An empty result means the configuration passed all checks.
///
/// # Extensibility
///
/// This struct is marked `#[non_exhaustive]` to allow adding new fields
/// in future versions without breaking changes. Use the constructor
/// methods [`ValidationIssue::error`] and [`ValidationIssue::warning`]
/// rather than constructing directly.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Severity of the issue.
    pub severity: Severity,

    /// The field path where the issue was found (e.g., "command", "url", "env.SECRET_KEY").
    pub field: String,

    /// Human-readable description of the issue.
    pub message: String,

    /// Machine-readable issue code for programmatic filtering.
    ///
    /// See the `CODE_*` constants in this module.
    pub code: Option<&'static str>,
}

impl ValidationIssue {
    /// Creates an error-level validation issue.
    ///
    /// # Arguments
    ///
    /// * `field` - The field path where the issue was found
    /// * `message` - Human-readable description
    /// * `code` - Optional machine-readable code
    #[must_use]
    pub fn error(
        field: impl Into<String>,
        message: impl Into<String>,
        code: Option<&'static str>,
    ) -> Self {
        Self {
            severity: Severity::Error,
            field: field.into(),
            message: message.into(),
            code,
        }
    }

    /// Creates a warning-level validation issue.
    ///
    /// # Arguments
    ///
    /// * `field` - The field path where the issue was found
    /// * `message` - Human-readable description
    /// * `code` - Optional machine-readable code
    #[must_use]
    pub fn warning(
        field: impl Into<String>,
        message: impl Into<String>,
        code: Option<&'static str>,
    ) -> Self {
        Self {
            severity: Severity::Warning,
            field: field.into(),
            message: message.into(),
            code,
        }
    }
}

/// Maximum recommended timeout in milliseconds (5 minutes).
const MAX_RECOMMENDED_TIMEOUT_MS: u64 = 300_000;

/// Patterns that suggest an environment variable contains sensitive data.
///
/// These are checked case-insensitively against variable names.
const SUSPICIOUS_ENV_PATTERNS: &[&str] = &[
    "PASSWORD",
    "PASSWD",
    "SECRET",
    "TOKEN",
    "API_KEY",
    "PRIVATE_KEY",
    "ACCESS_KEY",
    "CREDENTIAL",
    "BEARER",
    "AUTH",
];

/// Validates an MCP server configuration.
///
/// Checks for structural issues like empty commands, invalid URLs,
/// excessive timeouts, and suspicious environment variable names.
/// Returns all issues found, allowing callers to see the complete picture.
///
/// # Arguments
///
/// * `server` - The MCP server configuration to validate
///
/// # Returns
///
/// A vector of validation issues. An empty vector means no issues were found.
///
/// # Example
///
/// ```
/// use harness_locate::mcp::{McpServer, StdioMcpServer};
/// use harness_locate::validation::validate_mcp_server;
///
/// let server = McpServer::Stdio(StdioMcpServer {
///     command: "node".to_string(),
///     args: vec!["server.js".to_string()],
///     env: std::collections::HashMap::new(),
///     cwd: None,
///     enabled: true,
///     timeout_ms: None,
/// });
///
/// let issues = validate_mcp_server(&server);
/// assert!(issues.is_empty()); // Valid configuration
/// ```
#[must_use]
pub fn validate_mcp_server(server: &McpServer) -> Vec<ValidationIssue> {
    match server {
        McpServer::Stdio(s) => validate_stdio(s),
        McpServer::Sse(s) => validate_sse(s),
        McpServer::Http(s) => validate_http(s),
    }
}

/// Validates an MCP server configuration for a specific harness.
///
/// Combines base validation with harness-specific capability checks.
/// Returns all issues found, including structural problems and harness incompatibilities.
#[must_use]
pub fn validate_for_harness(server: &McpServer, kind: HarnessKind) -> Vec<ValidationIssue> {
    let mut issues = validate_mcp_server(server);
    let caps = McpCapabilities::for_kind(kind);
    let harness_name = kind.as_str();

    match server {
        McpServer::Stdio(s) => {
            if s.cwd.is_some() && !caps.cwd {
                issues.push(ValidationIssue::error(
                    "cwd",
                    format!("Working directory not supported by {harness_name}"),
                    Some(CODE_CWD_UNSUPPORTED),
                ));
            }
            if !s.enabled && !caps.toggle {
                issues.push(ValidationIssue::warning(
                    "enabled",
                    format!("{harness_name} ignores the enabled field; server will always run"),
                    Some(CODE_TOGGLE_UNSUPPORTED),
                ));
            }
        }
        McpServer::Sse(s) => {
            if kind == HarnessKind::ClaudeCode {
                issues.push(ValidationIssue::warning(
                    "transport",
                    "SSE transport works but HTTP is preferred for Claude Code",
                    Some(CODE_SSE_DEPRECATED),
                ));
            }
            if !s.enabled && !caps.toggle {
                issues.push(ValidationIssue::warning(
                    "enabled",
                    format!("{harness_name} ignores the enabled field; server will always run"),
                    Some(CODE_TOGGLE_UNSUPPORTED),
                ));
            }
        }
        McpServer::Http(s) => {
            if !s.enabled && !caps.toggle {
                issues.push(ValidationIssue::warning(
                    "enabled",
                    format!("{harness_name} ignores the enabled field; server will always run"),
                    Some(CODE_TOGGLE_UNSUPPORTED),
                ));
            }
        }
    }

    issues
}

fn validate_stdio(server: &StdioMcpServer) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if server.command.trim().is_empty() {
        issues.push(ValidationIssue::error(
            "command",
            "Command must not be empty",
            Some(CODE_EMPTY_COMMAND),
        ));
    }

    issues.extend(validate_timeout(server.timeout_ms, "timeout_ms"));
    issues.extend(validate_env(&server.env, "env"));
    issues
}

fn validate_sse(server: &SseMcpServer) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    issues.extend(validate_url(&server.url, "url"));
    issues.extend(validate_timeout(server.timeout_ms, "timeout_ms"));
    issues.extend(validate_env(&server.headers, "headers"));
    issues
}

fn validate_http(server: &HttpMcpServer) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    issues.extend(validate_url(&server.url, "url"));
    issues.extend(validate_timeout(server.timeout_ms, "timeout_ms"));
    issues.extend(validate_env(&server.headers, "headers"));
    issues
}
fn validate_url(url: &str, field: &str) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    match Url::parse(url) {
        Ok(parsed) => {
            let scheme = parsed.scheme();
            if scheme != "http" && scheme != "https" {
                issues.push(ValidationIssue::error(
                    field,
                    format!("URL scheme must be http or https, got '{scheme}'"),
                    Some(CODE_INVALID_SCHEME),
                ));
            }
        }
        Err(e) => {
            issues.push(ValidationIssue::error(
                field,
                format!("Invalid URL: {e}"),
                Some(CODE_INVALID_URL),
            ));
        }
    }

    issues
}

fn validate_timeout(timeout_ms: Option<u64>, field: &str) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if let Some(ms) = timeout_ms
        && ms > MAX_RECOMMENDED_TIMEOUT_MS
    {
        issues.push(ValidationIssue::warning(
            field,
            format!(
                "Timeout of {}ms exceeds recommended maximum of {}ms (5 minutes)",
                ms, MAX_RECOMMENDED_TIMEOUT_MS
            ),
            Some(CODE_TIMEOUT_EXCESSIVE),
        ));
    }

    issues
}

fn validate_env(env: &HashMap<String, EnvValue>, field_prefix: &str) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    for key in env.keys() {
        let upper = key.to_uppercase();
        for pattern in SUSPICIOUS_ENV_PATTERNS {
            if upper.contains(pattern) {
                issues.push(ValidationIssue::warning(
                    format!("{field_prefix}.{key}"),
                    format!(
                        "Variable name '{key}' suggests sensitive data; \
                         consider using environment variable references"
                    ),
                    Some(CODE_SUSPICIOUS_ENV),
                ));
                break;
            }
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_stdio(command: &str) -> McpServer {
        McpServer::Stdio(StdioMcpServer {
            command: command.to_string(),
            args: vec![],
            env: HashMap::new(),
            cwd: None,
            enabled: true,
            timeout_ms: None,
        })
    }

    fn make_sse(url: &str) -> McpServer {
        McpServer::Sse(SseMcpServer {
            url: url.to_string(),
            headers: HashMap::new(),
            enabled: true,
            timeout_ms: None,
        })
    }

    fn make_http(url: &str) -> McpServer {
        McpServer::Http(HttpMcpServer {
            url: url.to_string(),
            headers: HashMap::new(),
            oauth: None,
            enabled: true,
            timeout_ms: None,
        })
    }

    #[test]
    fn empty_command_returns_error() {
        let server = make_stdio("");
        let issues = validate_mcp_server(&server);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Error);
        assert_eq!(issues[0].field, "command");
        assert_eq!(issues[0].code, Some(CODE_EMPTY_COMMAND));
    }

    #[test]
    fn valid_command_returns_no_issues() {
        let server = make_stdio("node");
        let issues = validate_mcp_server(&server);

        assert!(issues.is_empty());
    }

    #[test]
    fn invalid_url_returns_error() {
        let server = make_sse("not-a-valid-url");
        let issues = validate_mcp_server(&server);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Error);
        assert_eq!(issues[0].field, "url");
        assert_eq!(issues[0].code, Some(CODE_INVALID_URL));
    }

    #[test]
    fn valid_https_url_returns_no_issues() {
        let server = make_http("https://example.com/mcp");
        let issues = validate_mcp_server(&server);

        assert!(issues.is_empty());
    }

    #[test]
    fn ftp_scheme_returns_error() {
        let server = make_sse("ftp://files.example.com");
        let issues = validate_mcp_server(&server);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Error);
        assert_eq!(issues[0].field, "url");
        assert_eq!(issues[0].code, Some(CODE_INVALID_SCHEME));
        assert!(issues[0].message.contains("ftp"));
    }

    #[test]
    fn excessive_timeout_returns_warning() {
        let server = McpServer::Stdio(StdioMcpServer {
            command: "node".to_string(),
            args: vec![],
            env: HashMap::new(),
            cwd: None,
            enabled: true,
            timeout_ms: Some(600_000),
        });
        let issues = validate_mcp_server(&server);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Warning);
        assert_eq!(issues[0].field, "timeout_ms");
        assert_eq!(issues[0].code, Some(CODE_TIMEOUT_EXCESSIVE));
    }

    #[test]
    fn normal_timeout_returns_no_issues() {
        let server = McpServer::Stdio(StdioMcpServer {
            command: "node".to_string(),
            args: vec![],
            env: HashMap::new(),
            cwd: None,
            enabled: true,
            timeout_ms: Some(30_000),
        });
        let issues = validate_mcp_server(&server);

        assert!(issues.is_empty());
    }

    #[test]
    fn suspicious_env_name_returns_warning() {
        let mut env = HashMap::new();
        env.insert("DB_PASSWORD".to_string(), EnvValue::plain("secret123"));

        let server = McpServer::Stdio(StdioMcpServer {
            command: "node".to_string(),
            args: vec![],
            env,
            cwd: None,
            enabled: true,
            timeout_ms: None,
        });
        let issues = validate_mcp_server(&server);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Warning);
        assert_eq!(issues[0].field, "env.DB_PASSWORD");
        assert_eq!(issues[0].code, Some(CODE_SUSPICIOUS_ENV));
    }

    #[test]
    fn normal_env_name_returns_no_issues() {
        let mut env = HashMap::new();
        env.insert("NODE_ENV".to_string(), EnvValue::plain("production"));
        env.insert("PORT".to_string(), EnvValue::plain("3000"));

        let server = McpServer::Stdio(StdioMcpServer {
            command: "node".to_string(),
            args: vec![],
            env,
            cwd: None,
            enabled: true,
            timeout_ms: None,
        });
        let issues = validate_mcp_server(&server);

        assert!(issues.is_empty());
    }

    #[test]
    fn multiple_issues_collected() {
        let mut env = HashMap::new();
        env.insert("API_TOKEN".to_string(), EnvValue::plain("tok_123"));

        let server = McpServer::Stdio(StdioMcpServer {
            command: "".to_string(),
            args: vec![],
            env,
            cwd: None,
            enabled: true,
            timeout_ms: Some(600_000),
        });
        let issues = validate_mcp_server(&server);

        assert_eq!(issues.len(), 3);
        let error_count = issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .count();
        let warning_count = issues
            .iter()
            .filter(|i| i.severity == Severity::Warning)
            .count();
        assert_eq!(error_count, 1);
        assert_eq!(warning_count, 2);
        assert!(issues.iter().any(|i| i.code == Some(CODE_EMPTY_COMMAND)));
        assert!(
            issues
                .iter()
                .any(|i| i.code == Some(CODE_TIMEOUT_EXCESSIVE))
        );
        assert!(issues.iter().any(|i| i.code == Some(CODE_SUSPICIOUS_ENV)));
    }

    #[test]
    fn valid_config_returns_empty_vec() {
        let mut env = HashMap::new();
        env.insert("NODE_ENV".to_string(), EnvValue::plain("production"));

        let server = McpServer::Stdio(StdioMcpServer {
            command: "node".to_string(),
            args: vec!["server.js".to_string()],
            env,
            cwd: None,
            enabled: true,
            timeout_ms: Some(30_000),
        });
        let issues = validate_mcp_server(&server);

        assert!(issues.is_empty());
    }

    // Harness-specific validation tests

    #[test]
    fn cwd_on_any_harness_returns_error() {
        let server = McpServer::Stdio(StdioMcpServer {
            command: "node".to_string(),
            args: vec![],
            env: HashMap::new(),
            cwd: Some(std::path::PathBuf::from("/tmp")),
            enabled: true,
            timeout_ms: None,
        });

        for kind in HarnessKind::ALL {
            let issues = validate_for_harness(&server, *kind);
            assert!(issues.iter().any(|i| i.code == Some(CODE_CWD_UNSUPPORTED)));
        }
    }

    #[test]
    fn disabled_on_claude_code_returns_warning() {
        let server = McpServer::Stdio(StdioMcpServer {
            command: "node".to_string(),
            args: vec![],
            env: HashMap::new(),
            cwd: None,
            enabled: false,
            timeout_ms: None,
        });

        let issues = validate_for_harness(&server, HarnessKind::ClaudeCode);
        assert!(
            issues
                .iter()
                .any(|i| i.code == Some(CODE_TOGGLE_UNSUPPORTED))
        );
    }

    #[test]
    fn disabled_on_opencode_returns_no_warning() {
        let server = McpServer::Stdio(StdioMcpServer {
            command: "node".to_string(),
            args: vec![],
            env: HashMap::new(),
            cwd: None,
            enabled: false,
            timeout_ms: None,
        });

        let issues = validate_for_harness(&server, HarnessKind::OpenCode);
        assert!(
            !issues
                .iter()
                .any(|i| i.code == Some(CODE_TOGGLE_UNSUPPORTED))
        );
    }

    #[test]
    fn sse_on_claude_code_returns_warning() {
        let server = McpServer::Sse(SseMcpServer {
            url: "https://example.com/sse".to_string(),
            headers: HashMap::new(),
            enabled: true,
            timeout_ms: None,
        });

        let issues = validate_for_harness(&server, HarnessKind::ClaudeCode);
        assert!(issues.iter().any(|i| i.code == Some(CODE_SSE_DEPRECATED)));
    }

    #[test]
    fn sse_on_opencode_returns_no_warning() {
        let server = McpServer::Sse(SseMcpServer {
            url: "https://example.com/sse".to_string(),
            headers: HashMap::new(),
            enabled: true,
            timeout_ms: None,
        });

        let issues = validate_for_harness(&server, HarnessKind::OpenCode);
        assert!(!issues.iter().any(|i| i.code == Some(CODE_SSE_DEPRECATED)));
    }

    #[test]
    fn validate_for_harness_includes_base_validation() {
        let server = McpServer::Stdio(StdioMcpServer {
            command: "".to_string(),
            args: vec![],
            env: HashMap::new(),
            cwd: Some(std::path::PathBuf::from("/tmp")),
            enabled: true,
            timeout_ms: None,
        });

        let issues = validate_for_harness(&server, HarnessKind::ClaudeCode);
        assert!(issues.iter().any(|i| i.code == Some(CODE_EMPTY_COMMAND)));
        assert!(issues.iter().any(|i| i.code == Some(CODE_CWD_UNSUPPORTED)));
    }
}
