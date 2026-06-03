use data_lib::{
    commit::Commit,
    common::{DownloadHistory, EntryChange, VersionHistory},
    date::Date,
    input_data::{ObsCommunityPlugin, ObsDownloadStats},
    version::Version,
};
use hashbrown::{HashMap, HashSet};

use serde::Serialize;
use serde_json::value;

pub mod analysis;
pub mod clone_repos;
pub mod data;
pub mod download_backfill;
pub mod license;
pub mod release_acquisition;

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
            .and_then(|v| v.get().parse::<u32>().ok())
            .unwrap_or(0);

        let versions = value
            .into_keys()
            .filter(|k| k != "downloads" && k != "latest" && k != "updated")
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

#[derive(Debug, Clone, Serialize)]
pub struct BorrowedPluginData<'a> {
    pub id: String,
    pub added_commit: &'a Commit,
    pub removed_commit: Option<&'a Commit>,
    pub initial_entry: &'a ObsCommunityPlugin,
    pub current_entry: &'a ObsCommunityPlugin,
    pub change_history: Vec<EntryChange>,
    pub download_history: DownloadHistory,
    pub download_count: u32,
    pub version_history: Vec<VersionHistory>,
    /// version -> lifecycle
    #[serde(skip)]
    version_history_map: HashMap<String, VersionLifecycle>,
}

#[derive(Debug, Clone)]
struct VersionLifecycle {
    initial_release_date: Date,
    deleted_date: Option<Date>,
}

impl<'a> BorrowedPluginData<'a> {
    pub fn new(
        id: String,
        added_commit: &'a Commit,
        initial_entry: &'a ObsCommunityPlugin,
    ) -> Self {
        let version_history_map = HashMap::new();
        let version_history = vec![];

        Self {
            id,
            added_commit,
            removed_commit: None,
            initial_entry,
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

        self.current_entry = new_entry;
    }

    pub fn update_download_history(&mut self, stats: &PluginDownloadStats) {
        if let Some(entry) = stats.entries.get(&self.id) {
            self.download_history
                .0
                .insert(stats.commit.date.to_fancy_string(), entry.downloads);

            if entry.downloads > self.download_count {
                self.download_count = entry.downloads;
            }
        }
    }

    pub fn update_version_history(&mut self, stats: &PluginDownloadStats) {
        let Some(entry) = stats.entries.get(&self.id) else {
            return;
        };

        self.update_version_history_from_versions(&stats.get_date(), &entry.versions);
    }

    pub fn update_version_history_from_versions(&mut self, date: &Date, versions: &[String]) {
        for version in versions {
            if !Version::validate(version) {
                continue;
            }

            self.mark_version_seen(date, version);
        }
    }

    pub fn update_version_history_from_snapshot(
        &mut self,
        date: &Date,
        previous_versions: Option<&HashSet<String>>,
        current_versions: &HashSet<String>,
    ) {
        for version in current_versions {
            self.mark_version_seen(date, version);
        }

        let Some(previous_versions) = previous_versions else {
            return;
        };

        for version in previous_versions.difference(current_versions) {
            if let Some(lifecycle) = self.version_history_map.get_mut(version)
                && lifecycle.deleted_date.is_none()
            {
                lifecycle.deleted_date = Some(date.clone());
            }
        }
    }

    fn mark_version_seen(&mut self, date: &Date, version: &str) {
        if !Version::validate(version) {
            return;
        }

        self.version_history_map
            .entry(version.to_string())
            .and_modify(|lifecycle| {
                lifecycle.deleted_date = None;
            })
            .or_insert_with(|| VersionLifecycle {
                initial_release_date: date.clone(),
                deleted_date: None,
            });
    }

    pub fn sort_version_history(&mut self) {
        self.version_history = self
            .version_history_map
            .iter()
            .map(|(version, lifecycle)| VersionHistory {
                version: version.clone(),
                version_object: Version::parse(version),
                initial_release_date: lifecycle.initial_release_date.clone(),
                deleted_date: lifecycle.deleted_date.clone(),
            })
            .collect();

        self.version_history
            .sort_by(|a, b| a.version_object.cmp(&b.version_object));
    }
}

#[cfg(test)]
mod tests {
    use super::BorrowedPluginData;
    use data_lib::{commit::Commit, date::Date, input_data::ObsCommunityPlugin};
    use hashbrown::HashSet;

    fn plugin() -> ObsCommunityPlugin {
        ObsCommunityPlugin {
            id: "plugin".to_string(),
            name: "Plugin".to_string(),
            author: "Author".to_string(),
            description: "Description".to_string(),
            repo: "owner/repo".to_string(),
        }
    }

    fn versions(values: &[&str]) -> HashSet<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn version_history_tracks_deleted_and_reappeared_versions() {
        let commit = Commit {
            date: Date::new(2024, 1, 1),
            hash: "abc".to_string(),
        };
        let plugin = plugin();
        let mut data = BorrowedPluginData::new("plugin".to_string(), &commit, &plugin);

        let first = versions(&["1.0.0", "1.1.0"]);
        data.update_version_history_from_snapshot(&Date::new(2024, 1, 1), None, &first);

        let second = versions(&["1.0.0"]);
        data.update_version_history_from_snapshot(&Date::new(2024, 1, 8), Some(&first), &second);

        data.sort_version_history();
        let removed = data
            .version_history
            .iter()
            .find(|entry| entry.version == "1.1.0")
            .unwrap();
        assert_eq!(removed.deleted_date, Some(Date::new(2024, 1, 8)));

        let third = versions(&["1.0.0", "1.1.0"]);
        data.update_version_history_from_snapshot(&Date::new(2024, 1, 15), Some(&second), &third);

        data.sort_version_history();
        let reappeared = data
            .version_history
            .iter()
            .find(|entry| entry.version == "1.1.0")
            .unwrap();
        assert_eq!(reappeared.initial_release_date, Date::new(2024, 1, 1));
        assert_eq!(reappeared.deleted_date, None);
    }
}
