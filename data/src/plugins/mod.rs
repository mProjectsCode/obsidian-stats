use data_lib::{
    commit::Commit,
    common::{DownloadHistory, EntryChange, VersionHistory},
    date::Date,
    input_data::{ObsCommunityPlugin, ObsDownloadStats},
};
use hashbrown::HashMap;

use serde::Serialize;
use serde_json::value;

pub mod analysis;
pub mod clone_repos;
pub mod data;
pub mod download_backfill;
pub mod license;
pub mod release_acquisition;
pub mod stats_helper;

const PLUGIN_ADDED_PROPERTY: &str = "Plugin Added";
const PLUGIN_REMOVED_PROPERTY: &str = "Plugin Removed";
const PLUGIN_RE_ADDED_PROPERTY: &str = "Plugin Re-Added";

#[derive(Debug, Clone)]
pub struct PluginList {
    pub entries: HashMap<String, ObsCommunityPlugin>,
    pub commit: Commit,
}

#[derive(Debug, Clone)]
pub struct PluginDownloadStat {
    pub downloads: u32,
}

impl<'a> From<HashMap<String, &'a value::RawValue>> for PluginDownloadStat {
    fn from(value: HashMap<String, &'a value::RawValue>) -> Self {
        let downloads = value
            .get("downloads")
            .and_then(|v| v.get().parse::<u32>().ok())
            .unwrap_or(0);

        Self { downloads }
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
}

impl<'a> BorrowedPluginData<'a> {
    pub fn new(
        id: String,
        added_commit: &'a Commit,
        initial_entry: &'a ObsCommunityPlugin,
    ) -> Self {
        let version_history = vec![];

        Self {
            id,
            added_commit,
            removed_commit: None,
            initial_entry,
            current_entry: initial_entry,
            change_history: vec![EntryChange {
                property: PLUGIN_ADDED_PROPERTY.to_string(),
                commit: added_commit.clone(),
                old_value: String::new(),
                new_value: String::new(),
            }],
            download_history: DownloadHistory::default(),
            download_count: 0,
            version_history,
        }
    }

    pub fn find_changes(&mut self, plugin_list: &'a PluginList) {
        let new_entry = plugin_list.entries.get(&self.id);
        let Some(new_entry) = new_entry else {
            // plugin was removed
            if self.removed_commit.is_none() {
                self.removed_commit = Some(&plugin_list.commit);
                self.change_history.push(EntryChange {
                    property: PLUGIN_REMOVED_PROPERTY.to_string(),
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
                property: PLUGIN_RE_ADDED_PROPERTY.to_string(),
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

    pub fn was_listed_on(&self, date: &Date) -> bool {
        if date < &self.added_commit.date {
            return false;
        }

        let mut listed = true;
        let mut changes = self
            .change_history
            .iter()
            .filter(|change| {
                change.property == PLUGIN_REMOVED_PROPERTY
                    || change.property == PLUGIN_RE_ADDED_PROPERTY
            })
            .collect::<Vec<_>>();
        changes.sort_by(|left, right| left.commit.date.cmp(&right.commit.date));

        for change in changes {
            if &change.commit.date > date {
                break;
            }

            match change.property.as_str() {
                PLUGIN_REMOVED_PROPERTY => listed = false,
                PLUGIN_RE_ADDED_PROPERTY => listed = true,
                _ => {}
            }
        }

        listed
    }
}

#[cfg(test)]
mod tests {
    use super::{BorrowedPluginData, PluginList};
    use data_lib::{commit::Commit, date::Date, input_data::ObsCommunityPlugin};
    use hashbrown::HashMap;

    fn plugin() -> ObsCommunityPlugin {
        ObsCommunityPlugin {
            id: "plugin".to_string(),
            name: "Plugin".to_string(),
            author: "Author".to_string(),
            description: "Description".to_string(),
            repo: "owner/repo".to_string(),
        }
    }

    fn commit(date: Date, hash: &str) -> Commit {
        Commit {
            date,
            hash: hash.to_string(),
        }
    }

    #[test]
    fn listed_state_tracks_removed_and_readded_periods() {
        let added_commit = commit(Date::new(2024, 1, 1), "added");
        let plugin = plugin();
        let mut data = BorrowedPluginData::new("plugin".to_string(), &added_commit, &plugin);

        let removed_list = PluginList {
            entries: HashMap::new(),
            commit: commit(Date::new(2024, 2, 1), "removed"),
        };
        data.find_changes(&removed_list);

        let readded_list = PluginList {
            entries: HashMap::from([("plugin".to_string(), plugin.clone())]),
            commit: commit(Date::new(2024, 3, 1), "readded"),
        };
        data.find_changes(&readded_list);

        assert!(!data.was_listed_on(&Date::new(2023, 12, 31)));
        assert!(data.was_listed_on(&Date::new(2024, 1, 15)));
        assert!(!data.was_listed_on(&Date::new(2024, 2, 15)));
        assert!(data.was_listed_on(&Date::new(2024, 3, 1)));
    }
}
