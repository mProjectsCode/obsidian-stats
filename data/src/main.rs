use crate::plugins::{data::build_plugin_stats, repo::extract_repo_data};

pub mod constants;
pub mod file_utils;
pub mod plugins;

fn main() {
    build_plugin_stats().expect("Failed to build plugin stats");

    println!();
    println!("Extracting repository data...");

    extract_repo_data().expect("Failed to extract repository data");

    println!();
    println!("Done!");
}
