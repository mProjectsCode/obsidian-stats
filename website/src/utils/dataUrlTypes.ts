export interface DownloadRelationLocDataPoint {
	id: string;
	name: string;
	downloads: number;
	total_loc: number;
}

export interface DownloadRelationReleaseDateDataPoint {
	id: string;
	name: string;
	downloads: number;
	date: string;
}

export interface DownloadRelationVersionCountDataPoint {
	id: string;
	name: string;
	downloads: number;
	version_count: number;
}
