use hashbrown::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::value;

use crate::{commit::Commit, common::EntryChange};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsCommunityPluginRemoved {
    pub id: String,
    pub name: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ObsCommunityPlugin {
    pub id: String,
    pub name: String,
    pub author: String,
    pub description: String,
    pub repo: String,
}

impl<'de> Deserialize<'de> for ObsCommunityPlugin {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct PluginVisitor;

        impl<'de> serde::de::Visitor<'de> for PluginVisitor {
            type Value = ObsCommunityPlugin;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a plugin object")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut id = None;
                let mut name = None;
                let mut author = None;
                let mut description = None;
                let mut repo = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "id" => id = Some(map.next_value()?),
                        "name" => name = Some(map.next_value()?),
                        "author" => author = Some(map.next_value()?),
                        "description" => description = Some(map.next_value()?),
                        "repo" => repo = Some(map.next_value()?),
                        _ => {
                            let _ = map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }

                Ok(ObsCommunityPlugin {
                    id: id.ok_or_else(|| serde::de::Error::missing_field("id"))?,
                    name: name.ok_or_else(|| serde::de::Error::missing_field("name"))?,
                    author: author.ok_or_else(|| serde::de::Error::missing_field("author"))?,
                    description: description
                        .ok_or_else(|| serde::de::Error::missing_field("description"))?,
                    repo: repo.ok_or_else(|| serde::de::Error::missing_field("repo"))?,
                })
            }
        }

        deserializer.deserialize_map(PluginVisitor)
    }
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
    #[serde(default)]
    pub modes: Vec<String>,
    #[serde(default)]
    pub legacy: bool,
}

impl ObsCommunityTheme {
    pub fn compare(&self, new: &ObsCommunityTheme, commit: &Commit) -> Vec<EntryChange> {
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
        if self.modes != new.modes {
            changes.push(EntryChange {
                property: "modes".to_string(),
                commit: commit.clone(),
                old_value: self.modes.join(", "),
                new_value: new.modes.join(", "),
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ObsThemeList(pub Vec<ObsCommunityTheme>);

impl ObsThemeList {
    pub fn get(&self) -> &Vec<ObsCommunityTheme> {
        &self.0
    }

    pub fn to_hashmap(self) -> HashMap<String, ObsCommunityTheme> {
        self.0.into_iter().map(|p| (p.name.clone(), p)).collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsReleasesFeed {
    pub feed: ObsReleasesFeedInner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsReleasesFeedInner {
    #[serde(rename = "entry", default)]
    pub entries: Vec<ObsReleasesFeedEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsReleasesFeedEntry {
    pub id: String,
    pub title: String,
    pub updated: String,
    pub content: String,
}

#[cfg(test)]
mod tests {
    use super::ObsCommunityPlugin;

    #[test]
    fn plugin_deserialization_supports_owned_reader_keys_and_last_duplicates() {
        let json = br#"{"id":"first","id":"last","name":"Name","author":"Author","description":"Description","repo":"owner/repo"}"#;
        let plugin: ObsCommunityPlugin =
            serde_json::from_reader(std::io::Cursor::new(json)).unwrap();
        assert_eq!(plugin.id, "last");
    }
}
