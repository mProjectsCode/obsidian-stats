use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use tsify::Tsify;

use crate::{
    commit::{Commit, StringCommit},
    date::Date,
    version::Version,
};

pub const LOC_EXCLUDED: &[&str] = &[
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

pub const FILE_EXT_INCLUDED: &[&str] = &[
    "ts", "tsx", "mts", "cts", "js", "jsx", "mjs", "cjs", "css", "svelte", "vue", "scss", "sass",
    "py", "rs", "java", "kt", "rb", "c", "cpp", "h", "hpp",
];

// list of i18n adjacent libraries, thanks chatgpt
pub const I18N_DEPENDENCIES: &[&str] = &[
    "i18next",
    "i18n",
    "node-polyglot",
    "formatjs",
    "intl-messageformat",
    "intl-relativeformat",
    "messageformat",
    "react-intl",
    "react-i18next",
    "next-i18next",
    "next-intl",
    "@lingui/core",
    "@lingui/react",
    "vue-i18n",
    "vue-i18next",
    "@ngx-translate/core",
    "@angular/localize",
    "react-native-localize",
    "expo-localization",
    "svelte-i18n",
    "intl",
    "globalize",
];

pub const I18N_LOCALE_CODES: &[&str] = &[
    "Cy-az-AZ", "Cy-sr-SP", "Cy-uz-UZ", "Lt-az-AZ", "Lt-sr-SP", "Lt-uz-UZ", "aa", "ab", "ae", "af",
    "af-ZA", "ak", "am", "an", "ar", "ar-AE", "ar-BH", "ar-DZ", "ar-EG", "ar-IQ", "ar-JO", "ar-KW",
    "ar-LB", "ar-LY", "ar-MA", "ar-OM", "ar-QA", "ar-SA", "ar-SY", "ar-TN", "ar-YE", "as", "av",
    "ay", "az", "ba", "be", "be-BY", "bg", "bg-BG", "bh", "bi", "bm", "bn", "bo", "br", "bs", "ca",
    "ca-ES", "ce", "ch", "co", "cr", "cs", "cs-CZ", "cu", "cv", "cy", "da", "da-DK", "de", "de-AT",
    "de-CH", "de-DE", "de-LI", "de-LU", "div-MV", "dv", "dz", "ee", "el", "el-GR", "en", "en-AU",
    "en-BZ", "en-CA", "en-CB", "en-GB", "en-IE", "en-JM", "en-NZ", "en-PH", "en-TT", "en-US",
    "en-ZA", "en-ZW", "eo", "es", "es-AR", "es-BO", "es-CL", "es-CO", "es-CR", "es-DO", "es-EC",
    "es-ES", "es-GT", "es-HN", "es-MX", "es-NI", "es-PA", "es-PE", "es-PR", "es-PY", "es-SV",
    "es-UY", "es-VE", "et", "et-EE", "eu", "eu-ES", "fa", "fa-IR", "ff", "fi", "fi-FI", "fj", "fo",
    "fo-FO", "fr", "fr-BE", "fr-CA", "fr-CH", "fr-FR", "fr-LU", "fr-MC", "fy", "ga", "gd", "gl",
    "gl-ES", "gn", "gu", "gu-IN", "gv", "ha", "he", "he-IL", "hi", "hi-IN", "ho", "hr", "hr-HR",
    "ht", "hu", "hu-HU", "hy", "hy-AM", "hz", "ia", "id", "id-ID", "ie", "ig", "ii", "ik", "io",
    "is", "is-IS", "it", "it-CH", "it-IT", "iu", "ja", "ja-JP", "jv", "ka", "ka-GE", "kg", "ki",
    "kj", "kk", "kk-KZ", "kl", "km", "kn", "kn-IN", "ko", "ko-KR", "kr", "ks", "ku", "kv", "kw",
    "ky", "ky-KZ", "la", "lb", "lg", "li", "ln", "lo", "lt", "lt-LT", "lu", "lv", "lv-LV", "mg",
    "mh", "mi", "mk", "mk-MK", "ml", "mn", "mn-MN", "mr", "mr-IN", "ms", "ms-BN", "ms-MY", "mt",
    "my", "na", "nb", "nb-NO", "nd", "ne", "ng", "nl", "nl-BE", "nl-NL", "nn", "nn-NO", "no", "nr",
    "nv", "ny", "oc", "oj", "om", "or", "os", "pa", "pa-IN", "pi", "pl", "pl-PL", "ps", "pt",
    "pt-BR", "pt-PT", "qu", "rm", "rn", "ro", "ro-RO", "ru", "ru-RU", "rw", "sa", "sa-IN", "sc",
    "sd", "se", "sg", "si", "sk", "sk-SK", "sl", "sl-SI", "sm", "sn", "so", "sq", "sq-AL", "sr",
    "ss", "st", "su", "sv", "sv-FI", "sv-SE", "sw", "sw-KE", "ta", "ta-IN", "te", "te-IN", "tg",
    "th", "th-TH", "ti", "tk", "tl", "tn", "to", "tr", "tr-TR", "ts", "tt", "tt-RU", "tw", "ty",
    "ug", "uk", "uk-UA", "ur", "ur-PK", "uz", "ve", "vi", "vi-VN", "vo", "wa", "wo", "xh", "yi",
    "yo", "za", "zh", "zh-CHS", "zh-CHT", "zh-CN", "zh-HK", "zh-MO", "zh-SG", "zh-TW", "zu",
];

pub const I18N_FILE_ENDINGS: &[&str] = &[".json", ".yaml", ".yml", ".js", ".ts"];

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
pub struct MultiDownloadDataPoint {
    pub date: String,
    pub category: String,
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

#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub struct MilestoneMonthGroup {
    pub month: String,
    pub milestones: Vec<MilestoneDataPoint>,
}

#[derive(Tsify, Debug, Clone, Serialize)]
#[tsify(into_wasm_abi)]
pub struct MilestoneDataPoint {
    pub milestone_type: String,
    pub milestone_value: u32,
    pub date: String,
    pub plugin_id: Option<String>,
}
