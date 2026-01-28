use std::ops::Range;

use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use crate::{
    date::Date,
    plugin::{data_array::PluginDataArray, data_array::PluginDataArrayView},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MilestoneType {
    PluginCount,
    TotalDownloads,
    ReleaseCount,
    DownloadCount,
}

impl MilestoneType {
    pub fn to_string(&self) -> &'static str {
        match self {
            MilestoneType::PluginCount => "plugin-count",
            MilestoneType::TotalDownloads => "total-downloads",
            MilestoneType::ReleaseCount => "release-count",
            MilestoneType::DownloadCount => "download-count",
        }
    }

    pub fn all() -> Vec<MilestoneType> {
        vec![
            MilestoneType::PluginCount,
            MilestoneType::TotalDownloads,
            MilestoneType::ReleaseCount,
            MilestoneType::DownloadCount,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneData {
    pub milestone_type: MilestoneType,
    pub milestone_value: u32,
    pub date: Date,
    pub plugin_id: Option<String>,
}

/// Generate milestone values from powers of 10 and step multipliers
/// For example: potentials=[1, 2], steps=[1, 2, 5] generates [10, 20, 50, 100, 200, 500]
pub fn generate_milestones(potentials: Range<u32>, steps: &[u32]) -> Vec<u32> {
    potentials
        .into_iter()
        .flat_map(|potential| {
            steps.iter().map(move |&step| {
                let base: u32 = 10;
                base.pow(potential) * step
            })
        })
        .collect()
}

/// Get the milestones for a specific type
pub fn get_milestones_for_type(milestone_type: MilestoneType) -> Vec<u32> {
    let steps = [1, 2, 5];
    match milestone_type {
        MilestoneType::PluginCount => generate_milestones(2..6, &steps),
        MilestoneType::TotalDownloads => generate_milestones(4..10, &steps),
        MilestoneType::ReleaseCount => generate_milestones(1..6, &steps),
        MilestoneType::DownloadCount => generate_milestones(3..10, &steps),
    }
}

/// Check if any new milestones have been reached and return them
pub fn check_milestones_reached(
    current_value: u32,
    milestones: &[u32],
    last_reached_index: &mut Option<usize>,
) -> Vec<u32> {
    let mut reached = Vec::new();
    let start_index = last_reached_index.map(|i| i + 1).unwrap_or(0);

    for (idx, &milestone) in milestones.iter().enumerate().skip(start_index) {
        if current_value >= milestone {
            reached.push(milestone);
            *last_reached_index = Some(idx);
        } else {
            break;
        }
    }

    reached
}

/// Get the count of non-retired plugins at a specific date
pub fn get_plugin_count_at_date(
    date: &Date,
    data: &PluginDataArray,
    view: &PluginDataArrayView,
) -> u32 {
    let mut count = 0;
    for plugin in view.iter_data(data) {
        // Plugin must be added before or on this date
        if plugin.data.added_commit.date <= *date {
            // Plugin must not be removed, or removed after this date
            if let Some(removed) = &plugin.data.removed_commit {
                if removed.date > *date {
                    count += 1;
                }
            } else {
                count += 1;
            }
        }
    }
    count
}

/// Get the total downloads across all plugins at a specific date
pub fn get_total_downloads_at_date(
    date: &Date,
    data: &PluginDataArray,
    view: &PluginDataArrayView,
) -> Option<u32> {
    let mut total = 0;
    let mut found_any = false;

    for plugin in view.iter_data(data) {
        if let Some(downloads) = plugin.find_downloads_in_week(date) {
            total += downloads;
            found_any = true;
        }
    }

    if found_any { Some(total) } else { None }
}

/// Get the plugin with the maximum release count at a specific date
pub fn get_max_release_count_at_date(
    date: &Date,
    data: &PluginDataArray,
    view: &PluginDataArrayView,
) -> Option<(u32, String)> {
    let date_string = date.to_fancy_string();
    let mut max_count = 0;
    let mut max_plugin_id = String::new();

    for plugin in view.iter_data(data) {
        let release_count = plugin
            .data
            .version_history
            .iter()
            .filter(|v| v.initial_release_date.to_fancy_string() <= date_string)
            .count() as u32;

        if release_count > max_count {
            max_count = release_count;
            max_plugin_id = plugin.data.id.clone();
        }
    }

    if max_count > 0 {
        Some((max_count, max_plugin_id))
    } else {
        None
    }
}

/// Get the plugin with the maximum download count at a specific date
pub fn get_max_download_count_at_date(
    date: &Date,
    data: &PluginDataArray,
    view: &PluginDataArrayView,
) -> Option<(u32, String)> {
    let date_string = date.to_fancy_string();
    let mut max_count = 0;
    let mut max_plugin_id = String::new();

    for plugin in view.iter_data(data) {
        // Find the last download entry that is <= the target date
        let download_count = plugin
            .data
            .download_history
            .0
            .iter()
            .filter(|(d, _)| d.as_str() <= date_string.as_str())
            .map(|(_, count)| *count)
            .max()
            .unwrap_or(0);

        if download_count > max_count {
            max_count = download_count;
            max_plugin_id = plugin.data.id.clone();
        }
    }

    if max_count > 0 {
        Some((max_count, max_plugin_id))
    } else {
        None
    }
}

/// Calculate all milestones reached, grouped by month
pub fn calculate_milestones(
    data: &PluginDataArray,
    view: &PluginDataArrayView,
) -> HashMap<String, Vec<MilestoneData>> {
    let start_date = Date::new(2020, 1, 1);
    let end_date = Date::now();

    // Initialize milestone trackers
    let mut milestone_trackers: HashMap<MilestoneType, Option<usize>> = HashMap::new();
    for milestone_type in MilestoneType::all() {
        milestone_trackers.insert(milestone_type, None);
    }

    let mut result: HashMap<String, Vec<MilestoneData>> = HashMap::new();

    // Iterate through each month
    for date in start_date.iterate_monthly_to(&end_date) {
        let month_string = format!("{:04}-{:02}", date.year, date.month);

        // Check PLUGIN_COUNT milestones
        let plugin_count = get_plugin_count_at_date(&date, data, view);
        let milestones = get_milestones_for_type(MilestoneType::PluginCount);
        let last_reached = milestone_trackers
            .get_mut(&MilestoneType::PluginCount)
            .unwrap();
        let reached = check_milestones_reached(plugin_count, &milestones, last_reached);
        for milestone in reached {
            result
                .entry(month_string.clone())
                .or_default()
                .push(MilestoneData {
                    milestone_type: MilestoneType::PluginCount,
                    milestone_value: milestone,
                    date: date.clone(),
                    plugin_id: None,
                });
        }

        // Check TOTAL_DOWNLOADS milestones
        if let Some(total_downloads) = get_total_downloads_at_date(&date, data, view) {
            let milestones = get_milestones_for_type(MilestoneType::TotalDownloads);
            let last_reached = milestone_trackers
                .get_mut(&MilestoneType::TotalDownloads)
                .unwrap();
            let reached = check_milestones_reached(total_downloads, &milestones, last_reached);
            for milestone in reached {
                result
                    .entry(month_string.clone())
                    .or_default()
                    .push(MilestoneData {
                        milestone_type: MilestoneType::TotalDownloads,
                        milestone_value: milestone,
                        date: date.clone(),
                        plugin_id: None,
                    });
            }
        }

        // Check RELEASE_COUNT milestones
        if let Some((release_count, plugin_id)) = get_max_release_count_at_date(&date, data, view) {
            let milestones = get_milestones_for_type(MilestoneType::ReleaseCount);
            let last_reached = milestone_trackers
                .get_mut(&MilestoneType::ReleaseCount)
                .unwrap();
            let reached = check_milestones_reached(release_count, &milestones, last_reached);
            for milestone in reached {
                result
                    .entry(month_string.clone())
                    .or_default()
                    .push(MilestoneData {
                        milestone_type: MilestoneType::ReleaseCount,
                        milestone_value: milestone,
                        date: date.clone(),
                        plugin_id: Some(plugin_id.clone()),
                    });
            }
        }

        // Check DOWNLOAD_COUNT milestones
        if let Some((download_count, plugin_id)) = get_max_download_count_at_date(&date, data, view)
        {
            let milestones = get_milestones_for_type(MilestoneType::DownloadCount);
            let last_reached = milestone_trackers
                .get_mut(&MilestoneType::DownloadCount)
                .unwrap();
            let reached = check_milestones_reached(download_count, &milestones, last_reached);
            for milestone in reached {
                result
                    .entry(month_string.clone())
                    .or_default()
                    .push(MilestoneData {
                        milestone_type: MilestoneType::DownloadCount,
                        milestone_value: milestone,
                        date: date.clone(),
                        plugin_id: Some(plugin_id.clone()),
                    });
            }
        }
    }

    result
}
