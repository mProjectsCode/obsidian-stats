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
	downloads: number;
	releases: number;
	initialReleaseDate: number;
	initialReleaseDateString: string;
}
