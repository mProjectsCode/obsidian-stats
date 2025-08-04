use crate::{
    plugins::{
        clone_repos::clone_plugin_repos, data::build_plugin_stats, extra::extract_extra_data,
        license::process_licenses,
    },
    release::data::build_release_stats,
    theme::data::build_theme_stats,
};

pub mod constants;
pub mod file_utils;
pub mod git_utils;
pub mod plugins;
pub mod release;
pub mod theme;

fn main() {
    println!("Building theme data...");

    build_theme_stats().expect("Failed to build theme stats");

    println!();
    println!("Building plugin data...");

    build_plugin_stats().expect("Failed to build plugin stats");

    println!();
    println!("Cloning plugin repositories...");

    clone_plugin_repos().expect("Failed to clone plugin repositories");

    println!();
    println!("Extracting repository data...");

    extract_extra_data().expect("Failed to extract repository data");

    println!();
    println!("Extracting licenses data...");

    process_licenses();

    println!();
    println!("Building release data...");

    build_release_stats().expect("Failed to build release stats");

    println!();
    println!("Done!");
}
