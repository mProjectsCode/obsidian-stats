use hashbrown::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::value;

use crate::{
    commit::Commit,
    common::{DownloadHistory, EntryChange, VersionHistory},
    date::Date,
    input_data::{ObsCommunityPlugin, ObsDownloadStats},
    version::Version,
};

pub mod data;
pub mod license;
pub mod repo;

#[derive(Debug, Clone)]
pub struct PluginList {
    pub entries: HashMap<String, ObsCommunityPlugin>,
    pub commit: Commit,
}

#[derive(Debug, Clone)]
pub struct PluginDownloadStat {
    pub downloads: u32,
    pub versions: Vec<String>,
}

impl<'a> From<HashMap<String, &'a value::RawValue>> for PluginDownloadStat {
    fn from(value: HashMap<String, &'a value::RawValue>) -> Self {
        let downloads = value
            .get("downloads")
            .and_then(|v| u32::from_str_radix(v.get(), 10).ok())
            .unwrap_or(0);

        let versions = value
            .into_keys()
            .filter(|k| k != &"downloads" && k != &"latest" && k != &"updated")
            .collect();

        Self {
            downloads,
            versions,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PluginDownloadStats {
    pub entries: HashMap<String, PluginDownloadStat>,
    pub commit: Commit,
}

impl PluginDownloadStats {
    pub fn from_obs_data(stats: ObsDownloadStats<'_>, commit: Commit) -> Self {
        let entries = stats
            .0
            .into_iter()
            .map(|(id, entry)| (id, PluginDownloadStat::from(entry)))
            .collect();

        Self { entries, commit }
    }

    pub fn get_date(&self) -> Date {
        self.commit.date.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedPluginData {
    pub id: String,
    pub added_commit: Commit,
    pub removed_commit: Option<Commit>,
    pub initial_entry: ObsCommunityPlugin,
    pub current_entry: ObsCommunityPlugin,
    pub change_history: Vec<EntryChange>,
    pub download_history: DownloadHistory,
    pub download_count: u32,
    pub version_history: Vec<VersionHistory>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PluginData<'a> {
    pub id: String,
    pub added_commit: &'a Commit,
    pub removed_commit: Option<&'a Commit>,
    pub initial_entry: &'a ObsCommunityPlugin,
    pub current_entry: &'a ObsCommunityPlugin,
    pub change_history: Vec<EntryChange>,
    pub download_history: DownloadHistory,
    pub download_count: u32,
    pub version_history: Vec<VersionHistory>,
    /// version -> release date
    #[serde(skip)]
    version_history_map: HashMap<String, Date>,
}

impl<'a> PluginData<'a> {
    pub fn new(
        id: String,
        added_commit: &'a Commit,
        initial_entry: &'a ObsCommunityPlugin,
    ) -> Self {
        let version_history_map = HashMap::new();
        let version_history = vec![];

        Self {
            id,
            added_commit: added_commit,
            removed_commit: None,
            initial_entry: initial_entry,
            current_entry: initial_entry,
            change_history: vec![EntryChange {
                property: "Plugin Added".to_string(),
                commit: added_commit.clone(),
                old_value: String::new(),
                new_value: String::new(),
            }],
            download_history: DownloadHistory::default(),
            download_count: 0,
            version_history,
            version_history_map,
        }
    }

    // TODO: This function is currently unused
    pub fn add_change(&mut self, change: EntryChange) {
        self.change_history.push(change);
    }

    pub fn find_changes(&mut self, plugin_list: &'a PluginList) {
        let new_entry = plugin_list.entries.get(&self.id);
        let Some(new_entry) = new_entry else {
            // plugin was removed
            if self.removed_commit.is_none() {
                self.removed_commit = Some(&plugin_list.commit);
                self.change_history.push(EntryChange {
                    property: "Plugin Removed".to_string(),
                    commit: plugin_list.commit.clone(),
                    old_value: String::new(),
                    new_value: String::new(),
                });
            }

            return;
        };

        if self.removed_commit.is_some() {
            // plugin was re-added
            self.removed_commit = None;
            self.change_history.push(EntryChange {
                property: "Plugin Re-Added".to_string(),
                commit: plugin_list.commit.clone(),
                old_value: String::new(),
                new_value: String::new(),
            });
        }

        self.change_history
            .extend(self.current_entry.compare(new_entry, &plugin_list.commit));

        self.current_entry = &new_entry;
    }

    pub fn update_download_history(&mut self, stats: &PluginDownloadStats) {
        // if let Some(entry) = stats.entries.get(&self.id) {
        //     self.download_history.0.insert(date, entry.downloads);

        //     if entry.downloads > self.download_count {
        //         self.download_count = entry.downloads;
        //     }
        //     true
        // } else {
        //     false
        // }
        match stats.entries.get(&self.id) {
            Some(entry) => {
                self.download_history.0.push(entry.downloads);

                if entry.downloads > self.download_count {
                    self.download_count = entry.downloads;
                }
            }
            None => {
                self.download_history.0.push(0);
            }
        }
    }

    pub fn update_version_history(&mut self, stats: &PluginDownloadStats) {
        let Some(entry) = stats.entries.get(&self.id) else {
            return;
        };

        for version in entry.versions.iter() {
            if !Version::validate(version) {
                continue;
            }

            if !self.version_history_map.contains_key(version) {
                self.version_history_map
                    .insert(version.clone(), stats.get_date());
            }
        }
    }

    pub fn sort_version_history(&mut self) {
        self.version_history = self
            .version_history_map
            .iter()
            .map(|(version, date)| VersionHistory {
                version: version.clone(),
                version_object: Version::parse(version),
                initial_release_date: date.clone(),
            })
            .collect();

        self.version_history
            .sort_by(|a, b| a.version_object.cmp(&b.version_object));
    }
}
