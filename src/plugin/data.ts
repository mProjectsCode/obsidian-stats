import { type PluginDataInterface } from './plugin.ts';
import { type DownloadDataPoint, type DownloadReleaseCorrelationDataPoint, type PerMonthDataPoint } from '../types.ts';
import { filterNonRemoved, filterRemoved, getAddedDataForMonth, getRemovedDataForMonth, iterateDataMonthly } from '../utils.ts';
import { reduce } from 'itertools-ts';
import { CDate } from '../date.ts';

export function getPluginDownloadsWeekly(plugins: PluginDataInterface[]): DownloadDataPoint[] {
	const data: DownloadDataPoint[] = [];

	for (const plugin of plugins) {
		const downloadHistoryData = Object.entries(plugin.downloadHistory);
		if (downloadHistoryData.length === 0) continue;

		const startDateString = reduce.toMin(downloadHistoryData, d => d[0])![0];
		const endDateString = reduce.toMax(downloadHistoryData, d => d[0])![0];

		const startDate = CDate.fromString(startDateString);
		const endDate = CDate.fromString(endDateString);

		data.push(
			...(CDate.iterateWeekly<DownloadDataPoint | undefined>(startDate, endDate, date => {
				const dateString = date.toString();

				const downloadCount: number | undefined = downloadHistoryData.find(d => d[0] === dateString)?.[1];

				const existingData = data.find(x => x.date === dateString);

				if (existingData !== undefined) {
					// update existing data
					if (downloadCount !== undefined) {
						if (existingData.downloads === undefined) {
							existingData.downloads = downloadCount;
						} else {
							existingData.downloads += downloadCount;
						}
					}

					return undefined;
				} else {
					return {
						date: dateString,
						downloads: downloadCount,
						growth: 0,
					};
				}
			}).filter(x => x !== undefined) as DownloadDataPoint[]),
		);
	}

	data.sort((a, b) => a.date.localeCompare(b.date));

	for (let i = 1; i < data.length; i++) {
		const currentDownloads = data[i].downloads;
		const previousDownloads = data[i - 1].downloads;

		if (currentDownloads === undefined || previousDownloads === undefined) continue;

		data[i].growth = currentDownloads - previousDownloads;
	}

	return data;
}

export function getDownloadReleaseCorrelationDataPoints(plugins: PluginDataInterface[]): DownloadReleaseCorrelationDataPoint[] {
	const data: DownloadReleaseCorrelationDataPoint[] = [];

	for (const plugin of plugins) {
		const downloadKeys = Object.keys(plugin.downloadHistory);

		const addedDate = CDate.fromString(plugin.addedCommit.date);

		data.push({
			id: plugin.id,
			name: plugin.currentEntry.name,
			downloads: plugin.downloadHistory[downloadKeys[downloadKeys.length - 1]] ?? 0,
			releases: plugin.versionHistory.length,
			initialReleaseDate: addedDate.toUTC(),
			initialReleaseDateString: addedDate.toString(),
		});
	}

	return data;
}

export function getPluginRemovedList(plugins: PluginDataInterface[]): PluginDataInterface[] {
	return filterRemoved(plugins).sort((a, b) => a.id.localeCompare(b.id));
}

export function getPluginRemovedRecentList(plugins: PluginDataInterface[]): PluginDataInterface[] {
	return filterRemoved(plugins)
		.sort((a, b) => b.removedCommit!.date.localeCompare(a.removedCommit!.date))
		.slice(0, 15);
}

export function getPluginPercentageRemovedByReleaseMonth(plugins: PluginDataInterface[]): PerMonthDataPoint[] {
	return iterateDataMonthly<PerMonthDataPoint>(plugins, (d, year, month) => {
		const releasedPlugins = getAddedDataForMonth(plugins, year, month);
		const retiredPlugins = filterRemoved(releasedPlugins);

		return {
			year: year,
			month: month,
			value: (retiredPlugins.length / releasedPlugins.length) * 100,
		} satisfies PerMonthDataPoint;
	});
}

export function getPluginCountAddedMonthly(plugins: PluginDataInterface[]): PerMonthDataPoint[] {
	return iterateDataMonthly<PerMonthDataPoint>(plugins, (d, year, month) => {
		const releasedPlugins = getAddedDataForMonth(plugins, year, month);

		return {
			year: year,
			month: month,
			value: releasedPlugins.length,
		};
	});
}

export function getPluginAddedRecentList(plugins: PluginDataInterface[]): PluginDataInterface[] {
	return filterNonRemoved(plugins)
		.sort((a, b) => b.addedCommit.date.localeCompare(a.addedCommit.date))
		.slice(0, 15);
}

export function getPluginCountMonthly(plugins: PluginDataInterface[]): PerMonthDataPoint[] {
	let total = 0;

	return iterateDataMonthly<PerMonthDataPoint>(plugins, (d, year, month) => {
		const releasedPlugins = getAddedDataForMonth(plugins, year, month);

		total += releasedPlugins.length;

		return {
			year: year,
			month: month,
			value: total,
		};
	});
}

export function getPluginCountWoRetiredMonthly(plugins: PluginDataInterface[]): PerMonthDataPoint[] {
	let total = 0;

	return iterateDataMonthly<PerMonthDataPoint>(plugins, (d, year, month) => {
		const releasedPlugins = getAddedDataForMonth(plugins, year, month);
		const retiredPlugins = getRemovedDataForMonth(plugins, year, month);

		total += releasedPlugins.length - retiredPlugins.length;

		return {
			year: year,
			month: month,
			value: total,
		};
	});
}

export function getPluginCountRemovedMonthly(plugins: PluginDataInterface[]): PerMonthDataPoint[] {
	let total = 0;

	return iterateDataMonthly<PerMonthDataPoint>(plugins, (d, year, month) => {
		const retiredPlugins = getRemovedDataForMonth(plugins, year, month);

		total += retiredPlugins.length;

		return {
			year: year,
			month: month,
			value: total,
		};
	});
}

export function getPluginCountRemovedChangeMonthly(plugins: PluginDataInterface[]): PerMonthDataPoint[] {
	return iterateDataMonthly<PerMonthDataPoint>(plugins, (d, year, month) => {
		const retiredPlugins = getRemovedDataForMonth(plugins, year, month);

		return {
			year: year,
			month: month,
			value: retiredPlugins.length,
		};
	});
}
