//! Skill file parsing utilities.

use crate::Result;

/// Parsed frontmatter result.
#[derive(Debug, Clone, PartialEq)]
pub struct Frontmatter<'a> {
    /// Parsed YAML frontmatter, if present.
    pub yaml: Option<serde_yaml::Value>,
    /// The markdown body after the frontmatter.
    pub body: &'a str,
}

/// Parse YAML frontmatter from markdown content.
///
/// # Errors
///
/// Returns `Error::YamlParse` if frontmatter exists but contains invalid YAML.
pub fn parse_frontmatter(content: &str) -> Result<Frontmatter<'_>> {
    let (opener, line_ending) = if content.starts_with("---\r\n") {
        ("---\r\n", "\r\n")
    } else if content.starts_with("---\n") {
        ("---\n", "\n")
    } else {
        return Ok(Frontmatter {
            yaml: None,
            body: content,
        });
    };

    let after_opener = &content[opener.len()..];
    let empty_closer = format!("---{line_ending}");
    let closer = format!("{line_ending}---{line_ending}");
    let closer_eof = format!("{line_ending}---");

    let (yaml_content, body) = if after_opener.starts_with(&empty_closer) {
        ("", &after_opener[empty_closer.len()..])
    } else if let Some(pos) = after_opener.find(&closer) {
        (&after_opener[..pos], &after_opener[pos + closer.len()..])
    } else if after_opener.ends_with(&closer_eof) {
        (&after_opener[..after_opener.len() - closer_eof.len()], "")
    } else if after_opener == "---" {
        ("", "")
    } else {
        return Ok(Frontmatter {
            yaml: None,
            body: content,
        });
    };

    let yaml_value: serde_yaml::Value = serde_yaml::from_str(yaml_content)?;
    Ok(Frontmatter {
        yaml: Some(yaml_value),
        body,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_standard_frontmatter() {
        let content = "---\nname: test\nversion: 1\n---\n# Body\n";
        let result = parse_frontmatter(content).unwrap();
        assert!(result.yaml.is_some());
        let yaml = result.yaml.unwrap();
        assert_eq!(yaml["name"], "test");
        assert_eq!(yaml["version"], 1);
        assert_eq!(result.body, "# Body\n");
    }

    #[test]
    fn returns_none_without_frontmatter() {
        let content = "# Just Markdown\nNo frontmatter here.";
        let result = parse_frontmatter(content).unwrap();
        assert!(result.yaml.is_none());
        assert_eq!(result.body, content);
    }

    #[test]
    fn handles_empty_frontmatter() {
        let content = "---\n---\nBody content";
        let result = parse_frontmatter(content).unwrap();
        assert!(result.yaml.is_some());
        assert_eq!(result.yaml.unwrap(), serde_yaml::Value::Null);
        assert_eq!(result.body, "Body content");
    }

    #[test]
    fn returns_error_for_malformed_yaml() {
        let content = "---\ninvalid: yaml: content:\n---\nBody";
        let result = parse_frontmatter(content);
        assert!(result.is_err());
    }

    #[test]
    fn handles_crlf_line_endings() {
        let content = "---\r\nname: test\r\n---\r\nBody";
        let result = parse_frontmatter(content).unwrap();
        assert!(result.yaml.is_some());
        assert_eq!(result.yaml.unwrap()["name"], "test");
        assert_eq!(result.body, "Body");
    }

    #[test]
    fn preserves_horizontal_rules_in_body() {
        let content = "---\nname: test\n---\n# Title\n\n---\n\nMore content";
        let result = parse_frontmatter(content).unwrap();
        assert!(result.yaml.is_some());
        assert!(result.body.contains("---"));
        assert!(result.body.contains("More content"));
    }

    #[test]
    fn handles_empty_body() {
        let content = "---\nname: test\n---\n";
        let result = parse_frontmatter(content).unwrap();
        assert!(result.yaml.is_some());
        assert_eq!(result.body, "");
    }

    #[test]
    fn handles_frontmatter_at_eof() {
        let content = "---\nname: test\n---";
        let result = parse_frontmatter(content).unwrap();
        assert!(result.yaml.is_some());
        assert_eq!(result.body, "");
    }
}
