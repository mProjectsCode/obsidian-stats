use std::ops::Index;

use itertools::Itertools;
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

    pub fn get_downloads_at(&self, date: &Date) -> Option<u32> {
        self.data.download_history.0.get(&date.to_fancy_string())
            .copied()
    }

    pub fn find_downloads_in_week(&self, date: &Date) -> Option<u32> {
        for i in 0..7 {
            let mut d = date.clone();
            d.reverse_days(i);
            if let Some(downloads) = self.get_downloads_at(&d) {
                return Some(downloads);
            }
        }

        None
    }
}

#[wasm_bindgen]
impl FullPluginData {
    pub fn has_repo_data(&self) -> bool {
        self.repo_data().is_some()
    }

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

    pub fn author_url(&self) -> Option<String> {
        self.repo_data().and_then(|r| r.manifest.author_url.clone())   
    }

    pub fn help_url(&self) -> Option<String> {
        self.repo_data().and_then(|r| r.manifest.help_url.clone())   
    }

    pub fn min_app_version(&self) -> Option<String> {
        self.repo_data().map(|r| r.manifest.min_app_version.clone())
    }

    pub fn is_desktop_only(&self) -> Option<bool> {
        self.repo_data()
            .and_then(|r| r.manifest.is_desktop_only.clone())
    }

    pub fn obsidian_url(&self) -> Option<String> {
        match self.data.removed_commit {
            Some(_) => None, // If the plugin is removed, we don't provide an Obsidian URL
            None => Some(format!("obsidian://show-plugin?id={}", self.data.id)),
        }
    }

    pub fn obsidian_hub_url(&self) -> Option<String> {
        match self.data.removed_commit {
            Some(_) => None, // If the plugin is removed, we don't provide an Obsidian Hub URL
            None => Some(format!("https://publish.obsidian.md/hub/02+-+Community+Expansions/02.05+All+Community+Expansions/Plugins/{}", self.data.id)),
        }
    }

    pub fn release_date(&self) -> String {
        self.data.added_commit.date.to_fancy_string()
    }

    pub fn added_commit_hash(&self) -> String {
        self.data.added_commit.hash.clone()
    }

    pub fn removed_date(&self) -> Option<String> {
        self.data
            .removed_commit
            .as_ref()
            .map(|c| c.date.to_fancy_string())
    }

    pub fn removed_commit_hash(&self) -> Option<String> {
        self.data
            .removed_commit
            .as_ref()
            .map(|c| c.hash.clone())
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

    pub fn has_beta_manifest(&self) -> Option<bool> {
        self.repo_data().map(|r| r.has_beta_manifest)
    }

    pub fn has_package_json(&self) -> Option<bool> {
        self.repo_data().map(|r| r.has_package_json)
    }

    pub fn has_test_files(&self) -> Option<bool> {
        self.repo_data().map(|r| r.has_test_files)
    }

    pub fn dev_dependencies(&self) -> Option<Vec<String>> {
        self.repo_data().map(|r| r.dev_dependencies.clone())
    }

    pub fn dependencies(&self) -> Option<Vec<String>> {
        self.repo_data().map(|r| r.dependencies.clone())
    }

    pub fn download_count(&self) -> u32 {
        self.data.download_count
    }

    /// Get the download data points in the form
    /// ```ts
    /// interface DownloadDataPoint {
    ///     date: string; // e.g. "2023-10-01"
    ///     downloads: number | undefined; // e.g. 100
    ///     delta: number | undefined; // e.g. 10 (change from previous data point)
    /// }
    /// ```
    pub fn download_data_points(&self) -> JsValue {
        let end_date = self
            .data
            .removed_commit
            .as_ref()
            .map_or_else(|| Date::now(), |c| c.date.clone());

        let data = self.data.added_commit.date.iterate_daily_to(&end_date).map(|date| {
            let mut prev_date = date.clone();
            prev_date.reverse_days(1);

            let downloads = self.get_downloads_at(&date).and_then(|d| {
                if d > 0 {
                    Some(d)
                } else {
                    None
                }
            });
            let previous_downloads = self.get_downloads_at(&prev_date).and_then(|d| {
                if d > 0 {
                    Some(d)
                } else {
                    None
                }
            });

            let delta = match (downloads, previous_downloads) {
                (Some(d), Some(pd)) if d >= pd => Some(d - pd),
                _ => None,
            };

            let obj = js_sys::Object::new();
            js_sys::Reflect::set(&obj, &"date".into(), &date.to_string().into()).unwrap();
            js_sys::Reflect::set(&obj, &"downloads".into(), &downloads.into()).unwrap();
            js_sys::Reflect::set(&obj, &"delta".into(), &delta.into()).unwrap();

            JsValue::from(obj)
        }).collect_vec();

        JsValue::from(data)
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

    pub fn get_ids(&self, data: &FullPluginDataArray) -> Vec<String> {
        self.data.iter().map(|&index| data[index].id()).collect()
    }

    pub fn get_by_id(&self, data: &FullPluginDataArray, id: &str) -> Option<FullPluginData> {
        self.data.iter().find_map(|&index| {
            let item = &data[index];
            if item.id() == id {
                Some(item.clone())
            } else {
                None
            }
        })
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

    /// Get the total downloads (sum of all plugins in the view).
    /// 
    /// The data is in the form:
    /// ```ts
    /// interface DownloadDataPoint {
    ///     date: string; // e.g. "2023-10-01"
    ///     downloads: number; // e.g. 100
    ///     delta: number; // e.g. 10 (change from previous data point)
    /// }
    /// ```
    pub fn total_download_data(&self, data: &FullPluginDataArray) -> JsValue {
        let start_date = Date::new(2020, 1, 1);
        let end_date = Date::now();

        let data = start_date.iterate_weekly_to(&end_date).map(|date| {
            let mut prev_date = date.clone();
            prev_date.reverse_days(7);

            let downloads = self.data.iter().fold(0, |acc, index| {
                acc + data[*index].find_downloads_in_week(&date).unwrap_or(0)
            });
            let previous_downloads = self.data.iter().fold(0, |acc, index| {
                acc + data[*index].find_downloads_in_week(&prev_date).unwrap_or(0)
            });

            let downloads = if downloads > 0 {
                Some(downloads)
            } else {
                None
            };

            let previous_downloads = if previous_downloads > 0 {
                Some(previous_downloads)
            } else {
                None
            };

            let delta = match (downloads, previous_downloads) {
                (Some(d), Some(pd)) if d >= pd => Some(d - pd),
                _ => None,
            };

            let obj = js_sys::Object::new();
            js_sys::Reflect::set(&obj, &"date".into(), &date.to_string().into()).unwrap();
            js_sys::Reflect::set(&obj, &"downloads".into(), &downloads.into()).unwrap();
            js_sys::Reflect::set(&obj, &"delta".into(), &delta.into()).unwrap();

            JsValue::from(obj)
        }).collect_vec();

        JsValue::from(data)
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
