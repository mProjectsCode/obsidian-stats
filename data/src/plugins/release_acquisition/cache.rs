use std::{
    fs::{self, File},
    io::{BufWriter, Read, Write},
    path::{Path, PathBuf},
};

use reqwest::blocking::Client;

use crate::{constants::PLUGIN_RELEASE_MAIN_JS_PATH, state::now_unix_seconds};

use super::ReleaseFetchStatus;

const MAX_MAIN_JS_DOWNLOAD_BYTES: u64 = 512 * 1024 * 1024; // 512 MB

#[derive(Clone, Copy)]
pub(super) enum MainJsCacheOutcome {
    Downloaded,
    Reused,
}

pub(super) enum MainJsDownloadError {
    RateLimited(u16),
    InvalidSize(u64),
    Request(String),
    Http(u16),
    Read(String),
    SizeMismatch { expected: u64, actual: u64 },
    Write(String),
}

impl MainJsDownloadError {
    pub(super) fn status(&self) -> ReleaseFetchStatus {
        match self {
            Self::RateLimited(_) => ReleaseFetchStatus::MainJsRateLimited,
            Self::InvalidSize(size) => {
                ReleaseFetchStatus::MainJsDownloadFailed(format!("invalid_size:{size}"))
            }
            Self::Request(err) => {
                ReleaseFetchStatus::MainJsDownloadFailed(format!("request:{err}"))
            }
            Self::Http(status) => {
                ReleaseFetchStatus::MainJsDownloadFailed(format!("http:{status}"))
            }
            Self::Read(err) => ReleaseFetchStatus::MainJsDownloadFailed(format!("read:{err}")),
            Self::SizeMismatch { expected, actual } => ReleaseFetchStatus::MainJsDownloadFailed(
                format!("size_mismatch:{expected}:{actual}"),
            ),
            Self::Write(err) => ReleaseFetchStatus::MainJsDownloadFailed(format!("write:{err}")),
        }
    }

    pub(super) fn detail_message(&self) -> String {
        match self {
            Self::RateLimited(status) => format!("GitHub returned HTTP {status}"),
            Self::InvalidSize(size) => format!("asset size {size} bytes is outside allowed bounds"),
            Self::Request(err) => err.clone(),
            Self::Http(status) => format!("GitHub returned HTTP {status}"),
            Self::Read(err) => err.clone(),
            Self::SizeMismatch { expected, actual } => {
                format!("expected {expected} bytes, downloaded {actual} bytes")
            }
            Self::Write(err) => err.clone(),
        }
    }
}

pub(super) fn save_main_js_to_cache(
    client: &Client,
    plugin_id: &str,
    release_tag: &str,
    download_url: &str,
    size: u64,
) -> Result<MainJsCacheOutcome, MainJsDownloadError> {
    if size == 0 || size > MAX_MAIN_JS_DOWNLOAD_BYTES {
        return Err(MainJsDownloadError::InvalidSize(size));
    }

    let cache_path = release_main_js_cache_path(plugin_id, release_tag);
    if let Ok(meta) = fs::metadata(&cache_path)
        && meta.len() == size
    {
        return Ok(MainJsCacheOutcome::Reused);
    }

    if let Some(parent) = cache_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let mut request = client
        .get(download_url)
        .header("Accept", "application/octet-stream")
        .header("User-Agent", "obsidian-stats-data");

    if let Ok(token) = std::env::var("GITHUB_TOKEN")
        && !token.is_empty()
    {
        request = request.bearer_auth(token);
    }

    let response = match request.send() {
        Ok(resp) => resp,
        Err(err) => return Err(MainJsDownloadError::Request(err.to_string())),
    };

    if response.status().as_u16() == 403 || response.status().as_u16() == 429 {
        return Err(MainJsDownloadError::RateLimited(response.status().as_u16()));
    }

    if !response.status().is_success() {
        return Err(MainJsDownloadError::Http(response.status().as_u16()));
    }

    let tmp_path = cache_path.with_file_name(format!(
        ".tmp-{}-{}",
        cache_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("main.js"),
        now_unix_seconds()
    ));

    let write_result = stream_response_to_file(response, &tmp_path, size);

    match write_result {
        Ok(actual) if actual == size => fs::rename(&tmp_path, cache_path)
            .map(|()| MainJsCacheOutcome::Downloaded)
            .map_err(|error| MainJsDownloadError::Write(error.to_string())),
        Ok(actual) => {
            let _ = fs::remove_file(&tmp_path);
            Err(MainJsDownloadError::SizeMismatch {
                expected: size,
                actual,
            })
        }
        Err(error) => {
            let _ = fs::remove_file(&tmp_path);
            Err(error)
        }
    }
}

fn stream_response_to_file(
    mut response: reqwest::blocking::Response,
    path: &Path,
    expected_size: u64,
) -> Result<u64, MainJsDownloadError> {
    let file = File::create(path).map_err(|error| MainJsDownloadError::Write(error.to_string()))?;
    let mut writer = BufWriter::new(file);
    let mut buffer = [0u8; 64 * 1024];
    let mut total = 0u64;

    loop {
        let read = response
            .read(&mut buffer)
            .map_err(|error| MainJsDownloadError::Read(error.to_string()))?;
        if read == 0 {
            break;
        }

        total += read as u64;
        if total > expected_size || total > MAX_MAIN_JS_DOWNLOAD_BYTES {
            return Err(MainJsDownloadError::SizeMismatch {
                expected: expected_size,
                actual: total,
            });
        }

        writer
            .write_all(&buffer[..read])
            .map_err(|error| MainJsDownloadError::Write(error.to_string()))?;
    }

    writer
        .flush()
        .map_err(|error| MainJsDownloadError::Write(error.to_string()))?;

    Ok(total)
}

pub fn release_main_js_cache_path(plugin_id: &str, release_tag: &str) -> PathBuf {
    let sanitized_tag = release_tag
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '.' || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();

    Path::new(PLUGIN_RELEASE_MAIN_JS_PATH)
        .join(plugin_id)
        .join(format!("{sanitized_tag}-main.js"))
}
