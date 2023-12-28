import { GithubReleaseEntry } from './release.ts';
import { RELEASE_FULL_DATA_PATH, RELEASE_STATS_URL, RELEASE_WEEKLY_DATA_PATH } from '../constants.ts';
import { dateToString, distributeValueEqually, getNextMondays } from '../utils.ts';
import { escape, from, fromCSV, op, table } from 'arquero';
import { fixVersion } from './data.ts';
import { Struct } from 'arquero/dist/types/op/op-api';
import ColumnTable from 'arquero/dist/types/table/column-table';

async function fetchReleaseStats(): Promise<ColumnTable> {
	const releases: GithubReleaseEntry[] = [];
	let currentPage: string | null = RELEASE_STATS_URL;
	while (currentPage) {
		// TODO: Investigate how authentication can be securely added here (for less risk of getting rate limited)
		const response = (await fetch(currentPage)) as Response;

		if (!response.ok) {
			throw new Error('Error while fetching releases data: ' + ((await response.json()) as any).message);
		}

		const releasesPage = (await response.json()) as GithubReleaseEntry[];

		releases.push(...releasesPage);
		let nextLink: string | null = null;
		const link = response.headers.get('link');
		if (link) {
			const nextLinkSearch = link.split(',').find(link => link.includes('rel="next"'));
			if (nextLinkSearch) {
				const nextLinkMatch = nextLinkSearch.match(/<(.+)>/);
				if (nextLinkMatch) nextLink = nextLinkMatch[1];
			}
		}

		currentPage = nextLink;
	}

	const releaseData = releases.flatMap(x =>
		x.assets.map(y => ({
			version: fixVersion(x.tag_name),
			date: new Date(x.published_at),
			asset: y.name,
			downloads: y.download_count,
			size: y.size,
		})),
	);

	return from(releaseData).orderby('version', 'asset', 'date');
}

function computeWeeklyDownloads(previousData: ColumnTable, currentData: ColumnTable, previousDate: Date, endDate: Date): ColumnTable {
	const incrementalData = previousData
		.join(currentData, [
			['version', 'asset'],
			['version', 'asset'],
		])
		// @ts-expect-error TS18048
		.derive({ downloads: d => d.downloads_2 - d.downloads_1 })
		.select('version', 'asset', 'downloads')
		// @ts-expect-error TS18048
		.filter(d => d.downloads > 0);

	const weeklyDates = getNextMondays(previousDate, endDate);
	const weeklyWeights = weeklyFactors(weeklyDates, previousDate, endDate);
	const stringDates = weeklyDates.map(x => dateToString(x));

	return (
		incrementalData
			.derive({ downloads: escape((d: Struct) => distributeValueEqually(d.downloads, weeklyWeights)) })
			.unroll('downloads', { index: 'date' })
			.derive({ date: escape((d: Struct) => stringDates[d.date]) })
			// @ts-expect-error TS18048
			.filter(d => d.downloads > 0)
	);
}

function combineWeeklyDownloads(weeklyData: ColumnTable, newData: ColumnTable): ColumnTable {
	return (
		from(weeklyData.objects().concat(newData.objects()))
			.groupby('date', 'version', 'asset')
			// @ts-expect-error TS18048
			.rollup({ downloads: d => op.sum(d.downloads) })
			.orderby('date', 'version', 'asset')
	);
}

/**
 * Determine how a value should be spread across an interval of weeks
 */
function weeklyFactors(dates: Date[], startDate: Date, endDate: Date): number[] {
	if (dates.length === 0) return [];
	if (dates.length === 1) return [1];

	const startWeekCover = (dates[0].getTime() - startDate.getTime()) / (7 * 86400000);
	const endWeekCover = 1 - (dates[dates.length - 1].getTime() - endDate.getTime()) / (7 * 86400000);

	const totalWeekCover = startWeekCover + dates.length - 2 + endWeekCover;
	return [startWeekCover / totalWeekCover, ...Array.from({ length: dates.length - 2 }, () => 1 / totalWeekCover), endWeekCover / totalWeekCover];
}

export async function testWeeklyDownloads(): Promise<void> {
	const previousFullDataFile = Bun.file('releases-prev-data.csv');
	const previousReleaseData = fromCSV(await previousFullDataFile.text()).select('version', 'asset', 'downloads');

	const currentFullDataFile = Bun.file('releases-full-data.csv');
	const currentReleaseData = fromCSV(await currentFullDataFile.text()).select('version', 'asset', 'downloads');

	const endDate = new Date();
	const previousDate = new Date(endDate.getTime() - 0 * 86400000);

	const incrementalData = computeWeeklyDownloads(previousReleaseData, currentReleaseData, previousDate, endDate);

	const weeklyDownloadsFile = Bun.file(RELEASE_WEEKLY_DATA_PATH);
	const weeklyDownloadsTable = fromCSV(await weeklyDownloadsFile.text(), { parse: { date: String } });
	const combinedWeeklyDownloadsTable = combineWeeklyDownloads(weeklyDownloadsTable, incrementalData);

	await Bun.write(Bun.file('res.csv'), combinedWeeklyDownloadsTable.toCSV());
	await Bun.write(Bun.file('incr.csv'), incrementalData.toCSV());
}

export async function buildReleaseStats(): Promise<void> {
	const releaseData = await fetchReleaseStats();
	// const releaseData = fromCSV(await Bun.file(RELEASE_FULL_DATA_PATH).text());

	const releaseFullDataFile = Bun.file(RELEASE_FULL_DATA_PATH);
	const releaseWeeklyDataFile = Bun.file(RELEASE_WEEKLY_DATA_PATH);

	const lastModifiedDate = new Date(releaseFullDataFile.lastModified);
	const currentDate = new Date();

	let weeklyData: ColumnTable;
	try {
		weeklyData = fromCSV(await releaseWeeklyDataFile.text(), { parse: { date: String } });
	} catch (e) {
		// If the CSV file is empty or doesn't exist, start from an empty table
		weeklyData = table([
			['date', []],
			['version', []],
			['asset', []],
			['downloads', []],
		]);
	}

	let previousReleaseData: ColumnTable;
	try {
		previousReleaseData = fromCSV(await releaseFullDataFile.text());
	} catch (e) {
		// If no previous data is given, start from an empty table
		previousReleaseData = table([
			['version', []],
			['date', []],
			['asset', []],
			['downloads', []],
			['size', []],
		]);
	}

	const incrementalData = computeWeeklyDownloads(previousReleaseData, releaseData, lastModifiedDate, currentDate);
	const combinedWeeklyDownloadsTable = combineWeeklyDownloads(weeklyData, incrementalData);

	await Bun.write(releaseWeeklyDataFile, combinedWeeklyDownloadsTable.toCSV());
	await Bun.write(releaseFullDataFile, releaseData.toCSV());
}

await buildReleaseStats();
// await testWeeklyDownloads();
