use data_lib::{commit::Commit, common::EntryChange, input_data::ObsCommunityTheme};
use hashbrown::HashMap;
use serde::Serialize;
use slug::slugify;

pub mod data;

#[derive(Debug, Clone)]
pub struct ThemeList {
    pub entries: HashMap<String, ObsCommunityTheme>,
    pub commit: Commit,
}

#[derive(Debug, Clone, Serialize)]
pub struct BorrowedThemeData<'a> {
    pub id: String,
    pub name: String,
    pub added_commit: &'a Commit,
    pub removed_commit: Option<&'a Commit>,
    pub initial_entry: &'a ObsCommunityTheme,
    pub current_entry: &'a ObsCommunityTheme,
    pub change_history: Vec<EntryChange>,
}

#[derive(Debug, Clone, Default)]
pub struct ThemeIdCounter(pub HashMap<String, usize>);

impl ThemeIdCounter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_id(&mut self, name: &str) -> String {
        let mut id = slugify(name);
        let count = self.0.entry(id.clone()).or_insert(0);
        if *count > 0 {
            id.push_str(&format!("-{count}"));
        }
        *count += 1;
        id
    }
}

impl<'a> BorrowedThemeData<'a> {
    pub fn new(
        name: String,
        added_commit: &'a Commit,
        initial_entry: &'a ObsCommunityTheme,
        id_counter: &mut ThemeIdCounter,
    ) -> Self {
        Self {
            id: id_counter.get_id(&name),
            name,
            added_commit,
            removed_commit: None,
            initial_entry,
            current_entry: initial_entry,
            change_history: vec![EntryChange {
                property: "Theme Added".to_string(),
                commit: added_commit.clone(),
                old_value: String::new(),
                new_value: String::new(),
            }],
        }
    }

    pub fn find_changes(&mut self, theme_list: &'a ThemeList) {
        let new_entry = theme_list.entries.get(&self.name);
        let Some(new_entry) = new_entry else {
            if self.removed_commit.is_none() {
                self.removed_commit = Some(&theme_list.commit);
                self.change_history.push(EntryChange {
                    property: "Theme Removed".to_string(),
                    commit: theme_list.commit.clone(),
                    old_value: String::new(),
                    new_value: String::new(),
                });
            }
            return;
        };

        if self.removed_commit.is_some() {
            // Theme was removed and added again
            self.removed_commit = None;
            self.change_history.push(EntryChange {
                property: "Theme Readded".to_string(),
                commit: theme_list.commit.clone(),
                old_value: String::new(),
                new_value: String::new(),
            });
        }

        self.change_history
            .extend(self.current_entry.compare(new_entry, &theme_list.commit));

        self.current_entry = new_entry;
    }
}
