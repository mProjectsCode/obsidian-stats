use serde::{Deserialize, Serialize};

use crate::{commit::Commit, date::Date, version::Version};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryChange {
    pub property: String,
    pub commit: Commit,
    pub old_value: String,
    pub new_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DownloadHistory(pub Vec<u32>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionHistory {
    pub version: String,
    #[serde(skip)]
    pub version_object: Option<Version>,
    pub initial_release_date: Date,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadDataPoint {
    pub date: String,
    pub downloads: Option<u32>,
    pub growth: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PerMonthDataPoint {
    pub year: String,
    pub month: String,
    pub value: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadReleaseCorrelationDataPoint {
    pub id: String,
    pub name: String,
    pub downloads: u32,
    pub releases: u32,
    pub initial_release_date: String,
}
