use crate::pipeline::run_data_pipeline;

pub mod constants;
pub mod file_utils;
pub mod git_utils;
pub mod pipeline;
pub mod plugins;
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

    run_data_pipeline().expect("Failed to run data pipeline");
}
