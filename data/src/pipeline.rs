use std::error::Error;

use crate::{
    alerts,
    latest_data_update::build_latest_data_update_summary,
    plugins::{
        analysis::extract_analysis_data,
        clone_repos::clone_plugin_repos,
        data::{build_plugin_stats, read_plugin_data},
        license::process_licenses,
        release_acquisition::acquire_plugin_release_main_js,
    },
    release::data::build_release_stats,
    theme::data::build_theme_stats,
};

struct PipelineStep {
    label: &'static str,
    run: Box<dyn Fn() -> Result<(), Box<dyn Error>>>,
}

fn run_pipeline_step(step: &PipelineStep) -> Result<(), Box<dyn Error>> {
    println!("{}...", step.label);
    let alert_count_before = alerts::alert_count();
    if let Err(error) = (step.run)() {
        if alerts::alert_count() == alert_count_before {
            alerts::record_unexpected_error(step.label, error.to_string());
        }
        return Err(error);
    }
    println!();
    Ok(())
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PipelineOptions {
    pub force: bool,
    pub no_clone: bool,
}

fn process_plugin_licenses_step() -> Result<(), Box<dyn Error>> {
    process_licenses()
}

fn acquire_plugin_releases_step(force: bool) -> Result<(), Box<dyn Error>> {
    let plugin_data = read_plugin_data()?;
    acquire_plugin_release_main_js(&plugin_data, force)
}

pub fn run_data_pipeline(options: PipelineOptions) -> Result<(), Box<dyn Error>> {
    let pipeline: Vec<PipelineStep> = vec![
        PipelineStep {
            label: "Building theme data",
            run: Box::new(build_theme_stats),
        },
        PipelineStep {
            label: "Building plugin data",
            run: Box::new(build_plugin_stats),
        },
        PipelineStep {
            label: "Cloning plugin repositories",
            run: Box::new(move || clone_plugin_repos(options.force, options.no_clone)),
        },
        PipelineStep {
            label: "Acquiring plugin release assets",
            run: Box::new(move || acquire_plugin_releases_step(options.force)),
        },
        PipelineStep {
            label: "Extracting repository data",
            run: Box::new(extract_analysis_data),
        },
        PipelineStep {
            label: "Extracting licenses data",
            run: Box::new(process_plugin_licenses_step),
        },
        PipelineStep {
            label: "Building release data",
            run: Box::new(move || build_release_stats(options.force)),
        },
        PipelineStep {
            label: "Building latest data update summary",
            run: Box::new(build_latest_data_update_summary),
        },
    ];

    for step in &pipeline {
        run_pipeline_step(step)?;
    }

    println!("Done!");

    Ok(())
}
