use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
