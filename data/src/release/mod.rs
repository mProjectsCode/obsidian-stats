use crate::date::Date;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubUser {
    pub login: String,
    pub id: u32,
    pub node_id: String,
    pub avatar_url: String,
    pub gravatar_id: String,
    pub url: String,
    pub html_url: String,
    pub followers_url: String,
    pub gist_url: String,
    pub starred_url: String,
    pub subscriptions_url: String,
    pub organizations_url: String,
    pub repos_url: String,
    pub events_url: String,
    pub received_events_url: String,
    // TODO: Check if this matters for serialization
    #[serde(rename = "type")]
    pub r#type: String,
    pub site_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubReleaseAsset {
    pub url: String,
    pub browser_download_url: String,
    pub id: u32,
    pub node_id: String,
    pub name: String,
    pub label: String,
    pub state: String,
    pub content_type: String,
    pub size: u32,
    pub download_count: u32,
    pub created_at: Date,
    pub updated_at: Date,
    pub uploader: GithubUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubReleaseReactions {
    pub url: String,
    pub total_count: u32,
    #[serde(rename = "+1")]
    pub plus_one: u32,
    #[serde(rename = "-1")]
    pub minus_one: u32,
    pub laugh: u32,
    pub hooray: u32,
    pub confused: u32,
    pub heart: u32,
    pub rocket: u32,
    pub eyes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubReleaseEntry {
    pub url: String,
    pub assets_url: String,
    pub upload_url: String,
    pub html_url: String,
    pub id: u32,
    pub author: GithubUser,
    pub node_id: String,
    pub tag_name: String,
    pub target_commitish: String,
    pub name: String,
    pub draft: bool,
    pub prerelease: bool,
    pub created_at: Date,
    pub published_at: Date,
    pub assets: Vec<GithubReleaseAsset>,
    pub tarball_url: String,
    pub zipball_url: String,
    pub body: String,
    pub reactions: GithubReleaseReactions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObsidianPlatform {
    Desktop,
    Mobile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsidianReleaseInfo {
    pub version: String,
    pub platform: ObsidianPlatform,
    pub insider: bool,
    pub date: Date,
    pub info: String,
    pub major_release: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub type GithubReleases = Vec<GithubReleaseEntry>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAsset {
    pub name: String,
    pub download_count: u32,
    pub size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseEntry {
    pub version: String,
    pub date: Date,
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyReleaseGrowthEntry {
    pub date: Date,
    pub version: String,
    pub asset: String,
    pub downloads: u32,
}

pub const ALL_OS: [&str; 3] = ["macos", "windows", "linux"];