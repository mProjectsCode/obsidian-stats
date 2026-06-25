use crate::pipeline::run_data_pipeline;

pub mod alerts;
pub mod constants;
pub mod file_utils;
pub mod git_utils;
pub mod github;
pub mod latest_data_update;
pub mod pipeline;
pub mod plugins;
pub mod progress;
pub mod release;
pub mod security;
pub mod state;
pub mod theme;

fn load_env() {
    // Support both running from `data/` and from workspace root.
    let _ = dotenvy::from_filename(".env");
    let _ = dotenvy::from_filename("../.env");
}

fn main() {
    load_env();
    alerts::install_panic_hook();

    let Some(options) = parse_cli_args() else {
        return;
    };

    if let Err(error) = run_data_pipeline(options) {
        alerts::print_summary();
        eprintln!("Data pipeline failed: {error}");
    }

    if let Err(error) = alerts::fail_if_any() {
        eprintln!("Data pipeline failed: {error}");
    }
}

fn parse_cli_args() -> Option<pipeline::PipelineOptions> {
    let mut options = pipeline::PipelineOptions::default();

    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--force" => options.force = true,
            "--no-clone" => options.no_clone = true,
            "--no-release" => options.no_release = true,
            "-h" | "--help" => {
                print_usage();
                return None;
            }
            _ => {
                eprintln!("Unknown argument: {arg}");
                print_usage();
                std::process::exit(2);
            }
        }
    }

    Some(options)
}

fn print_usage() {
    println!("Usage: data [--force] [--no-clone] [--no-release]");
    println!();
    println!("  --force       Ignore refresh windows and refresh cached GitHub data.");
    println!("  --no-clone    Skip repository recloning but run the remaining pipeline steps.");
    println!("  --no-release  Skip release acquisition but run the remaining pipeline steps.");
}
