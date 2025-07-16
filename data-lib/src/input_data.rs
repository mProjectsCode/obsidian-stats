use hashbrown::HashMap;

use serde::{Deserialize, Serialize};
use serde_deserialize_duplicates::DeserializeLastDuplicate;
use serde_json::value;

use crate::{commit::Commit, common::EntryChange};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsCommunityPluginRemoved {
    pub id: String,
    pub name: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, DeserializeLastDuplicate)]
pub struct ObsCommunityPlugin {
    pub id: String,
    pub name: String,
    pub author: String,
    pub description: String,
    pub repo: String,
}

impl ObsCommunityPlugin {
    pub fn compare(&self, new: &ObsCommunityPlugin, commit: &Commit) -> Vec<EntryChange> {
        let mut changes = Vec::new();

        if self.name != new.name {
            changes.push(EntryChange {
                property: "name".to_string(),
                commit: commit.clone(),
                old_value: self.name.clone(),
                new_value: new.name.clone(),
            });
        }
        if self.author != new.author {
            changes.push(EntryChange {
                property: "author".to_string(),
                commit: commit.clone(),
                old_value: self.author.clone(),
                new_value: new.author.clone(),
            });
        }
        if self.description != new.description {
            changes.push(EntryChange {
                property: "description".to_string(),
                commit: commit.clone(),
                old_value: self.description.clone(),
                new_value: new.description.clone(),
            });
        }
        if self.repo != new.repo {
            changes.push(EntryChange {
                property: "repo".to_string(),
                commit: commit.clone(),
                old_value: self.repo.clone(),
                new_value: new.repo.clone(),
            });
        }

        changes
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsCommunityTheme {
    pub name: String,
    pub author: String,
    pub repo: String,
    pub screenshot: String,
    pub modes: Vec<String>,
    pub legacy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsCommunityPluginDeprecations(pub HashMap<String, Vec<String>>);

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ObsPluginList(pub Vec<ObsCommunityPlugin>);

impl ObsPluginList {
    pub fn get(&self) -> &Vec<ObsCommunityPlugin> {
        &self.0
    }

    pub fn to_hashmap(self) -> HashMap<String, ObsCommunityPlugin> {
        self.0.into_iter().map(|p| (p.id.clone(), p)).collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ObsDownloadStats<'a>(
    #[serde(borrow)] pub HashMap<String, HashMap<String, &'a value::RawValue>>,
);
