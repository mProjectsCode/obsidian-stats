use std::ops::Index;

use itertools::Itertools;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    commit::StringCommit,
    date::Date,
    license::{LicenseDescriptionNested, Licenses},
    plugin::{
        DownloadDataPoint, EntryChangeDataPoint, FundingUrl, IndividualDownloadDataPoint,
        NamedDataPoint, PluginCountMonthlyDataPoint, PluginData, PluginExtraData,
        PluginInactivityByReleaseDataPoint, PluginLicenseDataPoints, PluginOverviewDataPoint,
        PluginRemovedByReleaseDataPoint, PluginRepoData, PluginRepoDataPoints,
        PluginYearlyDataPoint, VersionDataPoint,
        warnings::{PluginWarning, get_plugin_warnings},
    },
};

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct FullPluginData {
    #[wasm_bindgen(skip)]
    pub data: PluginData,
    #[wasm_bindgen(skip)]
    pub extended: Option<PluginExtraData>,
}

impl FullPluginData {
    pub fn new(data: PluginData, extended: Option<PluginExtraData>) -> Self {
        Self { data, extended }
    }

    pub fn extended(&self) -> Option<&PluginExtraData> {
        self.extended.as_ref()
    }

    pub fn repo_data(&self) -> Option<&PluginRepoData> {
        self.extended().and_then(|r| r.repo.as_ref().ok())
    }

    pub fn get_downloads_at(&self, date: &Date) -> Option<u32> {
        self.data
            .download_history
            .0
            .get(&date.to_fancy_string())
            .copied()
    }

    pub fn find_downloads_in_week(&self, date: &Date) -> Option<u32> {
        for i in 0..7 {
            let mut d = date.clone();
            d.advance_days(i);
            if let Some(downloads) = self.get_downloads_at(&d) {
                return Some(downloads);
            }
        }

        None
    }

    pub fn find_downloads_after_date(&self, date: &Date) -> Option<u32> {
        let end_date = self
            .data
            .removed_commit
            .as_ref()
            .map_or_else(Date::now, |c| c.date.clone());

        date.iterate_daily_to(&end_date)
            .find_map(|d| self.get_downloads_at(&d))
    }

    pub fn find_downloads_before_date(&self, date: &Date) -> Option<u32> {
        let start_date = self.data.added_commit.date.clone();

        date.iterate_daily_backwards(&start_date)
            .find_map(|d| self.get_downloads_at(&d))
    }

    pub fn released_in_month(&self, date: &Date) -> bool {
        self.data.added_commit.date.month == date.month
            && self.data.added_commit.date.year == date.year
    }

    pub fn removed_in_month(&self, date: &Date) -> bool {
        if let Some(removed_commit) = &self.data.removed_commit {
            removed_commit.date.month == date.month && removed_commit.date.year == date.year
        } else {
            false
        }
    }

    pub fn last_updated(&self) -> &Date {
        self.data
            .version_history
            .last()
            .map_or_else(|| &self.data.added_commit.date, |v| &v.initial_release_date)
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
            None => Some(format!(
                "https://publish.obsidian.md/hub/02+-+Community+Expansions/02.05+All+Community+Expansions/Plugins/{}",
                self.data.id
            )),
        }
    }

    pub fn added_commit(&self) -> StringCommit {
        self.data.added_commit.to_string_commit()
    }

    pub fn removed_commit(&self) -> Option<StringCommit> {
        self.data
            .removed_commit
            .as_ref()
            .map(|c| c.to_string_commit())
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
    pub fn download_data_points(&self) -> Vec<DownloadDataPoint> {
        let end_date = self
            .data
            .removed_commit
            .as_ref()
            .map_or_else(|| Date::now(), |c| c.date.clone());

        self.data
            .added_commit
            .date
            .iterate_weekly_to(&end_date)
            .map(|date| {
                let mut prev_date = date.clone();
                prev_date.reverse_days(7);

                let downloads = self
                    .find_downloads_in_week(&date)
                    .and_then(|d| if d > 0 { Some(d) } else { None });
                let previous_downloads = self
                    .find_downloads_in_week(&prev_date)
                    .and_then(|d| if d > 0 { Some(d) } else { None });

                let delta = match (downloads, previous_downloads) {
                    (Some(d), Some(pd)) if d >= pd => Some(d - pd),
                    _ => None,
                };

                DownloadDataPoint {
                    date: date.to_fancy_string(),
                    downloads,
                    delta,
                }
            })
            .collect()
    }

    pub fn warnings(&self) -> Vec<PluginWarning> {
        get_plugin_warnings(&self)
    }

    pub fn versions(&self) -> Vec<VersionDataPoint> {
        self.data
            .version_history
            .iter()
            .map(|v| VersionDataPoint {
                version: v.version.clone(),
                date: v.initial_release_date.to_fancy_string(),
                deprecated: self
                    .extended
                    .as_ref()
                    .map(|e| e.deprecated_versions.contains(&v.version))
                    .unwrap_or(false),
            })
            .collect()
    }

    pub fn changes(&self) -> Vec<EntryChangeDataPoint> {
        self.data
            .change_history
            .iter()
            .map(|change| change.to_data_point())
            .collect()
    }
}

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct FullPluginDataArray {
    #[wasm_bindgen(skip)]
    pub data: Vec<FullPluginData>,
}

impl FullPluginDataArray {
    pub fn new(data: Vec<PluginData>, extended: Vec<PluginExtraData>) -> Self {
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
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

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
    pub fn total_download_data(&self, data: &FullPluginDataArray) -> Vec<DownloadDataPoint> {
        let start_date = Date::new(2020, 11, 1);
        let end_date = Date::now();

        start_date
            .iterate_weekly_to(&end_date)
            .map(|date| {
                let mut prev_date = date.clone();
                prev_date.reverse_days(7);

                let downloads = self.data.iter().fold(0, |acc, index| {
                    acc + data[*index].find_downloads_in_week(&date).unwrap_or(0)
                });
                let previous_downloads = self.data.iter().fold(0, |acc, index| {
                    acc + data[*index].find_downloads_in_week(&prev_date).unwrap_or(0)
                });

                let downloads = if downloads > 0 { Some(downloads) } else { None };

                let previous_downloads = if previous_downloads > 0 {
                    Some(previous_downloads)
                } else {
                    None
                };

                let delta = match (downloads, previous_downloads) {
                    (Some(d), Some(pd)) if d >= pd => Some(d - pd),
                    _ => None,
                };

                DownloadDataPoint {
                    date: date.to_fancy_string(),
                    downloads,
                    delta,
                }
            })
            .collect()
    }

    pub fn individual_download_data(
        &self,
        data: &FullPluginDataArray,
    ) -> Vec<IndividualDownloadDataPoint> {
        self.data
            .iter()
            .map(|&index| {
                let plugin_data = &data[index];
                let id = plugin_data.id();
                let name = plugin_data.name();
                let date = plugin_data.added_commit().date;
                let downloads = plugin_data.download_count();
                let version_count = plugin_data.data.version_history.len() as u32;

                IndividualDownloadDataPoint {
                    id,
                    name,
                    date,
                    downloads,
                    version_count,
                }
            })
            .collect()
    }

    pub fn overview(&self, data: &FullPluginDataArray) -> Vec<PluginOverviewDataPoint> {
        self.data
            .iter()
            .map(|&index| {
                let plugin_data = &data[index];
                PluginOverviewDataPoint {
                    id: plugin_data.id(),
                    name: plugin_data.name(),
                    author: plugin_data.author(),
                    repo: plugin_data.data.current_entry.repo.clone(),
                    repo_url: plugin_data.repo_url(),
                    added_commit: plugin_data.added_commit(),
                    removed_commit: plugin_data.removed_commit(),
                }
            })
            .collect()
    }

    pub fn most_downloaded(
        &self,
        data: &FullPluginDataArray,
        count: usize,
        year: Option<u32>,
        restrict_release_date: bool,
    ) -> Vec<PluginYearlyDataPoint> {
        let (start_date, end_date) = match year {
            Some(y) => (Date::new(y, 1, 1), Date::new(y + 1, 1, 1)),
            None => (Date::new(2020, 11, 1), Date::now()),
        };

        let mut tmp = self
            .data
            .iter()
            .filter_map(|&index| {
                let plugin_data = &data[index];
                if restrict_release_date
                    && (plugin_data.data.added_commit.date < start_date
                        || plugin_data.data.added_commit.date > end_date)
                {
                    return None;
                }

                let downloads_start_date = plugin_data.find_downloads_after_date(&start_date)?;
                let downloads_end_date = plugin_data.find_downloads_before_date(&end_date)?;

                let downloads_new = downloads_end_date as i64 - downloads_start_date as i64;

                if downloads_new < 0 {
                    return None;
                }

                Some((index, downloads_new as u32, downloads_start_date))
            })
            .collect_vec();
        tmp.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by downloads_new in descending order
        tmp.truncate(count);

        tmp.into_iter()
            .map(|(index, downloads_new, downloads_start_date)| {
                let plugin_data = &data[index];

                let data = start_date
                    .iterate_weekly_to(&end_date)
                    .filter_map(|date| {
                        let downloads = plugin_data.find_downloads_in_week(&date)? as i64
                            - downloads_start_date as i64;
                        if downloads >= 0 {
                            Some(DownloadDataPoint {
                                date: date.to_fancy_string(),
                                downloads: Some(downloads as u32),
                                delta: None, // Delta is not calculated here
                            })
                        } else {
                            None
                        }
                    })
                    .collect();

                PluginYearlyDataPoint {
                    id: plugin_data.id(),
                    name: plugin_data.name(),
                    downloads_new,
                    downloads_start: downloads_start_date,
                    data,
                }
            })
            .collect_vec()
    }

    pub fn monthly_plugin_count(
        &self,
        data: &FullPluginDataArray,
    ) -> Vec<PluginCountMonthlyDataPoint> {
        let mut plugin_count: i32 = 0;
        let mut plugin_count_with_removed: i32 = 0;

        let start_date = Date::new(2020, 11, 1);
        let end_date = Date::now();

        start_date
            .iterate_monthly_to(&end_date)
            .map(|date| {
                let mut new_plugins = 0;
                let mut removed_plugins = 0;

                for index in &self.data {
                    let plugin_data = &data[*index];
                    if plugin_data.released_in_month(&date) {
                        new_plugins += 1;
                    }
                    if plugin_data.removed_in_month(&date) {
                        removed_plugins += 1;
                    }
                }

                plugin_count += new_plugins - removed_plugins;
                plugin_count_with_removed += new_plugins;

                PluginCountMonthlyDataPoint {
                    date: date.to_fancy_string(),
                    total: plugin_count.max(0) as u32,
                    total_with_removed: plugin_count_with_removed.max(0) as u32,
                    new: new_plugins.max(0) as u32,
                    new_removed: removed_plugins.max(0) as u32,
                }
            })
            .collect()
    }

    pub fn removed_by_release_month(
        &self,
        data: &FullPluginDataArray,
    ) -> Vec<PluginRemovedByReleaseDataPoint> {
        let start_date = Date::new(2020, 11, 1);
        let end_date = Date::now();

        start_date
            .iterate_monthly_to(&end_date)
            .map(|date| {
                let mut removed_count = 0;
                let mut count = 0;

                self.data.iter().for_each(|&index| {
                    let plugin_data = &data[index];
                    if plugin_data.released_in_month(&date) {
                        count += 1;
                        if plugin_data.data.removed_commit.is_some() {
                            removed_count += 1;
                        }
                    }
                });

                PluginRemovedByReleaseDataPoint {
                    date: date.to_fancy_string(),
                    percentage: if count > 0 {
                        (removed_count as f64 / count as f64) * 100.0
                    } else {
                        0.0
                    },
                }
            })
            .collect()
    }

    pub fn inactivity_by_release_month(
        &self,
        data: &FullPluginDataArray,
    ) -> Vec<PluginInactivityByReleaseDataPoint> {
        let start_date = Date::new(2020, 11, 1);
        let end_date = Date::now();

        start_date
            .iterate_monthly_to(&end_date)
            .map(|date| {
                let mut inactive = [0; 5];
                let mut released_in_month = 0;

                self.data.iter().for_each(|&index| {
                    let plugin_data = &data[index];
                    if plugin_data.released_in_month(&date) {
                        released_in_month += 1;
                        let last_updated_date = plugin_data.last_updated();

                        for i in (0..5).rev() {
                            let mut i_years_ago = Date::now();
                            i_years_ago.reverse_days(365 * (i + 1));
                            if last_updated_date < &i_years_ago {
                                inactive[i as usize] += 1;
                                break;
                            }
                        }
                    }
                });

                PluginInactivityByReleaseDataPoint {
                    date: date.to_fancy_string(),
                    inactive_one_year: inactive[0] as f64 / released_in_month as f64 * 100.0,
                    inactive_two_years: inactive[1] as f64 / released_in_month as f64 * 100.0,
                    inactive_three_years: inactive[2] as f64 / released_in_month as f64 * 100.0,
                    inactive_four_years: inactive[3] as f64 / released_in_month as f64 * 100.0,
                    inactive_five_years: inactive[4] as f64 / released_in_month as f64 * 100.0,
                }
            })
            .collect()
    }

    pub fn inactivity_distribution(&self, data: &FullPluginDataArray) -> Vec<i32> {
        let mut tmp: Vec<_> = self
            .data
            .iter()
            .map(|&index| {
                let plugin_data = &data[index];

                let last_updated = plugin_data.last_updated();

                Date::now().diff_in_days(last_updated).abs()
            })
            .collect();
        tmp.sort_by(|a, b| b.cmp(a));
        tmp
    }

    pub fn repo_data_points(&self, data: &FullPluginDataArray) -> PluginRepoDataPoints {
        let mut points = PluginRepoDataPoints::default();

        self.data.iter().for_each(|&index| {
            let plugin_data = &data[index];
            let Some(repo_data) = plugin_data.repo_data() else {
                return;
            };

            for package_manager in &repo_data.package_managers {
                increment_named_data_points(
                    &mut points.package_managers,
                    package_manager.get_identifier(),
                    1.0,
                );
            }
            if repo_data.package_managers.is_empty() {
                points.no_package_managers += 1.0;
            }

            for bundler in &repo_data.bundlers {
                increment_named_data_points(&mut points.bundlers, bundler.get_identifier(), 1.0);
            }
            if repo_data.bundlers.is_empty() {
                points.no_bundlers += 1.0;
            }

            for testing_framework in &repo_data.testing_frameworks {
                increment_named_data_points(
                    &mut points.testing_frameworks,
                    testing_framework.get_identifier(),
                    1.0,
                );
            }
            if repo_data.testing_frameworks.is_empty() {
                points.no_testing_frameworks += 1.0;
            }

            for dependency in &repo_data.dependencies {
                increment_named_data_points(&mut points.dependencies, dependency, 1.0);
            }
            for dev_dependency in &repo_data.dev_dependencies {
                increment_named_data_points(&mut points.dependencies, dev_dependency, 1.0);
            }

            if repo_data.uses_typescript {
                points.typescript += 1.0;
            }
            if repo_data.has_beta_manifest {
                points.beta_manifest += 1.0;
            }
        });

        // now turn everything into percentages
        let total_plugins = self.data.len() as f64;
        points.package_managers.iter_mut().for_each(|point| {
            to_percentage(&mut point.value, total_plugins);
        });
        points.bundlers.iter_mut().for_each(|point| {
            to_percentage(&mut point.value, total_plugins);
        });
        points.testing_frameworks.iter_mut().for_each(|point| {
            to_percentage(&mut point.value, total_plugins);
        });
        points.dependencies.iter_mut().for_each(|point| {
            to_percentage(&mut point.value, total_plugins);
        });
        to_percentage(&mut points.no_package_managers, total_plugins);
        to_percentage(&mut points.no_bundlers, total_plugins);
        to_percentage(&mut points.no_testing_frameworks, total_plugins);
        to_percentage(&mut points.typescript, total_plugins);
        to_percentage(&mut points.beta_manifest, total_plugins);

        points
    }

    pub fn license_data_points(
        &self,
        data: &FullPluginDataArray,
        license_data_string: String,
    ) -> Result<PluginLicenseDataPoints, String> {
        let licenses: Licenses = serde_json::from_str(&license_data_string)
            .map_err(|e| format!("Failed to parse license data: {}", e))?;

        let mut points = PluginLicenseDataPoints {
            licenses: Vec::new(),
            permissions: Vec::new(),
            conditions: Vec::new(),
            limitations: Vec::new(),
            descriptions: licenses.descriptions,
        };

        self.data.iter().for_each(|&index| {
            let plugin_data = &data[index];
            let Some(repo_data) = plugin_data.repo_data() else {
                return;
            };

            let license_data = licenses
                .licenses
                .iter()
                .find(|l| l.spdx_id == repo_data.license_file);

            if let Some(license_data) = license_data {
                increment_named_data_points(&mut points.licenses, &license_data.spdx_id, 1.0);

                for permission in &license_data.permissions {
                    increment_named_data_points(&mut points.permissions, permission, 1.0);
                }
                for condition in &license_data.conditions {
                    increment_named_data_points(&mut points.conditions, condition, 1.0);
                }
                for limitation in &license_data.limitations {
                    increment_named_data_points(&mut points.limitations, limitation, 1.0);
                }
            }
        });

        Ok(points)
    }
}

fn increment_named_data_points(points: &mut Vec<NamedDataPoint>, name: &str, value: f64) {
    if let Some(point) = points.iter_mut().find(|p| p.name == name) {
        point.value += value;
    } else {
        points.push(NamedDataPoint {
            name: name.to_string(),
            value,
        });
    }
}

fn to_percentage(value: &mut f64, total: f64) {
    if total == 0.0 {
        *value = 0.0;
    } else {
        *value = (*value / total) * 100.0;
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
