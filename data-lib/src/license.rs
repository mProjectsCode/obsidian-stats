use serde::{Deserialize, Serialize};
use tsify::Tsify;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseData {
    pub title: String,
    #[serde(rename = "spdx-id")]
    pub spdx_id: String,
    pub description: String,
    pub how: String,
    // pub using: String, // we don't really need this field and the format of it is not consistent
    pub permissions: Vec<String>,
    pub conditions: Vec<String>,
    pub limitations: Vec<String>,
}

#[derive(Tsify, Debug, Clone, Serialize, Deserialize)]
#[tsify(into_wasm_abi)]
pub struct LicenseDescription {
    pub description: String,
    pub label: String,
    pub tag: String,
}

#[derive(Tsify, Debug, Clone, Serialize, Deserialize)]
#[tsify(into_wasm_abi)]
pub struct LicenseDescriptionNested {
    pub permissions: Vec<LicenseDescription>,
    pub conditions: Vec<LicenseDescription>,
    pub limitations: Vec<LicenseDescription>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Licenses {
    pub licenses: Vec<LicenseData>,
    pub permissions: Vec<String>,
    pub conditions: Vec<String>,
    pub limitations: Vec<String>,
    pub descriptions: LicenseDescriptionNested,
}