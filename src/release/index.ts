import type { GithubReleaseEntry, ObsidianReleaseInfo } from './release.ts';
import {
	RELEASE_CHANGELOG_PATH,
	RELEASE_FULL_DATA_PATH,
	RELEASE_INFO_URL,
	RELEASE_STATS_URL,
	RELEASE_WEEKLY_DATA_PATH
} from '../constants.ts';
import { distributeValueEqually, getNextMondays } from '../utils.ts';
import { escape, from, fromCSV, op, table } from 'arquero';
import { XMLParser } from "fast-xml-parser";
import { Struct } from 'arquero/dist/types/op/op-api';
import ColumnTable from 'arquero/dist/types/table/column-table';
import { Version } from '../version.ts';
import { CDate } from '../date.ts';

async function fetchGithubStats(): Promise<ColumnTable> {
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
			version: Version.alphabetic(x.tag_name),
			date: new Date(x.published_at),
			asset: y.name,
			downloads: y.download_count,
			size: y.size,
		})),
	);

	return from(releaseData).orderby('version', 'asset', 'date');
}


async function fetchChangelogStats(): Promise<ColumnTable> {
	let currentPage: string | null = RELEASE_INFO_URL;

	const response = await fetch(currentPage);
	const parser = new XMLParser();
	const xml = parser.parse(await response.text());
	const entries: ObsidianReleaseInfo[] = xml.feed.entry.map((entry: any) => {
		const release_info = entry.id.slice(30);
		let version = release_info.match(/v\d+\.\d+(\.\d+)?/)?.[0] ?? '';
		let major_release = false;
		if (version.length && version.split('.').length !== 0) {
			// Major releases do not conform to semantic versioning as you can have a major release which has
			//   the same version number as the previous insider release (e.g. 1.7.4 insider and '1.7' (also 1.7.4))
			major_release = /(?<!\d\.)\b\d+\.\d+\b(?!\.\d)/.test(entry.title);
			version = Version.alphabetic(version);
		}
		return {
			version: version,
			platform: release_info.match(/desktop|mobile|publish/)?.[0] as "desktop" | "mobile" | "publish",
			insider: entry.title.includes('Early access'),
			date: new Date(entry.updated),
			info: entry.content,
			major_release
		}
	});

	return from(entries).orderby('version', 'date');
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
	const stringDates = weeklyDates.map(x => CDate.fromDate(x).toString());

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

async function updateChangelogData() {
	const changelogData = await fetchChangelogStats();
	await Bun.write(RELEASE_CHANGELOG_PATH, changelogData.toCSV());
}

async function updateGithubData() {
	const githubData = await fetchGithubStats();
	await Bun.write(RELEASE_FULL_DATA_PATH, githubData.toCSV());

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

	const incrementalData = computeWeeklyDownloads(previousReleaseData, githubData, lastModifiedDate, currentDate);
	const combinedWeeklyDownloadsTable = combineWeeklyDownloads(weeklyData, incrementalData);

	await Bun.write(releaseWeeklyDataFile, combinedWeeklyDownloadsTable.toCSV());
	await Bun.write(releaseFullDataFile, githubData.toCSV());
}

export async function buildReleaseStats(): Promise<void> {
	await updateGithubData();
	await updateChangelogData();
}

// await updateGithubData();
// await updateChangelogData();
