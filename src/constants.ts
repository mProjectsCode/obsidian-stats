import process from 'process';

export const RELEASE_STATS_URL = 'https://api.github.com/repos/obsidianmd/obsidian-releases/releases?page=1';
export const RELEASE_INFO_URL = 'https://obsidian.md/changelog.xml';

export const RELEASE_FULL_DATA_PATH = `releases-full-data.csv`;
export const RELEASE_WEEKLY_DATA_PATH = `releases-weekly-data.csv`;
export const RELEASE_CHANGELOG_PATH = `releases-changelog.csv`;

export const OBSIDIAN_RELEASES_PATH = 'obsidian-releases';
export const OBSIDIAN_RELEASES_FULL_PATH = `${process.cwd()}/${OBSIDIAN_RELEASES_PATH}`;
