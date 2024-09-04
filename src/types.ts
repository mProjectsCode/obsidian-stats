export interface Commit {
	date: string;
	hash: string;
}

export interface EntryChange {
	property: string;
	commit: Commit;
	oldValue: string;
	newValue: string;
}

export type DownloadHistory = Record<string, number>;

export interface VersionHistory {
	version: string;
	initialReleaseDate: string;
}

export interface DownloadDataPoint {
	date: string;
	downloads?: number;
	growth?: number;
}

export interface PerMonthDataPoint {
	year: string;
	month: string;
	value: number;
}

export interface DownloadReleaseCorrelationDataPoint {
	id: string;
	name: string;
	downloads: number;
	releases: number;
	initialReleaseDate: number;
	initialReleaseDateString: string;
}

export interface ORR_CommunityPluginRemoved {
	id: string;
	name: string;
	reason: string;
}

export interface ORR_CommunityPlugin {
	id: string;
	name: string;
	author: string;
	description: string;
	repo: string;
}

export interface ORR_CommunityTheme {
	name: string;
	author: string;
	repo: string;
	screenshot: string;
	modes: ['dark'] | ['light'] | ['dark', 'light'] | ['light', 'dark'];
	legacy?: boolean;
}

// record of plugin id to list of deprecated versions
export type ORR_CommunityPluginDeprecations = Record<string, string[]>;
