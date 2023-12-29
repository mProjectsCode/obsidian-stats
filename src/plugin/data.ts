import { type PluginDataInterface } from './plugin.ts';
import { type DownloadDataPoint, type DownloadReleaseCorrelationDataPoint, type PerMonthDataPoint } from '../types.ts';
import { dateToString, filterRemoved, getAddedDataForMonth, getRemovedDataForMonth, iterateDataMonthly, iterateWeekly } from '../utils.ts';
import { reduce } from 'itertools-ts';

export function getPluginDownloadsWeekly(plugins: PluginDataInterface[]): DownloadDataPoint[] {
	const data: DownloadDataPoint[] = [];

	for (const plugin of plugins) {
		const downloadHistoryData = Object.entries(plugin.downloadHistory);
		if (downloadHistoryData.length === 0) continue;

		const startDateString = reduce.toMin(downloadHistoryData, d => d[0])![0];
		const endDateString = reduce.toMax(downloadHistoryData, d => d[0])![0];

		const startDate = new Date(startDateString);
		const endDate = new Date(endDateString);

		let lastDownloadCount: number | undefined = 0;

		data.push(
			...(iterateWeekly<DownloadDataPoint | undefined>(startDate, endDate, (d, date) => {
				const downloadCount: number | undefined = downloadHistoryData.find(d => d[0] === date)?.[1];
				// calculate growth
				let growth: number | undefined = undefined;
				if (downloadCount !== undefined && lastDownloadCount !== undefined) {
					growth = downloadCount - lastDownloadCount;
				}
				// update last download count
				lastDownloadCount = downloadCount;

				const existingData = data.find(x => x.date === date);

				if (existingData !== undefined) {
					// update existing data
					if (downloadCount !== undefined) {
						if (existingData.downloads === undefined) {
							existingData.downloads = downloadCount;
						} else {
							existingData.downloads += downloadCount;
						}
					}

					if (growth !== undefined) {
						if (existingData.growth === undefined) {
							existingData.growth = growth;
						} else {
							existingData.growth += growth;
						}
					}

					return undefined;
				} else {
					return {
						date: date,
						downloads: downloadCount,
						growth: growth,
					};
				}
			}).filter(x => x !== undefined) as DownloadDataPoint[]),
		);
	}

	return data.sort((a, b) => a.date.localeCompare(b.date));
}

export function getDownloadReleaseCorrelationDataPoints(plugins: PluginDataInterface[]): DownloadReleaseCorrelationDataPoint[] {
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

export function getPluginRemovedList(plugins: PluginDataInterface[]): PluginDataInterface[] {
	return filterRemoved(plugins).sort((a, b) => a.id.localeCompare(b.id));
}

export function getPluginRemovedRecentList(plugins: PluginDataInterface[]): PluginDataInterface[] {
	return filterRemoved(plugins)
		.sort((a, b) => new Date(b.removedCommit!.date).valueOf() - new Date(a.removedCommit!.date).valueOf())
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
		};
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
