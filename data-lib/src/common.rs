use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use crate::{commit::Commit, date::Date, plugin::EntryChangeDataPoint, version::Version};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryChange {
    pub property: String,
    pub commit: Commit,
    pub old_value: String,
    pub new_value: String,
}

impl EntryChange {
    pub fn to_data_point(&self) -> EntryChangeDataPoint {
        EntryChangeDataPoint {
            property: self.property.clone(),
            commit: self.commit.to_string_commit(),
            old_value: self.old_value.clone(),
            new_value: self.new_value.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DownloadHistory(pub HashMap<String, u32>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionHistory {
    pub version: String,
    #[serde(skip)]
    pub version_object: Option<Version>,
    pub initial_release_date: Date,
}
