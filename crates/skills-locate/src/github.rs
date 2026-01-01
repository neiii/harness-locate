use crate::{Error, Result};

const RAW_GITHUB_BASE: &str = "https://raw.githubusercontent.com";
const GITHUB_ARCHIVE_BASE: &str = "https://github.com";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHubRef {
    pub owner: String,
    pub repo: String,
    pub git_ref: Option<String>,
    pub path: Option<String>,
}

impl GitHubRef {
    pub fn parse(source: &str) -> Result<Self> {
        let source = source
            .trim()
            .trim_start_matches("github:")
            .trim_start_matches("https://github.com/")
            .trim_start_matches("http://github.com/");

        let parts: Vec<&str> = source.splitn(2, '@').collect();
        let (path_part, git_ref) = if parts.len() == 2 {
            (parts[0], Some(parts[1].to_string()))
        } else {
            (parts[0], None)
        };

        let segments: Vec<&str> = path_part.split('/').collect();
        if segments.len() < 2 {
            return Err(Error::InvalidSource(format!(
                "Invalid GitHub source: expected owner/repo, got '{source}'"
            )));
        }

        let owner = segments[0].to_string();
        let repo = segments[1].to_string();
        let path = if segments.len() > 2 {
            Some(segments[2..].join("/"))
        } else {
            None
        };

        if owner.is_empty() || repo.is_empty() {
            return Err(Error::InvalidSource(
                "Owner and repo cannot be empty".to_string(),
            ));
        }

        Ok(Self {
            owner,
            repo,
            git_ref,
            path,
        })
    }

    pub fn raw_file_url(&self, file_path: &str) -> String {
        let git_ref = self.git_ref.as_deref().unwrap_or("HEAD");
        let base_path = self.path.as_deref().unwrap_or("");
        let full_path = if base_path.is_empty() {
            file_path.to_string()
        } else {
            format!("{base_path}/{file_path}")
        };
        format!(
            "{RAW_GITHUB_BASE}/{}/{}/{git_ref}/{full_path}",
            self.owner, self.repo
        )
    }

    pub fn archive_url(&self) -> String {
        let git_ref = self.git_ref.as_deref().unwrap_or("HEAD");
        format!(
            "{GITHUB_ARCHIVE_BASE}/{}/{}/archive/{git_ref}.zip",
            self.owner, self.repo
        )
    }

    pub fn tree_url(&self) -> String {
        let git_ref = self.git_ref.as_deref().unwrap_or("HEAD");
        let path_suffix = self
            .path
            .as_ref()
            .map(|p| format!("/{p}"))
            .unwrap_or_default();
        format!(
            "{GITHUB_ARCHIVE_BASE}/{}/{}/tree/{git_ref}{path_suffix}",
            self.owner, self.repo
        )
    }
}

pub fn resolve_github_source(source: &str) -> Result<GitHubRef> {
    GitHubRef::parse(source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_owner_repo() {
        let gh = GitHubRef::parse("anthropics/claude-code-plugins").unwrap();
        assert_eq!(gh.owner, "anthropics");
        assert_eq!(gh.repo, "claude-code-plugins");
        assert_eq!(gh.git_ref, None);
        assert_eq!(gh.path, None);
    }

    #[test]
    fn parse_with_ref() {
        let gh = GitHubRef::parse("owner/repo@v1.0.0").unwrap();
        assert_eq!(gh.owner, "owner");
        assert_eq!(gh.repo, "repo");
        assert_eq!(gh.git_ref, Some("v1.0.0".to_string()));
    }

    #[test]
    fn parse_with_path() {
        let gh = GitHubRef::parse("owner/repo/plugins/skill-a").unwrap();
        assert_eq!(gh.owner, "owner");
        assert_eq!(gh.repo, "repo");
        assert_eq!(gh.path, Some("plugins/skill-a".to_string()));
    }

    #[test]
    fn parse_with_path_and_ref() {
        let gh = GitHubRef::parse("owner/repo/plugins/skill-a@main").unwrap();
        assert_eq!(gh.owner, "owner");
        assert_eq!(gh.repo, "repo");
        assert_eq!(gh.path, Some("plugins/skill-a".to_string()));
        assert_eq!(gh.git_ref, Some("main".to_string()));
    }

    #[test]
    fn parse_github_prefix() {
        let gh = GitHubRef::parse("github:owner/repo").unwrap();
        assert_eq!(gh.owner, "owner");
        assert_eq!(gh.repo, "repo");
    }

    #[test]
    fn parse_full_url() {
        let gh = GitHubRef::parse("https://github.com/owner/repo").unwrap();
        assert_eq!(gh.owner, "owner");
        assert_eq!(gh.repo, "repo");
    }

    #[test]
    fn raw_file_url_default_ref() {
        let gh = GitHubRef::parse("owner/repo").unwrap();
        assert_eq!(
            gh.raw_file_url("plugin.json"),
            "https://raw.githubusercontent.com/owner/repo/HEAD/plugin.json"
        );
    }

    #[test]
    fn raw_file_url_with_ref() {
        let gh = GitHubRef::parse("owner/repo@v1.0.0").unwrap();
        assert_eq!(
            gh.raw_file_url("plugin.json"),
            "https://raw.githubusercontent.com/owner/repo/v1.0.0/plugin.json"
        );
    }

    #[test]
    fn archive_url_generation() {
        let gh = GitHubRef::parse("owner/repo@main").unwrap();
        assert_eq!(
            gh.archive_url(),
            "https://github.com/owner/repo/archive/main.zip"
        );
    }

    #[test]
    fn invalid_source_single_segment() {
        let result = GitHubRef::parse("just-owner");
        assert!(result.is_err());
    }
}
