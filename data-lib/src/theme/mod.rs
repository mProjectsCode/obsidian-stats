use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::{
    commit::{Commit, StringCommit},
    common::{EntryChange, EntryChangeDataPoint},
    date::Date,
    input_data::ObsCommunityTheme,
};

pub mod data_array;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct ThemeData {
    #[wasm_bindgen(skip)]
    pub id: String,
    #[wasm_bindgen(skip)]
    pub name: String,
    #[wasm_bindgen(skip)]
    pub added_commit: Commit,
    #[wasm_bindgen(skip)]
    pub removed_commit: Option<Commit>,
    #[wasm_bindgen(skip)]
    pub initial_entry: ObsCommunityTheme,
    #[wasm_bindgen(skip)]
    pub current_entry: ObsCommunityTheme,
    #[wasm_bindgen(skip)]
    pub change_history: Vec<EntryChange>,
}

impl ThemeData {
    pub fn released_in_month(&self, date: &Date) -> bool {
        self.added_commit.date.month == date.month && self.added_commit.date.year == date.year
    }

    pub fn removed_in_month(&self, date: &Date) -> bool {
        if let Some(removed_commit) = &self.removed_commit {
            removed_commit.date.month == date.month && removed_commit.date.year == date.year
        } else {
            false
        }
    }
}

#[wasm_bindgen]
impl ThemeData {
    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn name(&self) -> String {
        self.current_entry.name.clone()
    }

    pub fn author(&self) -> String {
        self.current_entry.author.clone()
    }

    pub fn modes(&self) -> Vec<String> {
        self.current_entry.modes.clone()
    }

    pub fn repo_url(&self) -> String {
        format!("https://github.com/{}", self.current_entry.repo)
    }

    pub fn obsidian_hub_url(&self) -> Option<String> {
        match self.removed_commit {
            Some(_) => None, // If the theme is removed, we don't provide an Obsidian Hub URL
            None => Some(format!(
                "https://publish.obsidian.md/hub/02+-+Community+Expansions/02.05+All+Community+Expansions/Themes/{}",
                self.id
            )),
        }
    }

    pub fn added_commit(&self) -> StringCommit {
        self.added_commit.to_string_commit()
    }

    pub fn removed_commit(&self) -> Option<StringCommit> {
        self.removed_commit.as_ref().map(|c| c.to_string_commit())
    }

    pub fn changes(&self) -> Vec<EntryChangeDataPoint> {
        self.change_history
            .iter()
            .map(|change| change.to_data_point())
            .collect()
    }
}
