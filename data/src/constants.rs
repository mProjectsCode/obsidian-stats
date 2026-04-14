pub const OBS_RELEASES_REPO_PATH: &str = "../obsidian-releases";

pub const PLUGIN_LIST_PATH: &str = "community-plugins.json";
pub const PLUGIN_STATS_PATH: &str = "community-plugin-stats.json";
pub const PLUGIN_DEPRECATIONS_PATH: &str = "community-plugin-deprecation.json";
pub const PLUGIN_REMOVED_PATH: &str = "community-plugins-removed.json";

pub const THEME_LIST_PATH: &str = "community-css-themes.json";
pub const THEME_REMOVED_PATH: &str = "community-css-themes-removed.json";

pub const RELEASE_STATS_URL: &str =
    "https://api.github.com/repos/obsidianmd/obsidian-releases/releases?page=1";
pub const RELEASE_STATS_TEST_PATH: &str = "./out/releases-github-test-response";
pub const RELEASE_INFO_URL: &str = "https://obsidian.md/changelog.xml";

pub const PLUGIN_DATA_PATH: &str = "./out/plugin-data";
pub const PLUGIN_REPO_PATH: &str = "./out/plugin-repos";
pub const PLUGIN_REPO_DATA_PATH: &str = "./out/plugin-repo-data";
pub const PLUGIN_RELEASE_MAIN_JS_PATH: &str = "./out/plugin-release-mainjs";
pub const THEME_DATA_PATH: &str = "./out/theme-data";

pub const RELEASE_GITHUB_RAW_PATH: &str = "./out/releases-github-raw";
pub const RELEASE_GITHUB_INTERPOLATED_PATH: &str = "./out/releases-github-interpolated";
pub const RELEASE_CHANGELOG_PATH: &str = "./out/releases-changelog";

pub const STATE_PATH: &str = "./out/state";
pub const PLUGIN_RELEASE_ENRICHMENT_STATE_PATH: &str =
    "./out/state/plugin-release-enrichment-state.json";
pub const CLONE_STATE_PATH: &str = "./out/state/clone-state.json";
pub const RELEASE_STATS_STATE_PATH: &str = "./out/state/release-stats-state.json";
pub const LATEST_DATA_UPDATE_SUMMARY_PATH: &str = "./out/state/latest-data-update-summary.json";

pub const DEFAULT_PLUGIN_RELEASE_REFRESH_DAYS: i64 = 3;
pub const DEFAULT_CLONE_REFRESH_DAYS: i64 = 3;
pub const DEFAULT_RELEASE_STATS_REFRESH_DAYS: i64 = 3;

pub const GITHUB_RATE_LIMIT_MODE_ENV: &str = "GITHUB_RATE_LIMIT_MODE";
