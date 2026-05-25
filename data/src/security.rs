use std::{
    path::{Component, Path, PathBuf},
    time::Duration,
};

use reqwest::{Url, blocking::Client};

const HTTP_TIMEOUT_SECONDS: u64 = 30;

pub fn validate_plugin_id(plugin_id: &str) -> Result<(), String> {
    if plugin_id.is_empty() || plugin_id == "." || plugin_id == ".." {
        return Err(format!("invalid plugin id: {plugin_id:?}"));
    }

    if plugin_id.len() > 128 {
        return Err(format!("plugin id is too long: {plugin_id:?}"));
    }

    if !plugin_id
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '.' || ch == '-' || ch == '_')
    {
        return Err(format!(
            "plugin id contains unsafe path characters: {plugin_id:?}"
        ));
    }

    Ok(())
}

pub fn validate_github_repo_slug(repo: &str) -> Result<(), String> {
    let parts = repo.split('/').collect::<Vec<_>>();
    if parts.len() != 2 {
        return Err(format!("GitHub repo must be owner/name: {repo:?}"));
    }

    for part in parts {
        if part.is_empty() || part == "." || part == ".." || part.len() > 100 {
            return Err(format!(
                "GitHub repo contains invalid path segment: {repo:?}"
            ));
        }

        if !part
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '.' || ch == '-' || ch == '_')
        {
            return Err(format!("GitHub repo contains unsafe characters: {repo:?}"));
        }
    }

    Ok(())
}

pub fn github_repo_url(repo: &str) -> Result<String, String> {
    validate_github_repo_slug(repo)?;
    Ok(format!("https://github.com/{repo}.git"))
}

pub fn validated_plugin_path(base: &Path, plugin_id: &str) -> Result<PathBuf, String> {
    validate_plugin_id(plugin_id)?;
    Ok(base.join(plugin_id))
}

pub fn validate_relative_repo_path(path: &str) -> Result<(), String> {
    let path = Path::new(path);
    if path.is_absolute() {
        return Err("absolute paths are not allowed inside repository analysis".to_string());
    }

    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(
            "parent directory paths are not allowed inside repository analysis".to_string(),
        );
    }

    Ok(())
}

pub fn validate_existing_path_under(base: &Path, path: &Path) -> Result<(), String> {
    let base = base
        .canonicalize()
        .map_err(|error| format!("failed to canonicalize base path: {error}"))?;
    let path = path
        .canonicalize()
        .map_err(|error| format!("failed to canonicalize candidate path: {error}"))?;

    if path.starts_with(&base) {
        Ok(())
    } else {
        Err(format!(
            "refusing to access path outside repository root: {}",
            path.display()
        ))
    }
}

pub fn validate_github_download_url(download_url: &str) -> Result<(), String> {
    let url = Url::parse(download_url).map_err(|error| format!("invalid download URL: {error}"))?;
    if url.scheme() != "https" {
        return Err(format!(
            "download URL must use https, got {:?}",
            url.scheme()
        ));
    }

    match url.host_str() {
        Some("github.com") | Some("objects.githubusercontent.com") => Ok(()),
        Some(host) => Err(format!("unexpected GitHub asset download host: {host}")),
        None => Err("download URL has no host".to_string()),
    }
}

pub fn http_client() -> Result<Client, reqwest::Error> {
    Client::builder()
        .timeout(Duration::from_secs(HTTP_TIMEOUT_SECONDS))
        .user_agent("obsidian-stats-data")
        .build()
}

#[cfg(test)]
mod tests {
    use super::{validate_github_download_url, validate_github_repo_slug, validate_plugin_id};

    #[test]
    fn validates_safe_plugin_ids() {
        assert!(validate_plugin_id("calendar-plugin_2.0").is_ok());
        assert!(validate_plugin_id("../secrets").is_err());
        assert!(validate_plugin_id("nested/plugin").is_err());
        assert!(validate_plugin_id("").is_err());
    }

    #[test]
    fn validates_github_repo_slugs() {
        assert!(validate_github_repo_slug("owner/repo.name").is_ok());
        assert!(validate_github_repo_slug("owner/repo/extra").is_err());
        assert!(validate_github_repo_slug("../repo").is_err());
    }

    #[test]
    fn validates_github_asset_hosts() {
        assert!(
            validate_github_download_url(
                "https://github.com/owner/repo/releases/download/1.0.0/main.js"
            )
            .is_ok()
        );
        assert!(validate_github_download_url("https://example.com/main.js").is_err());
        assert!(validate_github_download_url("http://github.com/owner/repo/main.js").is_err());
    }
}
