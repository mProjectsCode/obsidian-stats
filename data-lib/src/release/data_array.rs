use wasm_bindgen::prelude::*;

use crate::{
    common::{GroupByExt, SackedNamedDataPoint},
    release::{GithubReleaseInfo, OS, ObsidianReleaseInfo, ToFancyString},
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
}

#[wasm_bindgen]
impl ReleaseDataArray {
    pub fn total_downloads(&self) -> Vec<SackedNamedDataPoint> {
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
                            release.version.clone(),
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
            .map(|(version, os, value)| SackedNamedDataPoint {
                name: version.to_fancy_string(),
                layer: os,
                value,
            })
            .collect()
    }

    pub fn total_download_percentages(&self) -> Vec<SackedNamedDataPoint> {
        let total_downloads = self.total_downloads();

        let downloads_per_version = self
            .raw_data
            .iter()
            .map(|release| {
                let downloads = release
                    .assets
                    .iter()
                    .filter(|asset| OS::from_asset_name(&asset.name).is_some())
                    .map(|asset| *asset.downloads.values().max().unwrap_or(&0) as f64)
                    .sum::<f64>();

                (release.version.to_fancy_string(), downloads)
            })
            .collect::<Vec<_>>();

        total_downloads
            .into_iter()
            .map(|point| {
                let total = downloads_per_version
                    .iter()
                    .find(|(version, _)| version == &point.name)
                    .map_or(0.0, |(_, downloads)| *downloads);
                SackedNamedDataPoint {
                    name: point.name,
                    layer: point.layer,
                    value: if total == 0.0 {
                        0.0
                    } else {
                        (point.value / total) * 100.0
                    },
                }
            })
            .collect()
    }
}
