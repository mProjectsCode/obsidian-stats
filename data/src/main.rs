use crate::{
    plugins::{
        clone_repos::clone_plugin_repos, data::build_plugin_stats, extra::extract_extra_data,
        license::process_licenses,
    },
    theme::data::build_theme_stats,
};

pub mod constants;
pub mod file_utils;
pub mod git_utils;
pub mod plugins;
pub mod theme;

fn main() {
    build_theme_stats().expect("Failed to build theme stats");

    println!();

    build_plugin_stats().expect("Failed to build plugin stats");

    // println!();
    // println!("Cloning plugin repositories...");

    // clone_plugin_repos().expect("Failed to clone plugin repositories");

    // println!();
    // println!("Extracting repository data...");

    // extract_extra_data().expect("Failed to extract repository data");

    process_licenses();

    println!();
    println!("Done!");
}
