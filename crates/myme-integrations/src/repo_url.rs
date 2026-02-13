//! Normalize GitHub URLs to canonical `owner/repo` form for matching.

/// Normalizes a GitHub URL to `owner/repo`.
///
/// Supports:
/// - `https://github.com/owner/repo`
/// - `https://github.com/owner/repo.git`
/// - `http://github.com/owner/repo`
/// - `git@github.com:owner/repo.git`
/// - `git@github.com:owner/repo`
///
/// Returns `None` if the URL does not look like a GitHub URL.
pub fn normalize_github_url(url: &str) -> Option<String> {
    let url = url.trim();
    if url.is_empty() {
        return None;
    }

    // git@github.com:owner/repo[.git]
    if let Some(rest) = url.strip_prefix("git@github.com:") {
        let rest = rest.strip_suffix(".git").unwrap_or(rest);
        if rest.is_empty() || rest.contains("..") {
            return None;
        }
        let parts: Vec<&str> = rest.splitn(2, '/').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return None;
        }
        return Some(format!("{}/{}", parts[0], parts[1]));
    }

    // https?://github.com/owner/repo[.git][/...]
    if url.starts_with("https://github.com/") || url.starts_with("http://github.com/") {
        let prefix = if url.starts_with("https://") {
            "https://github.com/"
        } else {
            "http://github.com/"
        };
        let rest = url.strip_prefix(prefix).unwrap_or(url);
        let path = rest.split('?').next().unwrap_or(rest);
        let path = path.split('#').next().unwrap_or(path);
        let path = path.strip_suffix(".git").unwrap_or(path);
        let segments: Vec<&str> = path.split('/').collect();
        if segments.len() < 2 || segments[0].is_empty() || segments[1].is_empty() {
            return None;
        }
        return Some(format!("{}/{}", segments[0], segments[1]));
    }

    None
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn test_https_without_git() {
        assert_eq!(
            normalize_github_url("https://github.com/foo/bar"),
            Some("foo/bar".to_string())
        );
    }

    #[test]
    fn test_https_with_git() {
        assert_eq!(
            normalize_github_url("https://github.com/foo/bar.git"),
            Some("foo/bar".to_string())
        );
    }

    #[test]
    fn test_http() {
        assert_eq!(
            normalize_github_url("http://github.com/owner/repo"),
            Some("owner/repo".to_string())
        );
    }

    #[test]
    fn test_ssh_with_git() {
        assert_eq!(
            normalize_github_url("git@github.com:foo/bar.git"),
            Some("foo/bar".to_string())
        );
    }

    #[test]
    fn test_ssh_without_git() {
        assert_eq!(
            normalize_github_url("git@github.com:foo/bar"),
            Some("foo/bar".to_string())
        );
    }

    #[test]
    fn test_empty() {
        assert_eq!(normalize_github_url(""), None);
        assert_eq!(normalize_github_url("   "), None);
    }

    #[test]
    fn test_non_github() {
        assert_eq!(normalize_github_url("https://gitlab.com/a/b"), None);
        assert_eq!(normalize_github_url("https://example.com/"), None);
    }

    #[test]
    fn test_trimmed() {
        assert_eq!(
            normalize_github_url("  https://github.com/x/y  "),
            Some("x/y".to_string())
        );
    }
}
