use std::ops::Index;

use hashbrown::HashMap;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::plugin::{PluginData, PluginExtraData, full::FullPluginData};

mod downloads;
mod licenses;
mod repo_metrics;

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct PluginDataArray {
    #[wasm_bindgen(skip)]
    pub data: Vec<FullPluginData>,
}

impl PluginDataArray {
    pub fn new(data: Vec<PluginData>, extended: Vec<PluginExtraData>) -> Self {
        let mut extended_by_id = extended
            .into_iter()
            .map(|entry| (entry.id.clone(), entry))
            .collect::<HashMap<_, _>>();

        let data = data
            .into_iter()
            .map(|d| FullPluginData {
                extended: extended_by_id.remove(&d.id),
                data: d,
            })
            .collect();
        Self { data }
    }
}

impl Index<usize> for PluginDataArray {
    type Output = FullPluginData;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

#[wasm_bindgen]
impl PluginDataArray {
    pub fn view(&self) -> PluginDataArrayView {
        PluginDataArrayView::new(self.data.len())
    }
}

/// A view into a `PluginDataArray` that allows access to the underlying data without cloning.
#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct PluginDataArrayView {
    #[wasm_bindgen(skip)]
    pub indices: Vec<usize>,
}

impl PluginDataArrayView {
    pub fn new(len: usize) -> Self {
        Self {
            indices: (0..len).collect(),
        }
    }

    pub fn iter_data<'a>(
        &'a self,
        data: &'a PluginDataArray,
    ) -> impl Iterator<Item = &'a FullPluginData> {
        self.indices.iter().map(move |&index| &data[index])
    }
}

#[wasm_bindgen]
impl PluginDataArrayView {
    pub fn len(&self) -> usize {
        self.indices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    pub fn get(&self, data: &PluginDataArray, index: usize) -> FullPluginData {
        data[self.indices[index]].clone()
    }

    pub fn get_ids(&self, data: &PluginDataArray) -> Vec<String> {
        self.iter_data(data).map(|item| item.id()).collect()
    }

    pub fn get_by_id(&self, data: &PluginDataArray, id: &str) -> Option<FullPluginData> {
        self.iter_data(data).find(|item| item.id() == id).cloned()
    }

    pub fn to_vec(&self, data: &PluginDataArray) -> Vec<FullPluginData> {
        self.iter_data(data).cloned().collect()
    }

    pub fn sort_asc(&mut self, data: &PluginDataArray, spec: PluginDataSortSpec) {
        self.indices.sort_by(|&a, &b| spec.cmp(&data[a], &data[b]));
    }

    pub fn sort_desc(&mut self, data: &PluginDataArray, spec: PluginDataSortSpec) {
        self.indices.sort_by(|&a, &b| spec.cmp(&data[b], &data[a]));
    }

    /// Truncate the view to the top `count` elements.
    pub fn truncate_top(&mut self, count: usize) {
        if count < self.indices.len() {
            self.indices.truncate(count);
        }
    }

    /// Truncate the view to the bottom `count` elements.
    pub fn truncate_bottom(&mut self, count: usize) {
        if count < self.indices.len() {
            self.indices.drain(0..self.indices.len() - count);
        }
    }
}

#[wasm_bindgen]
pub enum PluginDataSortSpec {
    Id,
    Name,
    DateAdded,
    DateRemoved,
    DateUpdated,
    Downloads,
    Versions,
}

impl PluginDataSortSpec {
    pub fn cmp(&self, a: &FullPluginData, b: &FullPluginData) -> std::cmp::Ordering {
        match self {
            PluginDataSortSpec::Id => a.data.id.cmp(&b.data.id),
            PluginDataSortSpec::Name => a.data.current_entry.name.cmp(&b.data.current_entry.name),
            PluginDataSortSpec::DateAdded => {
                a.data.added_commit.date.cmp(&b.data.added_commit.date)
            }
            PluginDataSortSpec::DateRemoved => a
                .data
                .removed_commit
                .as_ref()
                .map(|x| &x.date)
                .cmp(&b.data.removed_commit.as_ref().map(|x| &x.date)),
            PluginDataSortSpec::DateUpdated => a.last_updated().cmp(&b.last_updated()),
            PluginDataSortSpec::Downloads => a.data.download_count.cmp(&b.data.download_count),
            PluginDataSortSpec::Versions => a.listed_version_count().cmp(&b.listed_version_count()),
        }
    }
}
