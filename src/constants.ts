import process from "process";

export const PLUGIN_LIST_PATH = 'community-plugins.json';
export const PLUGIN_STATS_PATH = 'community-plugin-stats.json';
export const THEME_LIST_PATH = 'community-css-themes.json';
export const RELEASE_STATS_URL = 'https://api.github.com/repos/obsidianmd/obsidian-releases/releases?page=1';

export const PLUGIN_DATA_PATH = `plugin-data.json`;
export const THEME_DATA_PATH = `theme-data.json`;
export const RELEASE_FULL_DATA_PATH = `releases-full-data.json`;
export const RELEASE_DAILY_DATA_PATH = `releases-daily-data.csv`;

export const OBSIDIAN_RELEASES_PATH = 'obsidian-releases';
export const OBSIDIAN_RELEASES_FULL_PATH = `${process.cwd()}/${OBSIDIAN_RELEASES_PATH}`;

export const PLUGIN_TEMPLATE_FILE_PATH = 'src/plugin/plugin_template.txt';
export const PLUGIN_TEMPLATE_REPLACEMENT_STRING = 'PLUGIN_ID';
export const PLUGIN_TEMPLATE_REPLACEMENT_STRING_JSON = 'PLUGIN_JSON';
export const PLUGIN_TEMPLATE_OUTPUT_PATH = 'website/src/content/docs/plugins';

export const THEME_TEMPLATE_FILE_PATH = 'src/theme/theme_template.txt';
export const THEME_TEMPLATE_REPLACEMENT_STRING = 'THEME_ID';
export const THEME_TEMPLATE_REPLACEMENT_STRING_JSON = 'THEME_JSON';
export const THEME_TEMPLATE_OUTPUT_PATH = 'website/src/content/docs/themes';
