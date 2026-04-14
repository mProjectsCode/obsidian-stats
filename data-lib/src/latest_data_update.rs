use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    plugin::{PluginData, PluginExtraData, PluginRepoAnalysisError},
    release::{GithubReleaseInfo, ObsidianReleaseInfo},
    theme::ThemeData,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneStateEntryInput {
    pub plugin_id: String,
    pub status: String,
    pub last_attempt_unix: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginReleaseStateEntryInput {
    pub repo: String,
    pub last_checked_unix: i64,
    pub latest_release_main_js_size_bytes: Option<u64>,
    pub latest_release_fetch_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReleaseStatsStateInput {
    pub last_fetch_unix: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountShare {
    pub label: String,
    pub count: usize,
    pub share: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestDataUpdateSummary {
    pub refreshed_at_unix: Option<i64>,
    pub clone_run_at_unix: Option<i64>,
    pub release_run_at_unix: Option<i64>,
    pub obsidian_release_fetch_at_unix: Option<i64>,
    pub latest_plugin_download_snapshot_date: Option<String>,
    pub latest_obsidian_release_date: Option<String>,
    pub latest_obsidian_version: Option<String>,
    pub plugins: PluginSummary,
    pub themes: ThemeSummary,
    pub releases: ReleaseSummary,
    pub clone: CloneSummary,
    pub release_acquisition: ReleaseAcquisitionSummary,
    pub repo_analysis: RepoAnalysisSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSummary {
    pub total: usize,
    pub active: usize,
    pub removed: usize,
    pub total_downloads: u64,
    pub version_snapshots: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSummary {
    pub total: usize,
    pub active: usize,
    pub removed: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseSummary {
    pub changelog_entries: usize,
    pub github_release_snapshots: usize,
    pub interpolated_release_snapshots: usize,
    pub asset_count: usize,
    pub download_snapshots: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneSummary {
    pub tracked: usize,
    pub ok: usize,
    pub skipped: usize,
    pub failed: usize,
    pub success_rate: f64,
    pub failed_plugins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LargestMainJs {
    pub repo: Option<String>,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAcquisitionSummary {
    pub tracked: usize,
    pub ok: usize,
    pub retained_previous_main_js: usize,
    pub no_release: usize,
    pub no_main_js_asset: usize,
    pub error_count: usize,
    pub main_js_coverage: usize,
    pub main_js_coverage_rate: f64,
    pub total_main_js_bytes: u64,
    pub average_main_js_bytes: u64,
    pub largest_main_js: LargestMainJs,
    pub status_counts: Vec<CountShare>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoAnalysisSummary {
    pub tracked: usize,
    pub active_success: usize,
    pub active_failures: usize,
    pub removed_skipped: usize,
    pub coverage_rate: f64,
    pub failure_samples: Vec<String>,
    pub error_counts: Vec<CountShare>,
}

pub struct BuildLatestDataUpdateSummaryInputs<'a> {
    pub plugins: &'a [PluginData],
    pub themes: &'a [ThemeData],
    pub repo_analysis_entries: &'a [PluginExtraData],
    pub changelog_releases: &'a [ObsidianReleaseInfo],
    pub github_releases: &'a [GithubReleaseInfo],
    pub interpolated_releases: &'a [GithubReleaseInfo],
    pub clone_entries: &'a [CloneStateEntryInput],
    pub release_entries: &'a [PluginReleaseStateEntryInput],
    pub release_stats_state: &'a ReleaseStatsStateInput,
}

fn clamp_rate(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        return 0.0;
    }

    numerator as f64 / denominator as f64
}

fn is_release_acquisition_error(status: &str) -> bool {
    status.contains("error")
        || status.contains("failed")
        || status.contains("rate_limit")
        || status.contains("rate_limited")
}

pub fn build_latest_data_update_summary(
    inputs: BuildLatestDataUpdateSummaryInputs<'_>,
) -> LatestDataUpdateSummary {
    let BuildLatestDataUpdateSummaryInputs {
        plugins,
        themes,
        repo_analysis_entries,
        changelog_releases,
        github_releases,
        interpolated_releases,
        clone_entries,
        release_entries,
        release_stats_state,
    } = inputs;

    let active_plugin_ids = plugins
        .iter()
        .filter(|plugin| plugin.removed_commit.is_none())
        .map(|plugin| plugin.id.clone())
        .collect::<HashSet<_>>();
    let removed_plugin_ids = plugins
        .iter()
        .filter(|plugin| plugin.removed_commit.is_some())
        .map(|plugin| plugin.id.clone())
        .collect::<HashSet<_>>();

    let latest_plugin_download_snapshot_date = plugins
        .iter()
        .flat_map(|plugin| plugin.download_history.0.keys().cloned())
        .max();

    let plugin_totals = PluginSummary {
        total: plugins.len(),
        active: active_plugin_ids.len(),
        removed: removed_plugin_ids.len(),
        total_downloads: plugins
            .iter()
            .map(|plugin| plugin.download_count as u64)
            .sum(),
        version_snapshots: plugins
            .iter()
            .map(|plugin| plugin.version_history.len())
            .sum(),
    };

    let theme_totals = ThemeSummary {
        total: themes.len(),
        active: themes
            .iter()
            .filter(|theme| theme.removed_commit.is_none())
            .count(),
        removed: themes
            .iter()
            .filter(|theme| theme.removed_commit.is_some())
            .count(),
    };

    let release_totals = ReleaseSummary {
        changelog_entries: changelog_releases.len(),
        github_release_snapshots: github_releases.len(),
        interpolated_release_snapshots: interpolated_releases.len(),
        asset_count: github_releases
            .iter()
            .map(|release| release.assets.len())
            .sum(),
        download_snapshots: github_releases
            .iter()
            .map(|release| {
                release
                    .assets
                    .iter()
                    .map(|asset| asset.downloads.len())
                    .sum::<usize>()
            })
            .sum(),
    };

    let latest_obsidian_release = github_releases.iter().max_by(|left, right| {
        left.date
            .to_fancy_string()
            .cmp(&right.date.to_fancy_string())
    });

    let clone_failed_entries = clone_entries
        .iter()
        .filter(|entry| entry.status.starts_with("failed:"))
        .collect::<Vec<_>>();
    let clone_ok = clone_entries
        .iter()
        .filter(|entry| entry.status == "ok")
        .count();
    let clone_skipped = clone_entries
        .iter()
        .filter(|entry| entry.status == "skipped")
        .count();
    let clone_failed = clone_failed_entries.len();
    let clone_attempted = clone_ok + clone_failed;
    let clone_run_at_unix = clone_entries
        .iter()
        .map(|entry| entry.last_attempt_unix)
        .max();

    let mut release_status_counts: HashMap<String, usize> = HashMap::new();
    for entry in release_entries {
        let status = entry
            .latest_release_fetch_status
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        *release_status_counts.entry(status).or_insert(0) += 1;
    }

    let mut release_statuses = release_status_counts
        .iter()
        .map(|(label, count)| CountShare {
            label: label.clone(),
            count: *count,
            share: clamp_rate(*count, release_entries.len()),
        })
        .collect::<Vec<_>>();
    release_statuses.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then(left.label.cmp(&right.label))
    });

    let main_js_entries = release_entries
        .iter()
        .filter(|entry| entry.latest_release_main_js_size_bytes.is_some())
        .collect::<Vec<_>>();

    let largest_main_js_entry = main_js_entries.iter().max_by(|left, right| {
        left.latest_release_main_js_size_bytes
            .unwrap_or(0)
            .cmp(&right.latest_release_main_js_size_bytes.unwrap_or(0))
    });

    let repo_ok_entries = repo_analysis_entries
        .iter()
        .filter_map(|entry| {
            if let Ok(data) = &entry.repo {
                Some((entry.id.clone(), data.clone()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let active_repo_success_entries = repo_ok_entries
        .iter()
        .filter(|(id, _)| active_plugin_ids.contains(id))
        .collect::<Vec<_>>();
    let active_repo_failure_entries = repo_analysis_entries
        .iter()
        .filter(|entry| active_plugin_ids.contains(&entry.id) && entry.repo.is_err())
        .collect::<Vec<_>>();
    let removed_repo_skipped_entries = repo_analysis_entries
        .iter()
        .filter(|entry| removed_plugin_ids.contains(&entry.id) && entry.repo.is_err())
        .collect::<Vec<_>>();

    let mut repo_analysis_error_counts: HashMap<PluginRepoAnalysisError, usize> = HashMap::new();
    for entry in repo_analysis_entries {
        if !active_plugin_ids.contains(&entry.id) {
            continue;
        }

        match &entry.repo {
            Ok(repo_data) => {
                for error in &repo_data.analysis_errors {
                    *repo_analysis_error_counts.entry(*error).or_insert(0) += 1;
                }
            }
            Err(error) => {
                let code = PluginRepoAnalysisError::from_raw(error);
                *repo_analysis_error_counts.entry(code).or_insert(0) += 1;
            }
        }
    }

    let mut repo_analysis_error_shares = repo_analysis_error_counts
        .iter()
        .map(|(label, count)| CountShare {
            label: label.as_label().to_string(),
            count: *count,
            share: clamp_rate(*count, active_plugin_ids.len()),
        })
        .collect::<Vec<_>>();
    repo_analysis_error_shares.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then(left.label.cmp(&right.label))
    });

    let release_run_at_unix = release_entries
        .iter()
        .map(|entry| entry.last_checked_unix)
        .max();
    let refreshed_at_unix = [
        clone_run_at_unix,
        release_run_at_unix,
        release_stats_state.last_fetch_unix,
    ]
    .into_iter()
    .flatten()
    .max();

    LatestDataUpdateSummary {
        refreshed_at_unix,
        clone_run_at_unix,
        release_run_at_unix,
        obsidian_release_fetch_at_unix: release_stats_state.last_fetch_unix,
        latest_plugin_download_snapshot_date,
        latest_obsidian_release_date: latest_obsidian_release
            .map(|release| release.date.to_fancy_string()),
        latest_obsidian_version: latest_obsidian_release
            .map(|release| release.version.to_fancy_string()),
        plugins: plugin_totals,
        themes: theme_totals,
        releases: release_totals,
        clone: CloneSummary {
            tracked: clone_entries.len(),
            ok: clone_ok,
            skipped: clone_skipped,
            failed: clone_failed,
            success_rate: clamp_rate(clone_ok, clone_attempted),
            failed_plugins: clone_failed_entries
                .iter()
                .map(|entry| entry.plugin_id.clone())
                .take(5)
                .collect(),
        },
        release_acquisition: ReleaseAcquisitionSummary {
            tracked: release_entries.len(),
            ok: *release_status_counts.get("ok").unwrap_or(&0),
            retained_previous_main_js: *release_status_counts
                .get("main_js_not_updated_since_success")
                .unwrap_or(&0),
            no_release: *release_status_counts.get("no_release").unwrap_or(&0),
            no_main_js_asset: *release_status_counts.get("no_main_js_asset").unwrap_or(&0),
            error_count: release_entries
                .iter()
                .filter(|entry| {
                    is_release_acquisition_error(
                        entry.latest_release_fetch_status.as_deref().unwrap_or(""),
                    )
                })
                .count(),
            main_js_coverage: main_js_entries.len(),
            main_js_coverage_rate: clamp_rate(main_js_entries.len(), release_entries.len()),
            total_main_js_bytes: main_js_entries
                .iter()
                .map(|entry| entry.latest_release_main_js_size_bytes.unwrap_or(0))
                .sum(),
            average_main_js_bytes: if main_js_entries.is_empty() {
                0
            } else {
                main_js_entries
                    .iter()
                    .map(|entry| entry.latest_release_main_js_size_bytes.unwrap_or(0))
                    .sum::<u64>()
                    / main_js_entries.len() as u64
            },
            largest_main_js: LargestMainJs {
                repo: largest_main_js_entry.map(|entry| entry.repo.clone()),
                size_bytes: largest_main_js_entry
                    .and_then(|entry| entry.latest_release_main_js_size_bytes)
                    .unwrap_or(0),
            },
            status_counts: release_statuses,
        },
        repo_analysis: RepoAnalysisSummary {
            tracked: repo_analysis_entries.len(),
            active_success: active_repo_success_entries.len(),
            active_failures: active_repo_failure_entries.len(),
            removed_skipped: removed_repo_skipped_entries.len(),
            coverage_rate: clamp_rate(active_repo_success_entries.len(), active_plugin_ids.len()),
            failure_samples: active_repo_failure_entries
                .iter()
                .map(|entry| entry.id.clone())
                .take(5)
                .collect(),
            error_counts: repo_analysis_error_shares,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{PluginRepoAnalysisError, is_release_acquisition_error};

    #[test]
    fn classifies_repository_missing_code() {
        assert_eq!(
            PluginRepoAnalysisError::from_raw("repository_missing"),
            PluginRepoAnalysisError::RepositoryMissing
        );
        assert_eq!(
            PluginRepoAnalysisError::from_raw(
                "repository_missing: plugin abc: ./out/plugin-repos/abc"
            ),
            PluginRepoAnalysisError::RepositoryMissing
        );
    }

    #[test]
    fn classifies_known_repo_analysis_codes_without_prefix_payload() {
        assert_eq!(
            PluginRepoAnalysisError::from_raw("manifest_read_error"),
            PluginRepoAnalysisError::ManifestRead
        );
        assert_eq!(
            PluginRepoAnalysisError::from_raw("package_json_parse_error"),
            PluginRepoAnalysisError::PackageJsonParse
        );
    }

    #[test]
    fn identifies_release_error_statuses() {
        assert!(is_release_acquisition_error("request_error:timeout"));
        assert!(is_release_acquisition_error(
            "main_js_download_failed:write:disk_full"
        ));
        assert!(is_release_acquisition_error("rate_limited"));
        assert!(!is_release_acquisition_error("ok"));
    }
}
