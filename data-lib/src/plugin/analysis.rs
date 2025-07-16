use std::ops::Index;

use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

use crate::{
    date::Date,
    plugin::{FundingUrl, PluginData, PluginRepoData, PluginRepoExtractedData},
};

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct FullPluginData {
    #[wasm_bindgen(skip)]
    pub data: PluginData,
    #[wasm_bindgen(skip)]
    pub extended: Option<PluginRepoData>,
}

impl FullPluginData {
    pub fn new(data: PluginData, extended: Option<PluginRepoData>) -> Self {
        Self { data, extended }
    }

    pub fn extended(&self) -> Option<&PluginRepoData> {
        self.extended.as_ref()
    }

    pub fn repo_data(&self) -> Option<&PluginRepoExtractedData> {
        self.extended().and_then(|r| r.repo.as_ref().ok())
    }
}

#[wasm_bindgen]
impl FullPluginData {
    pub fn id(&self) -> String {
        self.data.id.clone()
    }

    pub fn name(&self) -> String {
        self.data.current_entry.name.clone()
    }

    pub fn author(&self) -> String {
        self.data.current_entry.author.clone()
    }

    pub fn description(&self) -> String {
        self.data.current_entry.description.clone()
    }

    pub fn repo_url(&self) -> String {
        self.data.current_entry.repo.clone()
    }

    pub fn funding_url(&self) -> Option<String> {
        // TODO: support FundingUrl::Object
        self.repo_data()
            .and_then(|r| r.manifest.funding_url.as_ref())
            .and_then(|f| match f {
                FundingUrl::String(url) => Some(url.clone()),
                FundingUrl::Object(_) => None,
            })
    }

    pub fn release_date(&self) -> String {
        self.data.added_commit.date.to_fancy_string()
    }

    pub fn removed_date(&self) -> Option<String> {
        self.data
            .removed_commit
            .as_ref()
            .map(|c| c.date.to_fancy_string())
    }

    pub fn last_updated_date(&self) -> Option<String> {
        self.data
            .version_history
            .last()
            .map(|v| v.initial_release_date.to_fancy_string())
    }

    pub fn license_package_json(&self) -> Option<String> {
        self.repo_data().map(|r| r.package_json_license.clone())
    }

    pub fn license_file(&self) -> Option<String> {
        self.repo_data().map(|r| r.license_file.clone())
    }

    pub fn package_managers(&self) -> Option<Vec<String>> {
        self.repo_data().map(|r| {
            r.package_managers
                .iter()
                .map(|pm| pm.get_identifier().to_string())
                .collect()
        })
    }

    pub fn bundlers(&self) -> Option<Vec<String>> {
        self.repo_data().map(|r| {
            r.bundlers
                .iter()
                .map(|b| b.get_identifier().to_string())
                .collect()
        })
    }

    pub fn testing_frameworks(&self) -> Option<Vec<String>> {
        self.repo_data().map(|r| {
            r.testing_frameworks
                .iter()
                .map(|tf| tf.get_identifier().to_string())
                .collect()
        })
    }

    pub fn uses_typescript(&self) -> Option<bool> {
        self.repo_data().map(|r| r.uses_typescript)
    }

    pub fn download_count(&self) -> u32 {
        self.data.download_count
    }

    /// Get the download data points in the form
    /// ```ts
    /// interface DownloadDataPoint {
    ///     date: string; // e.g. "2023-10-01"
    ///     downloads: number; // e.g. 100
    ///     delta: number; // e.g. 10 (change from previous data point)
    /// }
    /// ```
    pub fn download_data_points(&self) -> JsValue {
        let end_date = self
            .data
            .removed_commit
            .as_ref()
            .map_or_else(|| Date::now(), |c| c.date.clone());

        let weekly_iterator = self.data.added_commit.date.iterate_weekly_to(&end_date);

        let data_iterator = self.data.download_history.0.iter().zip(weekly_iterator);

        // TODO

        JsValue::null()
    }
}

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct FullPluginDataArray {
    #[wasm_bindgen(skip)]
    pub data: Vec<FullPluginData>,
}

impl FullPluginDataArray {
    pub fn new(data: Vec<PluginData>, extended: Vec<PluginRepoData>) -> Self {
        let data = data
            .into_iter()
            .map(|d| FullPluginData {
                extended: extended.iter().find(|r| r.id == d.id).cloned(),
                data: d,
            })
            .collect();
        Self { data }
    }
}

impl Index<usize> for FullPluginDataArray {
    type Output = FullPluginData;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

#[wasm_bindgen]
impl FullPluginDataArray {
    pub fn view(&self) -> FullPluginDataArrayView {
        FullPluginDataArrayView::new(self.data.len())
    }
}

/// A view into a `FullPluginDataArray` that allows access to the underlying data without cloning.
#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct FullPluginDataArrayView {
    #[wasm_bindgen(skip)]
    pub data: Vec<usize>,
}

impl FullPluginDataArrayView {
    pub fn new(len: usize) -> Self {
        Self {
            data: (0..len).collect(),
        }
    }
}

#[wasm_bindgen]
impl FullPluginDataArrayView {
    pub fn get(&self, data: &FullPluginDataArray, index: usize) -> FullPluginData {
        data[self.data[index]].clone()
    }

    pub fn to_vec(&self, data: &FullPluginDataArray) -> Vec<FullPluginData> {
        self.data.iter().map(|&index| data[index].clone()).collect()
    }

    pub fn sort_asc(&mut self, data: &FullPluginDataArray, spec: PluginDataSortSpec) {
        self.data.sort_by(|&a, &b| spec.cmp(&data[a], &data[b]));
    }

    pub fn sort_desc(&mut self, data: &FullPluginDataArray, spec: PluginDataSortSpec) {
        self.data.sort_by(|&a, &b| spec.cmp(&data[b], &data[a]));
    }

    /// Truncate the view to the top `count` elements.
    pub fn truncate_top(&mut self, count: usize) {
        if count < self.data.len() {
            self.data.truncate(count);
        }
    }

    /// Truncate the view to the bottom `count` elements.
    pub fn truncate_bottom(&mut self, count: usize) {
        if count < self.data.len() {
            self.data.drain(0..self.data.len() - count);
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
            PluginDataSortSpec::DateUpdated => a
                .data
                .version_history
                .last()
                .map(|x| &x.initial_release_date)
                .cmp(
                    &b.data
                        .version_history
                        .last()
                        .map(|x| &x.initial_release_date),
                ),
            PluginDataSortSpec::Downloads => a.data.download_count.cmp(&b.data.download_count),
            PluginDataSortSpec::Versions => a
                .data
                .version_history
                .len()
                .cmp(&b.data.version_history.len()),
        }
    }
}
