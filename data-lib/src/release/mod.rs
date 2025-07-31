use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use tsify::Tsify;

use crate::{date::Date, version::Version};

pub mod data_array;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubAssetInfo {
    pub name: String,
    pub downloads: HashMap<String, u32>,
    pub size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubReleaseInfo {
    pub version: Version,
    pub date: Date,
    pub time: String,
    pub assets: Vec<GithubAssetInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ObsidianPlatform {
    Desktop,
    Mobile,
    Publish,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsidianReleaseInfo {
    pub version: Version,
    pub platform: ObsidianPlatform,
    pub insider: bool,
    pub date: Date,
    pub info: String,
    pub major_release: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum OS {
    Windows,
    MacOS,
    Linux,
}

impl OS {
    pub fn to_fancy_string(&self) -> String {
        match self {
            OS::Windows => "Windows".into(),
            OS::MacOS => "macOS".into(),
            OS::Linux => "Linux".into(),
        }
    }

    pub fn from_asset_name(asset_name: &str) -> Option<Self> {
        if asset_name.ends_with(".asar.gz") {
            None
        } else if asset_name.ends_with(".dmg") {
            Some(OS::MacOS)
        } else if asset_name.ends_with(".exe") {
            Some(OS::Windows)
        } else {
            Some(OS::Linux)
        }
    }
}

pub trait ToFancyString {
    fn to_fancy_string(&self) -> String;
}

impl ToFancyString for Option<OS> {
    fn to_fancy_string(&self) -> String {
        match self {
            Some(os) => os.to_fancy_string(),
            None => "Unknown".into(),
        }
    }
}

// | Distribution                         | OS      | Type    | Architecture | Comments                         |
// |--------------------------------------|---------|---------|--------------|----------------------------------|
// | obsidian-x.y.z.asar.gz               | N/A     | N/A     | N/A          | File downloaded by updater       |
// | Obsidian-x.y.z-universal.dmg         | MacOS   |         | x86/ARM      |                                  |
// | Obsidian-x.y.z.AppImage              | Linux   |         | x86          |                                  |
// | Obsidian-x.y.z-arm64.AppImage        | Linux   |         | ARM          |                                  |
// | obsidian-x.y.z-arm64.tar.gz          | Linux   |         | ARM          |                                  |
// | obsidian_x.y.z_amd64.deb             | Linux   | Debian  | x86          |                                  |
// | obsidian_x.y.z_amd64.snap            | Linux   | Snap    | x86          |                                  |
// | obsidian-x.y.z.tar.gz                | Linux   |         | x86          |                                  |
// | obsidian-x.y.z-32.exe                | Windows |         | x86-32       | Legacy 32bit                     |
// | obsidian-x.y.z-allusers.exe          | Windows |         | x86          | Installed for all users          |
// | obsidian-x.y.z.exe                   | Windows |         | x86          | Regular windows installer        |
// | obsidian_x.y.z_arm64.exe             | Windows |         | ARM          |                                  |

pub fn get_asset_release_file_type(file_name: &str) -> Option<String> {
    // for some reason, the exe files sometimes contain a dot between obsidian and the version
    // e.g. Obsidian.x.y.z.exe
    // to work around that we just conclude that if the file name ends with .exe, it is an exe file
    if file_name.ends_with(".exe") {
        return Some("exe".into());
    }

    // file name is of the form "obsidian-1.6.7-arm64.tar.gz"
    // since the version has two dots, we can split by dot, ignore the first 3 parts, and join the rest
    let parts: Vec<&str> = file_name.split('.').collect();
    if parts.len() < 4 {
        None
    } else {
        Some(parts[3..].join("."))
    }
}

pub fn get_asset_cpu_instruction_set(file_name: &str) -> Option<&'static str> {
    if file_name.ends_with(".dmg") {
        Some("both (.dmg)")
    } else if file_name.contains("arm64") {
        Some("arm64")
    } else {
        Some("x86")
    }
}

#[derive(Tsify, Debug, Clone, Serialize, Deserialize)]
#[tsify(into_wasm_abi)]
pub struct ChangelogDataPoint {
    pub minor_version: String,
    pub first_public_version: String,
    pub public_release_date: String,
    pub insider_release_date: String,
    pub number_of_insider_patches: usize,
    pub number_of_patches: usize,
}

#[derive(Tsify, Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[tsify(into_wasm_abi)]
pub enum ChangeLogChangeCategory {
    BreakingChange,
    BugFix,
    Developer,
    Feature,
    Improvement,
    Uncategorized,
}

impl ChangeLogChangeCategory {
    pub fn to_fancy_string(&self) -> String {
        match self {
            ChangeLogChangeCategory::BreakingChange => "Breaking Change".into(),
            ChangeLogChangeCategory::BugFix => "Bug Fix".into(),
            ChangeLogChangeCategory::Developer => "Developer".into(),
            ChangeLogChangeCategory::Feature => "Feature".into(),
            ChangeLogChangeCategory::Improvement => "Improvement".into(),
            ChangeLogChangeCategory::Uncategorized => "Uncategorized".into(),
        }
    }

    pub fn iter() -> impl Iterator<Item = Self> {
        [
            ChangeLogChangeCategory::Uncategorized,
            ChangeLogChangeCategory::BugFix,
            ChangeLogChangeCategory::Improvement,
            ChangeLogChangeCategory::Feature,
            ChangeLogChangeCategory::Developer,
            ChangeLogChangeCategory::BreakingChange,
        ]
        .iter()
        .cloned()
    }
}

impl From<Option<ChangeLogChangeCategory>> for ChangeLogChangeCategory {
    fn from(value: Option<ChangeLogChangeCategory>) -> Self {
        value.unwrap_or(ChangeLogChangeCategory::Uncategorized)
    }
}

const CHANGELOG_CATEGORIES_MAP: &[(&str, ChangeLogChangeCategory)] = &[
    ("Uncategorized", ChangeLogChangeCategory::Uncategorized),
    ("Shiny new things", ChangeLogChangeCategory::Feature),
    ("No longer broken", ChangeLogChangeCategory::BugFix),
    ("Changes", ChangeLogChangeCategory::Improvement),
    ("Breaking changes", ChangeLogChangeCategory::BreakingChange),
    ("Improvements", ChangeLogChangeCategory::Improvement),
    ("Developers", ChangeLogChangeCategory::Developer),
    ("Migration notice", ChangeLogChangeCategory::Uncategorized),
    ("Developer", ChangeLogChangeCategory::Developer),
    (
        "Shiny new things for Obsidian Publish",
        ChangeLogChangeCategory::Feature,
    ),
    ("For Developers", ChangeLogChangeCategory::Developer),
    ("Bug fixes", ChangeLogChangeCategory::BugFix),
    ("New shiny things", ChangeLogChangeCategory::Feature),
    ("For developers", ChangeLogChangeCategory::Developer),
    ("Improved", ChangeLogChangeCategory::Improvement),
];

impl From<&str> for ChangeLogChangeCategory {
    fn from(value: &str) -> Self {
        for (keyword, category) in CHANGELOG_CATEGORIES_MAP {
            if value == *keyword {
                return category.clone();
            }
        }
        ChangeLogChangeCategory::Uncategorized
    }
}

#[derive(Tsify, Debug, Clone, Serialize, Deserialize)]
#[tsify(into_wasm_abi)]
pub struct ChangelogChanges {
    pub version: Version,
    pub version_string: String,
    pub changes: HashMap<ChangeLogChangeCategory, usize>,
}
