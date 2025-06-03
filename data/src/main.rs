use crate::plugins::{data::build_plugin_stats, repo::data::extract_repo_data};

pub mod commit;
pub mod common;
pub mod constants;
pub mod date;
pub mod input_data;
pub mod plugins;
pub mod version;

fn main() {
    // build_plugin_stats();

    println!();
    println!("Extracting repository data...");

    extract_repo_data().expect("Failed to extract repository data");
}
