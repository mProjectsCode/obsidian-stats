use std::error::Error;

use crate::{
    plugins::{
        clone_repos::clone_plugin_repos, data::build_plugin_stats, extra::extract_extra_data,
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
    (step.run)()?;
    println!();
    Ok(())
}

fn process_plugin_licenses_step() -> Result<(), Box<dyn Error>> {
    process_licenses();
    Ok(())
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
            run: extract_extra_data,
        },
        PipelineStep {
            label: "Extracting licenses data",
            run: process_plugin_licenses_step,
        },
        PipelineStep {
            label: "Building release data",
            run: build_release_stats,
        },
    ];

    for step in &pipeline {
        run_pipeline_step(step)?;
    }

    println!("Done!");

    Ok(())
}
