use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    commit::StringCommit,
    common::{
        DownloadDataPoint, EntryChangeDataPoint, LOC_EXCLUDED, NamedDataPoint, VersionDataPoint,
    },
    date::Date,
    plugin::{
        FundingUrl, LicenseInfo, PluginData, PluginExtraData, PluginRepoData,
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
        format!("https://github.com/{}", self.data.current_entry.repo)
    }

    pub fn funding_url(&self) -> Option<FundingUrl> {
        self.repo_data()
            .and_then(|r| r.manifest.funding_url.clone())
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
        self.repo_data().and_then(|r| r.manifest.is_desktop_only)
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

    pub fn license_package_json(&self) -> String {
        LicenseInfo::from(self.repo_data().map(|r| &r.package_json_license)).to_fancy_string()
    }

    pub fn license_file(&self) -> String {
        LicenseInfo::from(self.repo_data().map(|r| &r.file_license)).to_fancy_string()
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

    pub fn package_managers_str(&self) -> Option<String> {
        self.repo_data().and_then(|r| {
            let mut strs = r
                .package_managers
                .iter()
                .map(|pm| pm.get_identifier())
                .collect::<Vec<_>>();
            if strs.is_empty() {
                return None;
            }
            strs.sort();
            Some(strs.join(", "))
        })
    }

    pub fn bundlers_str(&self) -> Option<String> {
        self.repo_data().and_then(|r| {
            let mut strs = r
                .bundlers
                .iter()
                .map(|pm| pm.get_identifier())
                .collect::<Vec<_>>();
            if strs.is_empty() {
                return None;
            }
            strs.sort();
            Some(strs.join(", "))
        })
    }

    pub fn testing_frameworks_str(&self) -> Option<String> {
        self.repo_data().and_then(|r| {
            let mut strs = r
                .testing_frameworks
                .iter()
                .map(|pm| pm.get_identifier())
                .collect::<Vec<_>>();
            if strs.is_empty() {
                return None;
            }
            strs.sort();
            Some(strs.join(", "))
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

    pub fn dev_dependencies_str(&self) -> Option<String> {
        self.repo_data().and_then(|r| {
            if r.dev_dependencies.is_empty() {
                return None;
            }
            Some(r.dev_dependencies.join(", "))
        })
    }

    pub fn dependencies(&self) -> Option<Vec<String>> {
        self.repo_data().map(|r| r.dependencies.clone())
    }

    pub fn dependencies_str(&self) -> Option<String> {
        self.repo_data().and_then(|r| {
            if r.dependencies.is_empty() {
                return None;
            }
            Some(r.dependencies.join(", "))
        })
    }

    pub fn has_dependency(&self, dependency: &str) -> Option<bool> {
        self.repo_data().map(|r| {
            r.dependencies.iter().any(|d| d == dependency)
                || r.dev_dependencies.iter().any(|d| d == dependency)
        })
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
            .map_or_else(Date::now, |c| c.date.clone());

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
        get_plugin_warnings(self)
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

    pub fn loc(&self) -> Option<Vec<NamedDataPoint>> {
        self.repo_data().map(|r| {
            r.lines_of_code
                .iter()
                .filter(|(lang, count)| **count > 0 && !LOC_EXCLUDED.contains(&lang.as_str()))
                .map(|(lang, count)| NamedDataPoint {
                    name: lang.clone(),
                    value: *count as f64,
                })
                .collect()
        })
    }
}
