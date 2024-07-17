import process from 'process';

export const PLUGIN_LIST_PATH = 'community-plugins.json';
export const PLUGIN_STATS_PATH = 'community-plugin-stats.json';
export const THEME_LIST_PATH = 'community-css-themes.json';
export const RELEASE_STATS_URL = 'https://api.github.com/repos/obsidianmd/obsidian-releases/releases?page=1';

export const PLUGIN_DATA_PATH = `plugin-data.json`;
export const THEME_DATA_PATH = `theme-data.json`;
export const RELEASE_FULL_DATA_PATH = `releases-full-data.csv`;
export const RELEASE_WEEKLY_DATA_PATH = `releases-weekly-data.csv`;

export const OBSIDIAN_RELEASES_PATH = 'obsidian-releases';
export const OBSIDIAN_RELEASES_FULL_PATH = `${process.cwd()}/${OBSIDIAN_RELEASES_PATH}`;
