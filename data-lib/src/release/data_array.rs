use wasm_bindgen::prelude::*;

use crate::{
    common::{GroupByExt, StackedNamedDataPoint},
    release::{
        GithubReleaseInfo, OS, ObsidianReleaseInfo, ToFancyString, get_asset_cpu_instruction_set,
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
        let mut tmp = self
            .raw_data
            .iter()
            .flat_map(|release| {
                release
                    .assets
                    .iter()
                    .group_by(|asset| OS::from_asset_name(&asset.name))
                    .into_iter()
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
            .collect::<Vec<_>>();

        tmp.sort_by(|a, b| a.0.cmp(&b.0));
        tmp.into_iter()
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
            .into_iter()
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
                .into_iter()
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
                .into_iter()
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

        let mut tmp = self
            .raw_data
            .iter()
            .flat_map(|release| release.assets.iter())
            .group_by(|asset| get_asset_cpu_instruction_set(&asset.name))
            .into_iter()
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
            .collect::<Vec<_>>();

        tmp.sort_by(|a, b| a.value.total_cmp(&b.value));

        tmp
    }

    pub fn get_asset_size_by_version(&self) -> Vec<StackedNamedDataPoint> {
        let mut tmp = self
            .raw_data
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
                    .into_iter()
                    .map(|(file_type, assets)| {
                        (
                            &release.version,
                            file_type.unwrap_or("Unknown".into()),
                            assets.iter().map(|asset| asset.size).max().unwrap_or(0) as f64,
                        )
                    })
            })
            .collect::<Vec<_>>();

        tmp.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| b.2.total_cmp(&a.2)));

        tmp.into_iter()
            .map(|(version, file_type, value)| StackedNamedDataPoint {
                name: version.to_fancy_string(),
                layer: file_type,
                value,
            })
            .collect()
    }
}
