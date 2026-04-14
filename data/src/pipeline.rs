use std::error::Error;

use crate::{
    alerts,
    latest_data_update::build_latest_data_update_summary,
    plugins::{
        analysis::extract_analysis_data, clone_repos::clone_plugin_repos, data::build_plugin_stats,
        license::process_licenses,
    },
    release::data::build_release_stats,
    theme::data::build_theme_stats,
};

type PipelineStepFn = fn() -> Result<(), Box<dyn Error>>;

struct PipelineStep {
    label: &'static str,
    run: PipelineStepFn,
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

fn process_plugin_licenses_step() -> Result<(), Box<dyn Error>> {
    process_licenses()
}

pub fn run_data_pipeline() -> Result<(), Box<dyn Error>> {
    let pipeline = [
        PipelineStep {
            label: "Building theme data",
            run: build_theme_stats,
        },
        PipelineStep {
            label: "Building plugin data",
            run: build_plugin_stats,
        },
        PipelineStep {
            label: "Cloning plugin repositories",
            run: clone_plugin_repos,
        },
        PipelineStep {
            label: "Extracting repository data",
            run: extract_analysis_data,
        },
        PipelineStep {
            label: "Extracting licenses data",
            run: process_plugin_licenses_step,
        },
        PipelineStep {
            label: "Building release data",
            run: build_release_stats,
        },
        PipelineStep {
            label: "Building latest data update summary",
            run: build_latest_data_update_summary,
        },
    ];

    for step in &pipeline {
        run_pipeline_step(step)?;
    }

    println!("Done!");

    Ok(())
}
