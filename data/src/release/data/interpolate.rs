use data_lib::{
    date::Date,
    release::{GithubAssetInfo, GithubReleaseInfo},
};
use hashbrown::HashMap;

pub(super) fn interpolate_github_release_info(
    full_data: &[GithubReleaseInfo],
) -> Vec<GithubReleaseInfo> {
    let today = Date::now();

    full_data
        .iter()
        .map(|info| {
            let assets = info
                .assets
                .iter()
                .map(|asset| {
                    if asset.downloads.is_empty() {
                        return asset.clone();
                    }

                    let mut had_invalid_dates = false;
                    let mut updates = asset
                        .downloads
                        .keys()
                        .filter_map(|x| match Date::from_string(x) {
                            Some(date) => Some(date),
                            None => {
                                had_invalid_dates = true;
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    updates.sort();

                    let Some(mut first_update) = updates.first().cloned() else {
                        if had_invalid_dates {
                            eprintln!(
                                "Warning: skipping interpolation for asset {} due to invalid date keys",
                                asset.name
                            );
                        }
                        return asset.clone();
                    };

                    first_update.reverse_days(7);
                    first_update.advance_to_weekday(0);

                    let downloads = first_update
                        .iterate_weekly_to(&today)
                        .filter_map(|date| {
                            let (pre, post) = date.find_surrounding(&updates)?;
                            let pre_diff = date.diff_in_days(pre).abs();
                            let post_diff = date.diff_in_days(post).abs();
                            let ratio = if pre_diff + post_diff == 0 {
                                0.0
                            } else {
                                pre_diff as f64 / (pre_diff + post_diff) as f64
                            };

                            let pre_downloads = asset.downloads.get(&pre.to_fancy_string())?;
                            let post_downloads = asset.downloads.get(&post.to_fancy_string())?;

                            let downloads = interpolate(*pre_downloads, *post_downloads, ratio);

                            Some((date.to_fancy_string(), downloads))
                        })
                        .collect::<HashMap<String, u32>>();

                    GithubAssetInfo {
                        name: asset.name.clone(),
                        downloads,
                        size: asset.size,
                    }
                })
                .collect();

            GithubReleaseInfo {
                version: info.version.clone(),
                date: info.date.clone(),
                time: info.time.clone(),
                assets,
            }
        })
        .collect()
}

fn interpolate(a: u32, b: u32, ratio: f64) -> u32 {
    if ratio >= 1.0 {
        b
    } else if ratio <= 0.0 {
        a
    } else {
        (a as f64 * (1.0 - ratio) + b as f64 * ratio) as u32
    }
}
