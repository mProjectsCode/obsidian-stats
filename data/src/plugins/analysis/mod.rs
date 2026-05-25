use std::{
    path::Path,
    sync::atomic::{AtomicUsize, Ordering},
};

use data_lib::plugin::PluginExtraData;
use hashbrown::{HashMap, HashSet};
use rayon::{
    ThreadPoolBuilder,
    iter::{IntoParallelRefIterator, ParallelIterator},
};

use self::{
    pipeline::analyze_plugin,
    repo_analysis::{read_plugin_version_deprecations, read_removed_plugins},
    run_stats::{ExtraPluginResult, ExtraRunStats},
};

use crate::{
    constants::{PLUGIN_RELEASE_ENRICHMENT_STATE_PATH, PLUGIN_REPO_DATA_PATH},
    file_utils::{read_chunked_data_or_default, write_in_chunks_atomic},
    plugins::{
        data::read_plugin_data, license::license_compare::LicenseComparer,
        release_acquisition::PluginReleaseState,
    },
    progress::should_log_progress,
    state::read_json_or_default,
};

mod mainjs;
mod pipeline;
mod repo;
mod repo_analysis;
mod run_stats;
mod types;

const EXTRA_ANALYSIS_THREADS_ENV: &str = "EXTRA_ANALYSIS_THREADS";

pub fn extract_analysis_data() -> Result<(), Box<dyn std::error::Error>> {
    let removed_plugins = read_removed_plugins()?;
    let removed_reason_by_id = removed_plugins
        .into_iter()
        .map(|entry| (entry.id, entry.reason))
        .collect::<HashMap<_, _>>();

    let deprecated_versions_by_plugin = read_plugin_version_deprecations()?;

    let plugin_data = read_plugin_data()?;
    let existing_extra_data: Vec<PluginExtraData> =
        read_chunked_data_or_default(Path::new(PLUGIN_REPO_DATA_PATH));

    let mut extra_data_by_id = existing_extra_data
        .into_iter()
        .map(|entry| (entry.id.clone(), entry))
        .collect::<HashMap<_, _>>();

    let active_plugin_ids = plugin_data
        .iter()
        .map(|plugin| plugin.id.clone())
        .collect::<HashSet<_>>();
    extra_data_by_id.retain(|id, _| active_plugin_ids.contains(id));

    let release_state: PluginReleaseState =
        read_json_or_default(Path::new(PLUGIN_RELEASE_ENRICHMENT_STATE_PATH));

    let mut license_comparer = LicenseComparer::new();
    license_comparer.init();

    let default_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let thread_count = configured_thread_count(EXTRA_ANALYSIS_THREADS_ENV, default_threads);

    println!(
        "Extra data: processing {} plugins (analysis phase, threads: {})",
        plugin_data.len(),
        thread_count
    );

    let processed = AtomicUsize::new(0);
    let thread_pool = ThreadPoolBuilder::new()
        .num_threads(thread_count)
        .build()
        .expect("Failed to build extra analysis thread pool");

    let plugin_results = thread_pool.install(|| {
        plugin_data
            .par_iter()
            .map(|plugin| {
                let removal_reason = removed_reason_by_id.get(&plugin.id).cloned();
                let deprecated_versions = deprecated_versions_by_plugin
                    .0
                    .get(&plugin.id)
                    .map_or_else(Vec::new, |versions| versions.clone());

                let mut stats = ExtraRunStats::default();

                let repo = if plugin.removed_commit.is_none() {
                    match analyze_plugin(plugin, &license_comparer, &release_state, &mut stats) {
                        Ok(repo_data) => Ok(repo_data),
                        Err(err) => {
                            stats.repo_extract_failed += 1;

                            println!("Failed to analyze plugin {}: {}", plugin.id, err);

                            Err(err)
                        }
                    }
                } else {
                    stats.removed_skipped += 1;
                    Err(format!(
                        "Plugin {} was removed, skipping repository extraction",
                        plugin.id
                    ))
                };

                let done = processed.fetch_add(1, Ordering::Relaxed) + 1;
                if should_log_progress(done, plugin_data.len()) {
                    println!("  Processed {done} / {}", plugin_data.len());
                }

                ExtraPluginResult {
                    data: PluginExtraData {
                        id: plugin.id.clone(),
                        repo,
                        removal_reason,
                        deprecated_versions,
                    },
                    stats,
                }
            })
            .collect::<Vec<_>>()
    });

    let mut run_stats = ExtraRunStats::default();
    for result in plugin_results {
        run_stats.merge(result.stats);
        extra_data_by_id.insert(result.data.id.clone(), result.data);
    }

    checkpoint_extra_data(&extra_data_by_id)?;

    println!("Extra data summary:");
    println!("  Removed plugins skipped: {}", run_stats.removed_skipped);
    println!(
        "  Repo extraction failures: {}",
        run_stats.repo_extract_failed
    );
    println!(
        "  Missing release acquisition state: {}",
        run_stats.release_state_missing
    );
    println!(
        "  Release main.js scans (success): {}",
        run_stats.release_main_js_scanned
    );
    println!(
        "  Release main.js scans (failed/skip): {}",
        run_stats.release_main_js_scan_failed
    );

    Ok(())
}

fn configured_thread_count(env_var: &str, default_threads: usize) -> usize {
    std::env::var(env_var)
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|count| *count > 0)
        .unwrap_or(default_threads)
}

fn checkpoint_extra_data(
    extra_data_by_id: &HashMap<String, PluginExtraData>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut extra_data = extra_data_by_id.values().cloned().collect::<Vec<_>>();
    extra_data.sort_by(|a, b| a.id.cmp(&b.id));

    write_in_chunks_atomic(Path::new(PLUGIN_REPO_DATA_PATH), &extra_data, 50)?;

    Ok(())
}
