use hashbrown::HashMap;
use regex::Regex;
use wasm_bindgen::prelude::*;

use crate::{
    common::{StackedNamedDataPoint, increment_named_data_points},
    iter_ext::{DedupExt, GroupByExt, SortExt},
    release::{
        ChangeLogChangeCategory, ChangelogChanges, ChangelogDataPoint, GithubReleaseInfo, OS,
        ObsidianPlatform, ObsidianReleaseInfo, ToFancyString, get_asset_cpu_instruction_set,
        get_asset_release_file_type,
    },
};

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct ReleaseDataArray {
    #[wasm_bindgen(skip)]
    pub raw_data: Vec<GithubReleaseInfo>,
    #[wasm_bindgen(skip)]
    pub interpolated_data: Vec<GithubReleaseInfo>,
    #[wasm_bindgen(skip)]
    pub changelog: Vec<ObsidianReleaseInfo>,
}

impl ReleaseDataArray {
    pub fn new(
        raw_data: Vec<GithubReleaseInfo>,
        interpolated_data: Vec<GithubReleaseInfo>,
        changelog: Vec<ObsidianReleaseInfo>,
    ) -> Self {
        Self {
            raw_data,
            interpolated_data,
            changelog,
        }
    }

    fn total_downloads_per_version_for_known_os(&self) -> Vec<(String, u64)> {
        self.raw_data
            .iter()
            .map(|release| {
                let downloads = release
                    .assets
                    .iter()
                    .filter(|asset| OS::from_asset_name(&asset.name).is_some())
                    .map(|asset| *asset.downloads.values().max().unwrap_or(&0) as u64)
                    .sum();

                (release.version.to_fancy_string(), downloads)
            })
            .collect()
    }

    fn get_total_downloads(&self) -> u64 {
        self.raw_data
            .iter()
            .flat_map(|release| release.assets.iter())
            .map(|asset| *asset.downloads.values().max().unwrap_or(&0) as u64)
            .sum()
    }

    fn get_total_asset_count(&self) -> u64 {
        self.raw_data
            .iter()
            .flat_map(|release| release.assets.iter())
            .count() as u64
    }
}

#[wasm_bindgen]
impl ReleaseDataArray {
    pub fn total_downloads_per_version_by_os(&self) -> Vec<StackedNamedDataPoint> {
        self.raw_data
            .iter()
            .flat_map(|release| {
                release
                    .assets
                    .iter()
                    .group_by(|asset| OS::from_asset_name(&asset.name))
                    .filter(|(os, _)| os.is_some())
                    .map(|(os, assets)| {
                        (
                            &release.version,
                            os.to_fancy_string(),
                            assets
                                .iter()
                                .map(|asset| *asset.downloads.values().max().unwrap_or(&0) as f64)
                                .sum(),
                        )
                    })
            })
            .sort_by(|a, b| a.0.cmp(&b.0))
            .map(|(version, os, value)| StackedNamedDataPoint {
                name: version.to_fancy_string(),
                layer: os,
                value,
            })
            .collect()
    }

    pub fn total_downloads_per_version_by_os_as_percentages(&self) -> Vec<StackedNamedDataPoint> {
        let total_downloads = self.total_downloads_per_version_by_os();

        let downloads_per_version = self.total_downloads_per_version_for_known_os();

        total_downloads
            .into_iter()
            .map(|point| {
                let total = downloads_per_version
                    .iter()
                    .find(|(version, _)| version == &point.name)
                    .map_or(0, |(_, downloads)| *downloads);
                StackedNamedDataPoint {
                    name: point.name,
                    layer: point.layer,
                    value: if total == 0 {
                        0.0
                    } else {
                        (point.value / total as f64) * 100.0
                    },
                }
            })
            .collect()
    }

    pub fn get_asset_type_percentages(&self) -> Vec<StackedNamedDataPoint> {
        let total_downloads = self.get_total_downloads();

        let mut tmp = self
            .raw_data
            .iter()
            .flat_map(|release| release.assets.iter())
            .group_by(|asset| get_asset_release_file_type(&asset.name))
            .map(|(file_type, assets)| {
                let downloads: u64 = assets
                    .iter()
                    .map(|asset| *asset.downloads.values().max().unwrap_or(&0) as u64)
                    .sum();

                StackedNamedDataPoint {
                    name: "Downloads".into(),
                    layer: file_type.unwrap_or("Unknown".into()),
                    value: if total_downloads == 0 {
                        0.0
                    } else {
                        (downloads as f64 / total_downloads as f64) * 100.0
                    },
                }
            })
            .collect::<Vec<_>>();

        tmp.sort_by(|a, b| a.value.total_cmp(&b.value));

        let total_asset_count = self.get_total_asset_count();

        tmp.extend(
            self.raw_data
                .iter()
                .flat_map(|release| release.assets.iter())
                .group_by(|asset| get_asset_release_file_type(&asset.name))
                .map(|(file_type, assets)| StackedNamedDataPoint {
                    name: "Assets".into(),
                    layer: file_type.unwrap_or("Unknown".into()),
                    value: if total_asset_count == 0 {
                        0.0
                    } else {
                        (assets.len() as f64 / total_asset_count as f64) * 100.0
                    },
                }),
        );

        let mut total_avg_size = 0.0;

        tmp.extend(
            self.raw_data
                .iter()
                .flat_map(|release| release.assets.iter())
                .group_by(|asset| get_asset_release_file_type(&asset.name))
                .map(|(file_type, assets)| {
                    let avg_size: f64 = assets.iter().map(|asset| asset.size as f64).sum::<f64>()
                        / assets.len() as f64;

                    total_avg_size += avg_size;

                    StackedNamedDataPoint {
                        name: "Avg. Size".into(),
                        layer: file_type.unwrap_or("Unknown".into()),
                        value: avg_size,
                    }
                })
                .collect::<Vec<_>>()
                .into_iter()
                .map(|mut point| {
                    if total_avg_size == 0.0 {
                        point.value = 0.0;
                    } else {
                        point.value = (point.value / total_avg_size) * 100.0;
                    }
                    point
                }),
        );

        tmp
    }

    pub fn get_asset_instruction_set_percentages(&self) -> Vec<StackedNamedDataPoint> {
        let total_downloads = self.get_total_downloads();

        self.raw_data
            .iter()
            .flat_map(|release| release.assets.iter())
            .group_by(|asset| get_asset_cpu_instruction_set(&asset.name))
            .map(|(instruction_set, assets)| {
                let downloads: u64 = assets
                    .iter()
                    .map(|asset| *asset.downloads.values().max().unwrap_or(&0) as u64)
                    .sum();

                StackedNamedDataPoint {
                    name: "Downloads".into(),
                    layer: instruction_set.unwrap_or("Unknown").into(),
                    value: if total_downloads == 0 {
                        0.0
                    } else {
                        (downloads as f64 / total_downloads as f64) * 100.0
                    },
                }
            })
            .sort_by(|a, b| a.value.total_cmp(&b.value))
            .collect()
    }

    pub fn get_asset_size_by_version(&self) -> Vec<StackedNamedDataPoint> {
        self.raw_data
            .iter()
            .filter(|release| {
                // filter out releases that only contain the .asar.gz file
                !release
                    .assets
                    .iter()
                    .all(|asset| asset.name.ends_with(".asar.gz"))
            })
            .flat_map(|release| {
                release
                    .assets
                    .iter()
                    .group_by(|asset| get_asset_release_file_type(&asset.name))
                    .map(|(file_type, assets)| {
                        (
                            &release.version,
                            file_type.unwrap_or("Unknown".into()),
                            assets.iter().map(|asset| asset.size).max().unwrap_or(0) as f64,
                        )
                    })
            })
            .sort_by(|a, b| a.0.cmp(&b.0).then_with(|| b.2.total_cmp(&a.2)))
            .map(|(version, file_type, value)| StackedNamedDataPoint {
                name: version.to_fancy_string(),
                layer: file_type,
                value,
            })
            .collect()
    }

    pub fn get_changelog_overview(&self) -> Vec<ChangelogDataPoint> {
        self.changelog
            .iter()
            .filter(|release| release.platform == ObsidianPlatform::Desktop)
            .group_by(|release| release.version.get_minor())
            .sort_by(|a, b| a.0.cmp(&b.0))
            .map(|(minor_version, releases)| {
                let first_public_version = releases
                    .iter()
                    .filter(|release| !release.insider)
                    .map(|release| &release.version)
                    .min()
                    .map(|v| v.to_fancy_string())
                    .unwrap_or_default();
                let public_release_date = releases
                    .iter()
                    .filter(|release| !release.insider)
                    .map(|release| &release.date)
                    .min()
                    .map(|v| v.to_fancy_string())
                    .unwrap_or_default();
                let insider_release_date = releases
                    .iter()
                    .filter(|release| release.insider)
                    .map(|release| &release.date)
                    .min()
                    .map(|v| v.to_fancy_string())
                    .unwrap_or_default();
                let number_of_insider_patches =
                    releases.iter().filter(|release| release.insider).count();
                let number_of_patches = releases.len();

                ChangelogDataPoint {
                    minor_version: minor_version.to_fancy_string(),
                    first_public_version,
                    public_release_date,
                    insider_release_date,
                    number_of_insider_patches,
                    number_of_patches,
                }
            })
            .collect()
    }

    pub fn get_changelog_changes(&self) -> Vec<ChangelogChanges> {
        let header_regexp: Regex = Regex::new(r"<h(\d)>(.*?)<\/h\d>").unwrap();
        let li_regexp: Regex = Regex::new(r"<li>").unwrap();

        self.changelog
            .iter()
            .filter(|release| release.platform == ObsidianPlatform::Desktop)
            .dedup_by(|a, b| a.version == b.version)
            .filter_map(|release| {
                let mut changes = ChangelogChanges {
                    version: release.version.clone(),
                    version_string: release.version.to_fancy_string(),
                    changes: HashMap::new(),
                };

                let headings = header_regexp
                    .captures_iter(&release.info)
                    .collect::<Vec<_>>();
                if headings.is_empty() {
                    let list_matches = li_regexp.captures_iter(&release.info).count();

                    changes
                        .changes
                        .insert(ChangeLogChangeCategory::Uncategorized, list_matches);
                } else {
                    let headings = headings
                        .iter()
                        .map(|m| {
                            let (_, [level_str, heading]) = m.extract();

                            let level = level_str.parse::<u8>().unwrap_or(6); // h6 as default
                            let start = m.get(0).unwrap().start();
                            let end = m.get(0).unwrap().end();

                            Heading {
                                level,
                                category: heading.into(),
                                start,
                                end,
                            }
                        })
                        .collect::<Vec<_>>();

                    // heading levels are inverse (h1, h2, h3, ...), so the max heading has the lowest number
                    let max_heading = headings
                        .iter()
                        .min_by(|a, b| a.level.cmp(&b.level))
                        .map(|h| h.level)
                        .expect("Non empty");

                    let headings = headings
                        .into_iter()
                        .filter(|h| h.level == max_heading)
                        .collect::<Vec<_>>();

                    let start_chunk = &release.info[0..headings[0].start];
                    let list_matches = li_regexp.captures_iter(start_chunk).count();
                    changes
                        .changes
                        .entry(ChangeLogChangeCategory::Uncategorized)
                        .and_modify(|e| *e += list_matches)
                        .or_insert(list_matches);

                    for i in 0..headings.len() {
                        let heading = &headings[i];
                        let next_heading = headings.get(i + 1);

                        let end = next_heading.map_or(release.info.len(), |h| h.start);
                        let chunk = &release.info[heading.end..end];

                        let list_matches = li_regexp.captures_iter(chunk).count();
                        changes
                            .changes
                            .entry(heading.category.clone())
                            .and_modify(|e| *e += list_matches)
                            .or_insert(list_matches);
                    }
                }

                Some(changes)
            })
            .sort_by(|a, b| a.version.cmp(&b.version))
            .collect()
    }

    pub fn get_changelog_changes_as_data_points(&self) -> Vec<StackedNamedDataPoint> {
        self.get_changelog_changes()
            .into_iter()
            .flat_map(|changes| {
                ChangeLogChangeCategory::iter().map(move |category| {
                    (
                        changes.version.clone(),
                        category.to_fancy_string(),
                        changes.changes.get(&category).cloned().unwrap_or(0),
                    )
                })
            })
            .sort_by(|a, b| a.0.cmp(&b.0))
            .map(|(version, category, value)| StackedNamedDataPoint {
                name: version.to_fancy_string(),
                layer: category,
                value: value as f64,
            })
            .collect()
    }

    pub fn get_changelog_changes_for_minor_releases(&self) -> Vec<ChangelogChanges> {
        self.get_changelog_changes()
            .into_iter()
            .group_by(|changes| changes.version.get_minor())
            .sort_by(|a, b| a.0.cmp(&b.0))
            .map(|(minor_version, changes)| {
                let mut combined_changes = ChangelogChanges {
                    version: minor_version.clone(),
                    version_string: minor_version.to_fancy_string(),
                    changes: HashMap::new(),
                };

                for change in changes {
                    for (category, count) in change.changes {
                        combined_changes
                            .changes
                            .entry(category)
                            .and_modify(|e| *e += count)
                            .or_insert(count);
                    }
                }

                combined_changes
            })
            .collect()
    }

    pub fn get_changelog_categories(&self) -> Vec<ChangeLogChangeCategory> {
        ChangeLogChangeCategory::iter().collect()
    }
}

struct Heading {
    level: u8,
    category: ChangeLogChangeCategory,
    start: usize,
    end: usize,
}
