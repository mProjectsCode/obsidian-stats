use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    common::{
        CountMonthlyDataPoint, DownloadDataPoint, HallOfFameDataPoint,
        InactivityByReleaseDataPoint, IndividualDownloadDataPoint, LOC_EXCLUDED,
        MilestoneDataPoint, MilestoneMonthGroup, NamedDataPoint, OverviewDataPoint,
        RemovedByReleaseDataPoint, increment_named_data_points,
    },
    date::Date,
    plugin::milestones,
};

use super::{PluginDataArray, PluginDataArrayView};

#[wasm_bindgen]
impl PluginDataArrayView {
    /// Get the total downloads (sum of all plugins in the view).
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
                let total_loc = item.repo_data().map_or(0, |repo| {
                    repo.lines_of_code
                        .iter()
                        .filter(|(lang, _)| !LOC_EXCLUDED.contains(&lang.as_str()))
                        .map(|(_, loc)| loc)
                        .sum()
                }) as u32;

                IndividualDownloadDataPoint {
                    id: item.id(),
                    name: item.name(),
                    date: item.added_commit().date,
                    downloads: item.download_count(),
                    version_count: item.listed_version_count() as u32,
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
        tmp.sort_by_key(|entry| std::cmp::Reverse(entry.1));
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
                                delta: None,
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
                            if last_updated_date < i_years_ago {
                                inactive[i as usize] += 1;
                                break;
                            }
                        }
                    }
                });

                if released_in_month == 0 {
                    return InactivityByReleaseDataPoint {
                        date: date.to_fancy_string(),
                        inactive_one_year: 0.0,
                        inactive_two_years: 0.0,
                        inactive_three_years: 0.0,
                        inactive_four_years: 0.0,
                        inactive_five_years: 0.0,
                    };
                }

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
            .map(|item| Date::now().diff_in_days(&item.last_updated()).abs())
            .collect();
        tmp.sort_by(|a, b| b.cmp(a));
        tmp
    }

    pub fn updates_weekly(&self, data: &PluginDataArray) -> Vec<NamedDataPoint> {
        let mut points = vec![];

        self.iter_data(data).for_each(|item| {
            item.data
                .version_history
                .iter()
                .filter(|version| version.released_while_listed)
                .for_each(|version| {
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

    /// Calculate all milestones reached, grouped by month and sorted newest first
    pub fn calculate_milestones(&self, data: &PluginDataArray) -> Vec<MilestoneMonthGroup> {
        let milestone_data = milestones::calculate_milestones(data, self);

        let mut result: Vec<_> = milestone_data
            .into_iter()
            .map(|(month, milestones_in_month)| {
                let milestone_points: Vec<MilestoneDataPoint> = milestones_in_month
                    .into_iter()
                    .map(|m| MilestoneDataPoint {
                        milestone_type: m.milestone_type.as_str().to_string(),
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

        result.sort_by(|a, b| b.month.cmp(&a.month));
        result
    }
}
