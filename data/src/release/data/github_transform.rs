use data_lib::{
    date::Date,
    release::{GithubAssetInfo, GithubReleaseInfo},
    version::Version,
};
use hashbrown::HashMap;

use crate::release::GithubReleaseEntry;

pub(super) fn get_github_release_info(
    release_entries: &mut Vec<GithubReleaseInfo>,
    new_entries: Vec<GithubReleaseEntry>,
) {
    let today = Date::now();

    new_entries.into_iter().for_each(|entry| {
        let Some(version) = Version::parse(&entry.tag_name) else {
            eprintln!(
                "Warning: skipping GitHub release entry with invalid version tag: {}",
                entry.tag_name
            );
            return;
        };

        let existing_entry = release_entries.iter_mut().find(|e| e.version == version);

        if let Some(existing_entry) = existing_entry {
            entry.assets.iter().for_each(|asset| {
                let existing_asset = existing_entry
                    .assets
                    .iter_mut()
                    .find(|a| a.name == asset.name);
                if let Some(existing_asset) = existing_asset {
                    existing_asset
                        .downloads
                        .insert(today.to_fancy_string(), asset.download_count);
                } else {
                    let mut download_map = HashMap::new();
                    download_map.insert(today.to_fancy_string(), asset.download_count);

                    existing_entry.assets.push(GithubAssetInfo {
                        name: asset.name.clone(),
                        downloads: download_map,
                        size: asset.size,
                    });
                }
            });
        } else {
            let Some(date) = Date::from_string(&entry.published_at.date_naive().to_string()) else {
                eprintln!(
                    "Warning: skipping GitHub release entry with invalid publish date: {}",
                    entry.tag_name
                );
                return;
            };

            release_entries.push(GithubReleaseInfo {
                version: version.clone(),
                date,
                time: entry.published_at.time().to_string(),
                assets: entry
                    .assets
                    .into_iter()
                    .map(|asset| {
                        let mut download_map = HashMap::new();
                        download_map.insert(today.to_fancy_string(), asset.download_count);

                        GithubAssetInfo {
                            name: asset.name,
                            downloads: download_map,
                            size: asset.size,
                        }
                    })
                    .collect(),
            });
        }
    });
}
