use crate::pipeline::run_data_pipeline;

pub mod alerts;
pub mod constants;
pub mod file_utils;
pub mod git_utils;
pub mod latest_data_update;
pub mod pipeline;
pub mod plugins;
pub mod progress;
pub mod release;
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

    if let Err(error) = run_data_pipeline() {
        alerts::print_summary();
        eprintln!("Data pipeline failed: {error}");
    }

    if let Err(error) = alerts::fail_if_any() {
        eprintln!("Data pipeline failed: {error}");
    }
}
