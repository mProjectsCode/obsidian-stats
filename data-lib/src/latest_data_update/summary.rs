use std::collections::{HashMap, HashSet};

use crate::{
    latest_data_update::{
        CloneSummary, CountShare, LargestMainJs, LatestDataUpdateSummary, PluginPageCloneFreshness,
        PluginReleaseStateEntryInput, PluginSummary, ReleaseAcquisitionSummary,
        ReleaseStatsStateInput, ReleaseSummary, RepoAnalysisSummary, ThemeSummary,
    },
    plugin::{PluginData, PluginExtraData, PluginRepoAnalysisError},
    release::{GithubReleaseInfo, ObsidianReleaseInfo},
    theme::ThemeData,
};

pub struct BuildLatestDataUpdateSummaryInputs<'a> {
    pub plugins: &'a [PluginData],
    pub themes: &'a [ThemeData],
    pub repo_analysis_entries: &'a [PluginExtraData],
    pub changelog_releases: &'a [ObsidianReleaseInfo],
    pub github_releases: &'a [GithubReleaseInfo],
    pub interpolated_releases: &'a [GithubReleaseInfo],
    pub clone_entries: &'a HashMap<String, PluginPageCloneFreshness>,
    pub release_entries: &'a [PluginReleaseStateEntryInput],
    pub release_stats_state: &'a ReleaseStatsStateInput,
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

    let active_plugin_ids = plugin_ids_by_removed_state(plugins, false);
    let removed_plugin_ids = plugin_ids_by_removed_state(plugins, true);
    let latest_obsidian_release = github_releases.iter().max_by(|left, right| {
        left.date
            .to_fancy_string()
            .cmp(&right.date.to_fancy_string())
    });

    let clone_summary = build_clone_summary(clone_entries);
    let release_acquisition_summary = build_release_acquisition_summary(release_entries);
    let repo_analysis_summary = build_repo_analysis_summary(
        repo_analysis_entries,
        &active_plugin_ids,
        &removed_plugin_ids,
    );
    let release_run_at_unix = release_entries
        .iter()
        .map(|entry| entry.last_checked_unix)
        .max();
    let clone_run_at_unix = clone_entries
        .values()
        .map(|entry| entry.last_attempt_unix)
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
        latest_plugin_download_snapshot_date: latest_plugin_download_snapshot_date(plugins),
        latest_obsidian_release_date: latest_obsidian_release
            .map(|release| release.date.to_fancy_string()),
        latest_obsidian_version: latest_obsidian_release
            .map(|release| release.version.to_fancy_string()),
        plugins: build_plugin_summary(plugins, &active_plugin_ids, &removed_plugin_ids),
        themes: build_theme_summary(themes),
        releases: build_release_summary(changelog_releases, github_releases, interpolated_releases),
        clone: clone_summary,
        release_acquisition: release_acquisition_summary,
        repo_analysis: repo_analysis_summary,
    }
}

fn plugin_ids_by_removed_state(plugins: &[PluginData], removed: bool) -> HashSet<String> {
    plugins
        .iter()
        .filter(|plugin| plugin.removed_commit.is_some() == removed)
        .map(|plugin| plugin.id.clone())
        .collect()
}

fn latest_plugin_download_snapshot_date(plugins: &[PluginData]) -> Option<String> {
    plugins
        .iter()
        .flat_map(|plugin| plugin.download_history.0.keys().cloned())
        .max()
}

fn build_plugin_summary(
    plugins: &[PluginData],
    active_plugin_ids: &HashSet<String>,
    removed_plugin_ids: &HashSet<String>,
) -> PluginSummary {
    PluginSummary {
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
    }
}

fn build_theme_summary(themes: &[ThemeData]) -> ThemeSummary {
    ThemeSummary {
        total: themes.len(),
        active: themes
            .iter()
            .filter(|theme| theme.removed_commit.is_none())
            .count(),
        removed: themes
            .iter()
            .filter(|theme| theme.removed_commit.is_some())
            .count(),
    }
}

fn build_release_summary(
    changelog_releases: &[ObsidianReleaseInfo],
    github_releases: &[GithubReleaseInfo],
    interpolated_releases: &[GithubReleaseInfo],
) -> ReleaseSummary {
    ReleaseSummary {
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
    }
}

fn build_clone_summary(clone_entries: &HashMap<String, PluginPageCloneFreshness>) -> CloneSummary {
    let clone_failed_entries = clone_entries
        .iter()
        .filter(|(_, entry)| entry.status.starts_with("failed:"))
        .collect::<Vec<_>>();
    let clone_ok = clone_entries
        .values()
        .filter(|entry| entry.status == "ok")
        .count();
    let clone_skipped = clone_entries
        .values()
        .filter(|entry| matches!(entry.status.as_str(), "skipped" | "skipped_removed"))
        .count();
    let clone_failed = clone_failed_entries.len();
    let clone_attempted = clone_ok + clone_failed;

    CloneSummary {
        tracked: clone_entries.len(),
        ok: clone_ok,
        skipped: clone_skipped,
        failed: clone_failed,
        success_rate: clamp_rate(clone_ok, clone_attempted),
        failed_plugins: clone_failed_entries
            .iter()
            .map(|(plugin_id, _)| (*plugin_id).clone())
            .take(5)
            .collect(),
        status_counts: status_counts(
            clone_entries
                .values()
                .map(|entry| clone_status_label(&entry.status).to_string()),
            clone_entries.len(),
        ),
    }
}

fn build_release_acquisition_summary(
    release_entries: &[PluginReleaseStateEntryInput],
) -> ReleaseAcquisitionSummary {
    let release_status_counts = release_status_counts(release_entries);
    let release_statuses = count_shares(&release_status_counts, release_entries.len());
    let main_js_entries = release_entries
        .iter()
        .filter(|entry| entry.latest_release_main_js_size_bytes.is_some())
        .collect::<Vec<_>>();
    let largest_main_js_entry = main_js_entries.iter().max_by(|left, right| {
        left.latest_release_main_js_size_bytes
            .unwrap_or(0)
            .cmp(&right.latest_release_main_js_size_bytes.unwrap_or(0))
    });
    let total_main_js_bytes = main_js_entries
        .iter()
        .map(|entry| entry.latest_release_main_js_size_bytes.unwrap_or(0))
        .sum::<u64>();

    ReleaseAcquisitionSummary {
        tracked: release_entries.len(),
        ok: *release_status_counts.get("ok").unwrap_or(&0),
        retained_previous_main_js: *release_status_counts
            .get("main_js_not_updated_since_success")
            .unwrap_or(&0),
        no_release: *release_status_counts.get("no_release").unwrap_or(&0)
            + *release_status_counts
                .get("no_release_for_version")
                .unwrap_or(&0),
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
        total_main_js_bytes,
        average_main_js_bytes: if main_js_entries.is_empty() {
            0
        } else {
            total_main_js_bytes / main_js_entries.len() as u64
        },
        largest_main_js: LargestMainJs {
            repo: largest_main_js_entry.map(|entry| entry.repo.clone()),
            size_bytes: largest_main_js_entry
                .and_then(|entry| entry.latest_release_main_js_size_bytes)
                .unwrap_or(0),
        },
        status_counts: release_statuses,
    }
}

fn release_status_counts(
    release_entries: &[PluginReleaseStateEntryInput],
) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for entry in release_entries {
        let status = entry
            .latest_release_fetch_status
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        *counts.entry(status).or_insert(0) += 1;
    }
    counts
}

fn build_repo_analysis_summary(
    repo_analysis_entries: &[PluginExtraData],
    active_plugin_ids: &HashSet<String>,
    removed_plugin_ids: &HashSet<String>,
) -> RepoAnalysisSummary {
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

    RepoAnalysisSummary {
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
        error_counts: repo_analysis_error_shares(repo_analysis_entries, active_plugin_ids),
    }
}

fn repo_analysis_error_shares(
    repo_analysis_entries: &[PluginExtraData],
    active_plugin_ids: &HashSet<String>,
) -> Vec<CountShare> {
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

    let mut shares = repo_analysis_error_counts
        .iter()
        .map(|(label, count)| CountShare {
            label: label.as_label().to_string(),
            count: *count,
            share: clamp_rate(*count, active_plugin_ids.len()),
        })
        .collect::<Vec<_>>();
    sort_count_shares(&mut shares);
    shares
}

fn status_counts(labels: impl Iterator<Item = String>, total: usize) -> Vec<CountShare> {
    let mut counts = HashMap::new();
    for label in labels {
        *counts.entry(label).or_insert(0) += 1;
    }
    count_shares(&counts, total)
}

fn count_shares(counts: &HashMap<String, usize>, total: usize) -> Vec<CountShare> {
    let mut shares = counts
        .iter()
        .map(|(label, count)| CountShare {
            label: label.clone(),
            count: *count,
            share: clamp_rate(*count, total),
        })
        .collect::<Vec<_>>();
    sort_count_shares(&mut shares);
    shares
}

fn sort_count_shares(shares: &mut [CountShare]) {
    shares.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then(left.label.cmp(&right.label))
    });
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

fn clone_status_label(status: &str) -> &str {
    if status.starts_with("failed:") {
        return "failed";
    }

    if status == "skipped" {
        return "skipped_removed";
    }

    status
}

#[cfg(test)]
mod tests {
    use super::{clone_status_label, is_release_acquisition_error};
    use crate::plugin::PluginRepoAnalysisError;

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
        assert_eq!(
            PluginRepoAnalysisError::from_raw("repository_scan_error"),
            PluginRepoAnalysisError::RepositoryScan
        );
        assert_eq!(
            PluginRepoAnalysisError::from_raw("main_js_analysis_too_large"),
            PluginRepoAnalysisError::MainJsAnalysisTooLarge
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

    #[test]
    fn normalizes_clone_status_labels() {
        assert_eq!(clone_status_label("failed:timeout"), "failed");
        assert_eq!(clone_status_label("skipped"), "skipped_removed");
        assert_eq!(
            clone_status_label("version_history_missing"),
            "version_history_missing"
        );
    }
}
