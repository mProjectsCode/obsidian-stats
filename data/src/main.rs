use crate::plugins::{
    clone_repos::clone_plugin_repos, data::build_plugin_stats, extra::extract_extra_data,
};

pub mod constants;
pub mod file_utils;
pub mod plugins;

fn main() {
    // build_plugin_stats().expect("Failed to build plugin stats");

    // println!();
    // println!("Cloning plugin repositories...");

    // clone_plugin_repos().expect("Failed to clone plugin repositories");

    println!();
    println!("Extracting repository data...");

    extract_extra_data().expect("Failed to extract repository data");

    println!();
    println!("Done!");
}
