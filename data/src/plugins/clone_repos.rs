use std::{num::NonZero, path::Path};

use data_lib::plugin::PluginData;
use gix::prepare_clone;
use indicatif::{MultiProgress, ProgressBar};
use rayon::{
    ThreadPoolBuilder,
    iter::{IntoParallelIterator, ParallelIterator},
};

use crate::{constants::PLUGIN_REPO_PATH, file_utils::empty_dir, plugins::data::read_plugin_data};

enum CloneResult {
    Success,
    Skipped(String),
    Failed(String, String),
}

pub fn clone_plugin_repos() -> Result<(), Box<dyn std::error::Error>> {
    empty_dir(Path::new(PLUGIN_REPO_PATH))?;

    let data = read_plugin_data()?;

    println!("Cloning plugin repositories...");

    let progress = MultiProgress::new();
    let total_progress = ProgressBar::new(data.len() as u64);
    let success_progress = ProgressBar::new(data.len() as u64);
    let skipped_progress = ProgressBar::new(data.len() as u64);
    let failed_progress = ProgressBar::new(data.len() as u64);

    progress.add(total_progress.clone());
    progress.add(success_progress.clone());
    progress.add(skipped_progress.clone());
    progress.add(failed_progress.clone());

    let now = std::time::Instant::now();

    ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global()
        .expect("Failed to build thread pool");

    let results: Vec<CloneResult> = data
        .into_par_iter()
        .map(|plugin| {
            if plugin.removed_commit.is_some() {
                total_progress.inc(1);
                skipped_progress.inc(1);
                return CloneResult::Skipped(plugin.id);
            }

            let clone_task = prep_clone(&plugin);
            let mut clone_task = match clone_task {
                Ok(task) => task,
                Err(e) => {
                    total_progress.inc(1);
                    failed_progress.inc(1);
                    return CloneResult::Failed(plugin.id, e.to_string());
                }
            };

            let clone_res: Result<_, String> = clone_task
                .fetch_then_checkout(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
                .map_err(|e| e.to_string())
                .and_then(|(mut checkout, _)| {
                    checkout
                        .main_worktree(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
                        .map_err(|e| e.to_string())
                });
            match clone_res {
                Ok(_) => {
                    total_progress.inc(1);
                    success_progress.inc(1);
                    return CloneResult::Success;
                }
                Err(e) => {
                    total_progress.inc(1);
                    failed_progress.inc(1);
                    return CloneResult::Failed(plugin.id, e);
                }
            }
        })
        .collect();

    total_progress.finish();
    success_progress.finish();
    skipped_progress.finish();
    failed_progress.finish();

    let failed_plugins: Vec<_> = results
        .iter()
        .filter_map(|result| match result {
            CloneResult::Failed(id, error) => Some((id, error)),
            _ => None,
        })
        .collect();

    let skipped_plugins: Vec<_> = results
        .iter()
        .filter_map(|result| match result {
            CloneResult::Skipped(id) => Some(id),
            _ => None,
        })
        .collect();

    println!("Skipped plugins: {:?}", skipped_plugins.len());
    println!("Failed plugins: {:?}", failed_plugins.len());
    println!();

    for (id, error) in failed_plugins {
        eprintln!("Failed to clone plugin {}: {}", id, error);
    }

    println!("Cloning completed in {:?}", now.elapsed());

    Ok(())
}

fn prep_clone(plugin: &PluginData) -> Result<gix::clone::PrepareFetch, Box<dyn std::error::Error>> {
    let clone = prepare_clone(
        gix::url::parse(
            format!("https://github.com/{}.git", plugin.current_entry.repo)
                .as_str()
                .into(),
        )?,
        Path::new(PLUGIN_REPO_PATH).join(plugin.id.clone()),
    )?
    .with_shallow(gix::remote::fetch::Shallow::DepthAtRemote(
        NonZero::new(1).unwrap(),
    ))
    .configure_remote(|remote| Ok(remote.with_fetch_tags(gix::remote::fetch::Tags::None)));

    Ok(clone)
}
