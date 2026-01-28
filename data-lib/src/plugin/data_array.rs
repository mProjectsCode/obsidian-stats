use std::ops::Index;

use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    common::{
        CountMonthlyDataPoint, DownloadDataPoint, FILE_EXT_INCLUDED, HallOfFameDataPoint,
        InactivityByReleaseDataPoint, IndividualDownloadDataPoint, LOC_EXCLUDED,
        MilestoneDataPoint, MilestoneMonthGroup, OverviewDataPoint, RemovedByReleaseDataPoint,
        increment_named_data_points, to_percentage,
    },
    date::Date,
    license::Licenses,
    plugin::{
        LicenseInfo, NamedDataPoint, PluginData, PluginExtraData, PluginLicenseDataPoints,
        PluginRepoDataPoints, full::FullPluginData, milestones,
    },
};

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct PluginDataArray {
    #[wasm_bindgen(skip)]
    pub data: Vec<FullPluginData>,
}

impl PluginDataArray {
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
    pub fn total_download_data(&self, data: &PluginDataArray) -> Vec<DownloadDataPoint> {
        let start_date = Date::new(2020, 11, 1);
        let end_date = Date::now();

        start_date
            .iterate_weekly_to(&end_date)
            .map(|date| {
                let mut prev_date = date.clone();
                prev_date.reverse_days(7);

                let downloads = self.indices.iter().fold(0, |acc, index| {
                    acc + data[*index].find_downloads_in_week(&date).unwrap_or(0)
                });
                let previous_downloads = self.indices.iter().fold(0, |acc, index| {
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
        data: &PluginDataArray,
    ) -> Vec<IndividualDownloadDataPoint> {
        self.iter_data(data)
            .map(|item| {
                let id = item.id();
                let name = item.name();
                let date = item.added_commit().date;
                let downloads = item.download_count();
                let version_count = item.data.version_history.len() as u32;
                let total_loc = item.repo_data().map_or(0, |repo| {
                    repo.lines_of_code
                        .iter()
                        .filter(|(lang, _)| !LOC_EXCLUDED.contains(&lang.as_str()))
                        .map(|(_, loc)| loc)
                        .sum()
                }) as u32;

                IndividualDownloadDataPoint {
                    id,
                    name,
                    date,
                    downloads,
                    version_count,
                    total_loc,
                }
            })
            .collect()
    }

    pub fn overview(&self, data: &PluginDataArray) -> Vec<OverviewDataPoint> {
        self.iter_data(data)
            .map(|item| OverviewDataPoint {
                id: item.id(),
                name: item.name(),
                author: item.author(),
                repo: item.repo(),
                repo_url: item.repo_url(),
                added_commit: item.added_commit(),
                removed_commit: item.removed_commit(),
            })
            .collect()
    }

    pub fn most_downloaded(
        &self,
        data: &PluginDataArray,
        count: usize,
        year: Option<u32>,
        restrict_release_date: bool,
    ) -> Vec<HallOfFameDataPoint> {
        let (start_date, end_date) = match year {
            Some(y) => (Date::new(y, 1, 1), Date::new(y + 1, 1, 1)),
            None => (Date::new(2020, 11, 1), Date::now()),
        };

        let mut tmp = self
            .indices
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
            .collect::<Vec<_>>();
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

                HallOfFameDataPoint {
                    id: plugin_data.id(),
                    name: plugin_data.name(),
                    downloads_new,
                    downloads_start: downloads_start_date,
                    data,
                }
            })
            .collect()
    }

    pub fn monthly_count(&self, data: &PluginDataArray) -> Vec<CountMonthlyDataPoint> {
        let mut plugin_count: i32 = 0;
        let mut plugin_count_with_removed: i32 = 0;

        let start_date = Date::new(2020, 11, 1);
        let end_date = Date::now();

        start_date
            .iterate_monthly_to(&end_date)
            .map(|date| {
                let mut new_plugins = 0;
                let mut removed_plugins = 0;

                for index in &self.indices {
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

                CountMonthlyDataPoint {
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
        data: &PluginDataArray,
    ) -> Vec<RemovedByReleaseDataPoint> {
        let start_date = Date::new(2020, 11, 1);
        let end_date = Date::now();

        start_date
            .iterate_monthly_to(&end_date)
            .map(|date| {
                let mut removed_count = 0;
                let mut count = 0;

                self.indices.iter().for_each(|&index| {
                    let plugin_data = &data[index];
                    if plugin_data.released_in_month(&date) {
                        count += 1;
                        if plugin_data.data.removed_commit.is_some() {
                            removed_count += 1;
                        }
                    }
                });

                RemovedByReleaseDataPoint {
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
        data: &PluginDataArray,
    ) -> Vec<InactivityByReleaseDataPoint> {
        let start_date = Date::new(2020, 11, 1);
        let end_date = Date::now();

        start_date
            .iterate_monthly_to(&end_date)
            .map(|date| {
                let mut inactive = [0; 5];
                let mut released_in_month = 0;

                self.indices.iter().for_each(|&index| {
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

                InactivityByReleaseDataPoint {
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

    pub fn inactivity_distribution(&self, data: &PluginDataArray) -> Vec<i32> {
        let mut tmp: Vec<_> = self
            .iter_data(data)
            .map(|item| {
                let last_updated = item.last_updated();

                Date::now().diff_in_days(last_updated).abs()
            })
            .collect();
        tmp.sort_by(|a, b| b.cmp(a));
        tmp
    }

    pub fn updates_weekly(&self, data: &PluginDataArray) -> Vec<NamedDataPoint> {
        let mut points = vec![];

        self.iter_data(data).for_each(|item| {
            item.data.version_history.iter().for_each(|version| {
                increment_named_data_points(
                    &mut points,
                    &version.initial_release_date.week_start().to_fancy_string(),
                    1.0,
                );
            });
        });

        points.sort_by(|a, b| a.name.cmp(&b.name));

        points
    }

    pub fn repo_data_points(&self, data: &PluginDataArray) -> PluginRepoDataPoints {
        let mut points = PluginRepoDataPoints::default();

        self.iter_data(data).for_each(|item| {
            let Some(repo_data) = item.repo_data() else {
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
        let total_plugins = self.indices.len() as f64;
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
        data: &PluginDataArray,
        license_data_string: String,
    ) -> Result<PluginLicenseDataPoints, String> {
        let licenses: Licenses = serde_json::from_str(&license_data_string)
            .map_err(|e| format!("Failed to parse license data: {e}"))?;

        let mut points = PluginLicenseDataPoints {
            licenses: Vec::new(),
            permissions: Vec::new(),
            conditions: Vec::new(),
            limitations: Vec::new(),
            descriptions: licenses.descriptions,
        };

        self.iter_data(data).for_each(|item| {
            let Some(repo_data) = item.repo_data() else {
                return;
            };

            match &repo_data.file_license {
                LicenseInfo::Known(name) => {
                    let license_data = licenses.licenses.iter().find(|l| *name == l.spdx_id);

                    if let Some(license_data) = license_data {
                        increment_named_data_points(
                            &mut points.licenses,
                            &license_data.spdx_id,
                            1.0,
                        );

                        for permission in &license_data.permissions {
                            increment_named_data_points(&mut points.permissions, permission, 1.0);
                        }
                        for condition in &license_data.conditions {
                            increment_named_data_points(&mut points.conditions, condition, 1.0);
                        }
                        for limitation in &license_data.limitations {
                            increment_named_data_points(&mut points.limitations, limitation, 1.0);
                        }
                    } else {
                        // I think we should never hit this path, as we make sure that the license is known during data extraction.
                        increment_named_data_points(
                            &mut points.licenses,
                            &LicenseInfo::Unrecognized.to_fancy_string(),
                            1.0,
                        );
                    }
                }
                other => {
                    increment_named_data_points(
                        &mut points.licenses,
                        &other.to_fancy_string(),
                        1.0,
                    );
                }
            }
        });

        Ok(points)
    }

    /// Named data points for mismatched data between the plugin's repo data and the current entry in the community list.
    /// The data is in percentage form.
    pub fn mismatched_data(&self, data: &PluginDataArray) -> Vec<NamedDataPoint> {
        let mut points = Vec::new();
        self.iter_data(data).for_each(|item| {
            let Some(repo_data) = item.repo_data() else {
                return;
            };

            if repo_data.manifest.description != item.data.current_entry.description {
                increment_named_data_points(&mut points, "Description mismatch", 1.0);
            }

            if repo_data.manifest.name != item.data.current_entry.name {
                increment_named_data_points(&mut points, "Name mismatch", 1.0);
            }

            if repo_data.manifest.author != item.data.current_entry.author {
                increment_named_data_points(&mut points, "Author mismatch", 1.0);
            }
        });

        points.iter_mut().for_each(|point| {
            to_percentage(&mut point.value, self.indices.len() as f64);
        });

        points
    }

    /// Usage percentages of optional manifest fields across all plugins in the view.
    pub fn optional_manifest_fields(&self, data: &PluginDataArray) -> Vec<NamedDataPoint> {
        let mut points = Vec::new();
        self.iter_data(data).for_each(|item| {
            let Some(repo_data) = item.repo_data() else {
                return;
            };

            if repo_data.manifest.funding_url.is_some() {
                increment_named_data_points(&mut points, "Has funding URL", 1.0);
            }
            if repo_data.manifest.author_url.is_some() {
                increment_named_data_points(&mut points, "Has author URL", 1.0);
            }
            if repo_data.manifest.help_url.is_some() {
                increment_named_data_points(&mut points, "Has help URL", 1.0);
            }
        });

        points.iter_mut().for_each(|point| {
            to_percentage(&mut point.value, self.indices.len() as f64);
        });

        points
    }

    pub fn desktop_only_data(&self, data: &PluginDataArray) -> Vec<NamedDataPoint> {
        let mut points = Vec::new();
        self.iter_data(data).for_each(|item| {
            let Some(repo_data) = item.repo_data() else {
                increment_named_data_points(&mut points, "Unknown", 1.0);
                return;
            };

            match repo_data.manifest.is_desktop_only {
                Some(true) => {
                    increment_named_data_points(&mut points, "Desktop only", 1.0);
                }
                Some(false) => {
                    increment_named_data_points(&mut points, "Mobile compatible", 1.0);
                }
                None => {
                    increment_named_data_points(&mut points, "Not specified", 1.0);
                }
            }
        });

        points.iter_mut().for_each(|point| {
            to_percentage(&mut point.value, self.indices.len() as f64);
        });

        points
    }

    pub fn lines_of_code_by_language(&self, data: &PluginDataArray) -> Vec<NamedDataPoint> {
        let mut points = Vec::new();

        self.iter_data(data).for_each(|item| {
            let Some(repo_data) = item.repo_data() else {
                return;
            };

            repo_data.lines_of_code.iter().for_each(|(lang, count)| {
                increment_named_data_points(&mut points, lang, *count as f64);
            });
        });

        points
            .into_iter()
            .filter(|point| point.value > 10_000.0 && !LOC_EXCLUDED.contains(&point.name.as_str()))
            .collect()
    }

    pub fn lines_of_code_by_language_usage(&self, data: &PluginDataArray) -> Vec<NamedDataPoint> {
        let mut points = Vec::new();

        self.iter_data(data).for_each(|item| {
            let Some(repo_data) = item.repo_data() else {
                return;
            };

            repo_data.lines_of_code.iter().for_each(|(lang, _)| {
                increment_named_data_points(&mut points, lang, 1.0);
            });
        });

        points
            .into_iter()
            .filter(|point| !LOC_EXCLUDED.contains(&point.name.as_str()))
            .collect()
    }

    pub fn file_count_by_extension(&self, data: &PluginDataArray) -> Vec<NamedDataPoint> {
        let mut points = Vec::new();

        self.iter_data(data).for_each(|item| {
            let Some(repo_data) = item.repo_data() else {
                return;
            };

            repo_data.file_type_counts.iter().for_each(|(ext, count)| {
                increment_named_data_points(&mut points, ext, *count as f64);
            });
        });

        points
            .into_iter()
            .filter(|point| FILE_EXT_INCLUDED.contains(&point.name.to_lowercase().as_str()))
            .collect()
    }

    pub fn lines_of_code_distribution(&self, data: &PluginDataArray) -> Vec<u32> {
        let mut tmp: Vec<_> = self
            .iter_data(data)
            .map(|item| {
                let Some(repo_data) = item.repo_data() else {
                    return 0;
                };

                repo_data
                    .lines_of_code
                    .iter()
                    .filter(|(lang, _)| !LOC_EXCLUDED.contains(&lang.as_str()))
                    .map(|(_, loc)| loc)
                    .sum::<usize>() as u32
            })
            .filter(|&count| count > 0)
            .collect();

        tmp.sort_by(|a, b| b.cmp(a));
        tmp
    }

    pub fn file_count_distribution(&self, data: &PluginDataArray) -> Vec<u32> {
        let mut tmp: Vec<_> = self
            .iter_data(data)
            .map(|item| {
                let Some(repo_data) = item.repo_data() else {
                    return 0;
                };

                repo_data
                    .file_type_counts
                    .iter()
                    .filter(|(ext, _)| FILE_EXT_INCLUDED.contains(&ext.to_lowercase().as_str()))
                    .map(|(_, count)| count)
                    .sum::<usize>() as u32
            })
            .filter(|&count| count > 0)
            .collect();

        tmp.sort_by(|a, b| b.cmp(a));
        tmp
    }

    pub fn i18n_usage(&self, data: &PluginDataArray) -> Vec<NamedDataPoint> {
        let mut data_points = Vec::new();

        self.iter_data(data).for_each(|item| {
            let Some(repo_data) = item.repo_data() else {
                return;
            };

            if repo_data.has_i18n_dependencies && repo_data.has_i18n_files {
                increment_named_data_points(
                    &mut data_points,
                    "Has i18n dependencies and files",
                    1.0,
                );
            } else if repo_data.has_i18n_dependencies {
                increment_named_data_points(&mut data_points, "Has i18n dependencies", 1.0);
            } else if repo_data.has_i18n_files {
                increment_named_data_points(&mut data_points, "Has i18n files", 1.0);
            }
        });

        data_points
    }

    pub fn i18n_plugin_ids(&self, data: &PluginDataArray) -> Vec<String> {
        let mut ids = Vec::new();

        self.iter_data(data).for_each(|item| {
            let Some(repo_data) = item.repo_data() else {
                return;
            };

            if repo_data.has_i18n_dependencies || repo_data.has_i18n_files {
                ids.push(item.id());
            }
        });

        ids.sort();

        ids
    }

    /// Calculate all milestones reached, grouped by month and sorted newest first
    pub fn calculate_milestones(&self, data: &PluginDataArray) -> Vec<MilestoneMonthGroup> {
        let milestone_data = milestones::calculate_milestones(data, self);

        // Convert to Vec and sort by month (newest first)
        let mut result: Vec<_> = milestone_data
            .into_iter()
            .map(|(month, milestones_in_month)| {
                let milestone_points: Vec<MilestoneDataPoint> = milestones_in_month
                    .into_iter()
                    .map(|m| MilestoneDataPoint {
                        milestone_type: m.milestone_type.to_string().to_string(),
                        milestone_value: m.milestone_value,
                        date: m.date.to_fancy_string(),
                        plugin_id: m.plugin_id,
                    })
                    .collect();

                MilestoneMonthGroup {
                    month,
                    milestones: milestone_points,
                }
            })
            .collect();

        // Sort by month in descending order (newest first)
        result.sort_by(|a, b| b.month.cmp(&a.month));

        result
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
