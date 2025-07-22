use std::{num::NonZero, path::Path};

use data_lib::plugin::PluginData;
use gix::prepare_clone;
use indicatif::ProgressBar;

use crate::{constants::PLUGIN_REPO_PATH, file_utils::empty_dir, plugins::data::read_plugin_data};

pub fn clone_plugin_repos() -> Result<(), Box<dyn std::error::Error>> {
    empty_dir(Path::new(PLUGIN_REPO_PATH))?;

    let data = read_plugin_data()?;

    let mut skipped_plugins = vec![];
    let mut failed_plugins = vec![];

    let progress = ProgressBar::new(data.len() as u64);
    let now = std::time::Instant::now();
    let progress = progress.with_elapsed(now.elapsed());

    data.into_iter().for_each(|plugin| {
        if plugin.removed_commit.is_some() {
            skipped_plugins.push(plugin.id);
            progress.inc(1);
            return;
        }

        let clone_task = prep_clone(&plugin);
        let mut clone_task = match clone_task {
            Ok(task) => task,
            Err(e) => {
                failed_plugins.push((plugin.id, e.to_string()));
                progress.inc(1);
                return;
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
                // TODO
            }
            Err(e) => {
                failed_plugins.push((plugin.id, e));
            }
        }

        progress.inc(1);
    });

    progress.finish();

    println!("Skipped plugins: {:?}", skipped_plugins.len());
    println!("Failed plugins: {:?}", failed_plugins.len());
    println!();

    for (id, error) in failed_plugins {
        eprintln!("Failed to clone plugin {}: {}", id, error);
    }

    println!(
        "Cloning completed in {:?}",
        now.elapsed()
    );

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
