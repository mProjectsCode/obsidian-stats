import type { PluginData } from '../../../src/types.ts';

export function dateToString(date: Date): string {
	return `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2, '0')}-${date.getDate().toString().padStart(2, '0')}`;
}

export interface DownloadDataPoint {
	date: string;
	downloads?: number;
	growth?: number;
}

export function getDownloadDataPoints(plugins: PluginData[]): DownloadDataPoint[] {
	const data: DownloadDataPoint[] = [];

	for (const plugin of plugins) {
		const downloadHistoryData = Object.entries(plugin.downloadHistory);
		if (downloadHistoryData.length === 0) continue;

		const startDate = new Date(downloadHistoryData[0][0]);
		const endDate = new Date(downloadHistoryData[downloadHistoryData.length - 1][0]);

		startDate.setDate(startDate.getDate() + (7 - startDate.getDay()));

		let lastDownloadCount: number | undefined = 0;

		for (let d = startDate; d <= endDate; d.setDate(d.getDate() + 7)) {
			const date = dateToString(d);

			const downloadCount = downloadHistoryData.find(d => d[0] === date)?.[1];
			let growth: number | undefined = undefined;
			if (downloadCount !== undefined && lastDownloadCount !== undefined) {
				growth = downloadCount - lastDownloadCount;
			}
			lastDownloadCount = downloadCount;

			const existingData = data.find(x => x.date === date);
			if (existingData !== undefined) {
				if (existingData.downloads === undefined) {
					existingData.downloads = downloadCount;
				} else {
					if (downloadCount !== undefined) {
						existingData.downloads += downloadCount;
					}
				}

				if (existingData.growth === undefined) {
					existingData.growth = growth;
				} else {
					if (growth !== undefined) {
						existingData.growth += growth;
					}
				}
			} else {
				data.push({
					date: date,
					downloads: downloadCount,
					growth: growth,
				});
			}
		}
	}

	return data.sort((a, b) => a.date.localeCompare(b.date));
}

export interface DownloadReleaseCorrelationDataPoint {
	id: string;
	downloads: number;
	releases: number;
	initialReleaseDate: number;
	initialReleaseDateString: string;
}

export function getDownloadReleaseCorrelationDataPoints(plugins: PluginData[]): DownloadReleaseCorrelationDataPoint[] {
	const data: DownloadReleaseCorrelationDataPoint[] = [];

	for (const plugin of plugins) {
		const downloadKeys = Object.keys(plugin.downloadHistory);

		const downloads = plugin.downloadHistory[downloadKeys[downloadKeys.length - 1]];
		const initialReleaseDate = new Date(plugin.addedCommit.date).valueOf();
		const initialReleaseDateString = dateToString(new Date(plugin.addedCommit.date));
		const releases = plugin.versionHistory.length;

		data.push({
			id: plugin.id,
			downloads: downloads,
			releases: releases,
			initialReleaseDate: initialReleaseDate,
			initialReleaseDateString: initialReleaseDateString,
		});
	}

	return data;
}
