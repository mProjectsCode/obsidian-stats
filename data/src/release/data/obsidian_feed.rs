use data_lib::{
    date::Date,
    input_data::ObsReleasesFeedInner,
    release::{ObsidianPlatform, ObsidianReleaseInfo},
    version::Version,
};

use crate::{constants::RELEASE_INFO_URL, security::http_client};

pub(super) fn get_obs_release_info() -> Result<Vec<ObsidianReleaseInfo>, Box<dyn std::error::Error>>
{
    let response = http_client()?
        .get(RELEASE_INFO_URL)
        .send()?
        .error_for_status()?;
    let text = response.text()?;

    let feed_data: ObsReleasesFeedInner = quick_xml::de::from_str(&text)?;

    Ok(feed_data
        .entries
        .into_iter()
        .filter_map(|entry| {
            if entry.id.contains("publish") {
                return None; // Skip publish entries, because they are weird and we don't use them
            }

            let id_parts = entry.id.split('-').collect::<Vec<&str>>();
            let version_str = id_parts.last()?.trim_matches('/');
            let version = Version::parse(version_str)?;

            let insider = entry.title.contains("Early access");

            // Check that the title contains a version of the form "X.Y" or "X.Y.Z"
            // This assumes that the title contains no other dots
            let major_release = entry.title.split('.').count() == 2;

            let platform = if entry.id.contains("desktop") {
                ObsidianPlatform::Desktop
            } else if entry.id.contains("mobile") {
                ObsidianPlatform::Mobile
            } else if entry.id.contains("publish") {
                ObsidianPlatform::Publish
            } else {
                ObsidianPlatform::Desktop // Default to Desktop if not specified
            };

            let date = Date::from_string(entry.updated.split('T').next()?)?;

            Some(ObsidianReleaseInfo {
                version,
                platform,
                insider,
                date,
                info: entry.content,
                major_release,
            })
        })
        .collect())
}
