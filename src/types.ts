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

export type DownloadHistory = Record<string, number>

export interface VersionHistory {
	version: string;
	initialReleaseDate: string;
}