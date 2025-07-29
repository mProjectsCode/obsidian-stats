use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub mod data;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubReleaseAsset {
    pub name: String,
    pub size: u32,
    pub download_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubReleaseEntry {
    pub tag_name: String,
    pub published_at: DateTime<Utc>,
    pub assets: Vec<GithubReleaseAsset>,
}
