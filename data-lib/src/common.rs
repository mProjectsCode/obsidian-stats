use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use tsify::Tsify;

use crate::{
    commit::{Commit, StringCommit},
    date::Date,
    version::Version,
};

pub const LOC_EXCLUDED: [&'static str; 24] = [
    "JSON",
    "SVG",
    "XML",
    "YAML",
    "AsciiDoc",
    "BASH",
    "Batch",
    "Dockerfile",
    "Edn",
    "Fish",
    "INI",
    "Jsonnet",
    "Makefile",
    "Markdown",
    "MSBuild",
    "Nix",
    "Org",
    "Plain Text",
    "PowerShell",
    "ReStructuredText",
    "Rakefile",
    "Shell",
    "TOML",
    "TeX",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryChange {
    pub property: String,
    pub commit: Commit,
    pub old_value: String,
    pub new_value: String,
}

impl EntryChange {
    pub fn to_data_point(&self) -> EntryChangeDataPoint {
        EntryChangeDataPoint {
            property: self.property.clone(),
            commit: self.commit.to_string_commit(),
            old_value: self.old_value.clone(),
            new_value: self.new_value.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DownloadHistory(pub HashMap<String, u32>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionHistory {
    pub version: String,
    #[serde(skip)]
    pub version_object: Option<Version>,
    pub initial_release_date: Date,
}

#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub struct OverviewDataPoint {
    pub id: String,
    pub name: String,
    pub author: String,
    pub repo: String,
    pub repo_url: String,
    pub added_commit: StringCommit,
    pub removed_commit: Option<StringCommit>,
}

#[derive(Tsify, Debug, Clone, Serialize, Deserialize)]
#[tsify(into_wasm_abi)]
pub struct DownloadDataPoint {
    pub date: String,
    pub downloads: Option<u32>,
    pub delta: Option<u32>,
}

#[derive(Tsify, Debug, Clone, Serialize, Deserialize)]
#[tsify(into_wasm_abi)]
pub struct VersionDataPoint {
    pub version: String,
    pub date: String,
    pub deprecated: bool,
}

#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub struct EntryChangeDataPoint {
    pub property: String,
    pub commit: StringCommit,
    pub old_value: String,
    pub new_value: String,
}

#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub struct IndividualDownloadDataPoint {
    pub id: String,
    pub name: String,
    pub date: String,
    pub downloads: u32,
    pub version_count: u32,
    pub total_loc: u32,
}

#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub struct HallOfFameDataPoint {
    pub id: String,
    pub name: String,
    pub downloads_new: u32,
    pub downloads_start: u32,
    pub data: Vec<DownloadDataPoint>,
}

#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub struct CountMonthlyDataPoint {
    pub date: String,
    pub total: u32,
    pub total_with_removed: u32,
    pub new: u32,
    pub new_removed: u32,
}

#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub struct RemovedByReleaseDataPoint {
    pub date: String,
    pub percentage: f64,
}

#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub struct InactivityByReleaseDataPoint {
    pub date: String,
    pub inactive_one_year: f64,
    pub inactive_two_years: f64,
    pub inactive_three_years: f64,
    pub inactive_four_years: f64,
    pub inactive_five_years: f64,
}

#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub struct NamedDataPoint {
    pub name: String,
    pub value: f64,
}

pub fn increment_named_data_points(points: &mut Vec<NamedDataPoint>, name: &str, value: f64) {
    if let Some(point) = points.iter_mut().find(|p| p.name == name) {
        point.value += value;
    } else {
        points.push(NamedDataPoint {
            name: name.to_string(),
            value,
        });
    }
}

pub fn to_percentage(value: &mut f64, total: f64) {
    if total == 0.0 {
        *value = 0.0;
    } else {
        *value = (*value / total) * 100.0;
    }
}

#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub struct StackedNamedDataPoint {
    pub name: String,
    pub layer: String,
    pub value: f64,
}
