import { $, Verboseness } from '../shellUtils.ts';
import { PluginData, PluginDownloadStats, PluginList } from './plugin.ts';
import CliProgress from 'cli-progress';
import { dateToString, dateDiffInDays, gitLogToCommits } from '../utils.ts';
import { Commit } from '../types.ts';

import {
	PLUGIN_LIST_PATH,
	PLUGIN_STATS_PATH,
	PLUGIN_DATA_PATH,
	OBSIDIAN_RELEASES_FULL_PATH,
	PLUGIN_TEMPLATE_FILE_PATH,
	PLUGIN_TEMPLATE_REPLACEMENT_STRING,
	PLUGIN_TEMPLATE_REPLACEMENT_STRING_JSON,
	PLUGIN_TEMPLATE_OUTPUT_PATH,
} from '../constants.ts';

async function getPluginListChanges(): Promise<Commit[]> {
	console.log(`Looking for changes to "${OBSIDIAN_RELEASES_FULL_PATH}/${PLUGIN_LIST_PATH}"`);

	const pluginListChanges = await $(
		`git log --diff-filter=M --date-order --reverse --format="%ad %H" --date=iso-strict "${PLUGIN_LIST_PATH}"`,
		OBSIDIAN_RELEASES_FULL_PATH,
	);

	const commits = gitLogToCommits(pluginListChanges.stdout);

	console.log(`Found ${commits.length} commits to "${PLUGIN_LIST_PATH}"`);

	return commits;
}

async function getPluginLists(): Promise<PluginList[]> {
	const pluginListChangeCommits = await getPluginListChanges();

	const pluginLists = (
		await Promise.all(
			pluginListChangeCommits.map(async (x, i) => {
				const pluginList = await $(`git cat-file -p ${x.hash}:${PLUGIN_LIST_PATH}`, OBSIDIAN_RELEASES_FULL_PATH, Verboseness.QUITET);
				try {
					const pluginListEntries = JSON.parse(pluginList.stdout);
					return new PluginList(pluginListEntries, x);
				} catch (e) {
					console.warn(`Error parsing plugin list at commit ${x.hash}`);
					return undefined;
				}
			}),
		)
	).filter(x => x !== undefined) as PluginList[];

	console.log(`Found ${pluginLists.length} version of "${PLUGIN_LIST_PATH}"`);

	if (pluginLists.length === 0) {
		throw new Error(`No plugin lists found`);
	}

	return pluginLists;
}

async function getPluginDownloadChanges(): Promise<Commit[]> {
	const pluginDownloadChanges = await $(
		`git log --diff-filter=M --date-order --reverse --format="%ad %H" --date=iso-strict "${PLUGIN_STATS_PATH}"`,
		OBSIDIAN_RELEASES_FULL_PATH,
	);

	return gitLogToCommits(pluginDownloadChanges.stdout);
}

function buildPluginData(pluginLists: PluginList[]): PluginData[] {
	let pluginDataMap: Map<string, PluginData> = new Map();

	console.log(`Building plugin data`);

	const progress = new CliProgress.SingleBar({}, CliProgress.Presets.rect);
	progress.start(pluginLists.length, 0);

	progress.increment();
	for (const entry of pluginLists[0].entries) {
		pluginDataMap.set(entry.id, new PluginData(entry.id, pluginLists[0].commit, entry));
	}

	for (let i = 1; i < pluginLists.length; i++) {
		progress.increment();

		const pluginList = pluginLists[i];

		for (const plugin of pluginDataMap.values()) {
			plugin.findChanges(pluginList);
		}

		for (const entry of pluginList.entries) {
			if (!pluginDataMap.has(entry.id)) {
				pluginDataMap.set(entry.id, new PluginData(entry.id, pluginList.commit, entry));
			}
		}
	}

	progress.stop();

	return [...pluginDataMap.values()];
}

function updateWeeklyDownloadStats(pluginData: PluginData[], pluginDownloadStats: PluginDownloadStats[]) {
	console.log(`Updating weekly download stats`);

	const downloadStatsMap = new Map<string, PluginDownloadStats>();
	for (const pluginDownload of pluginDownloadStats) {
		downloadStatsMap.set(pluginDownload.getDateString(), pluginDownload);
	}

	const startDate = new Date('2020-01-01');
	const endDate = new Date();

	startDate.setDate(startDate.getDate() + (7 - startDate.getDay()));

	const progress = new CliProgress.SingleBar({}, CliProgress.Presets.rect);
	progress.start(Math.ceil(dateDiffInDays(startDate, endDate) / 7), 0);

	for (let d = startDate; d <= endDate; d.setDate(d.getDate() + 7)) {
		const date = dateToString(d);
		progress.increment();

		for (const pluginDataEntry of pluginData) {
			for (let j = 0; j < 6; j++) {
				const currentDate = new Date(d);
				currentDate.setDate(currentDate.getDate() + j);
				const currentDateString = dateToString(currentDate);

				const pluginDownload = downloadStatsMap.get(currentDateString);

				if (pluginDownload !== undefined && pluginDataEntry.updateDownloadHistory(pluginDownload, date)) {
					break;
				}
			}
		}
	}

	progress.stop();
}

function updateVersionHistory(pluginData: PluginData[], pluginDownloadStats: PluginDownloadStats[]): void {
	console.log(`Updating version history`);

	const progress = new CliProgress.SingleBar({}, CliProgress.Presets.rect);
	progress.start(pluginDownloadStats.length, 0);

	for (const pluginDownload of pluginDownloadStats) {
		progress.increment();
		for (const pluginDataEntry of pluginData) {
			pluginDataEntry.updateVersionHistory(pluginDownload);
		}
	}

	progress.stop();

	console.log(`Sorting Versions`);
	for (const pluginDataEntry of pluginData) {
		pluginDataEntry.sortVersionHistory();
	}
}

async function getPluginDownloadStats(): Promise<PluginDownloadStats[]> {
	const pluginDownloadChangeCommits = await getPluginDownloadChanges();

	const pluginDownloadStats = (
		await Promise.all(
			pluginDownloadChangeCommits.map(async (x, i) => {
				const pluginList = await $(`git cat-file -p ${x.hash}:${PLUGIN_STATS_PATH}`, OBSIDIAN_RELEASES_FULL_PATH, Verboseness.QUITET);
				try {
					const pluginDownloadStats = JSON.parse(pluginList.stdout);
					return new PluginDownloadStats(pluginDownloadStats, x);
				} catch (e) {
					console.log(`Error parsing plugin list at commit ${x.hash}`);
					return undefined;
				}
			}),
		)
	).filter(x => x !== undefined) as PluginDownloadStats[];

	console.log(`Found ${pluginDownloadStats.length} versions of "community-plugin-stats.json"`);

	return pluginDownloadStats;
}

export async function buildPluginStats(): Promise<void> {
	const pluginLists = await getPluginLists();
	let pluginData = buildPluginData(pluginLists);

	const pluginDownloadStats = await getPluginDownloadStats();
	updateWeeklyDownloadStats(pluginData, pluginDownloadStats);
	updateVersionHistory(pluginData, pluginDownloadStats);

	pluginData = pluginData.filter(x => x !== undefined);

	console.log(`Processed all plugins, writing to ${PLUGIN_DATA_PATH}`);

	const pluginDataFile = Bun.file(PLUGIN_DATA_PATH);
	await Bun.write(pluginDataFile, JSON.stringify(pluginData, null, 4));

	const templateFile = Bun.file(PLUGIN_TEMPLATE_FILE_PATH);
	const template = await templateFile.text();

	for (const plugin of pluginData) {
		let output = template.replaceAll(PLUGIN_TEMPLATE_REPLACEMENT_STRING, plugin.id);
		output = output.replaceAll(PLUGIN_TEMPLATE_REPLACEMENT_STRING_JSON, JSON.stringify(plugin));
		const outputFile = Bun.file(`${PLUGIN_TEMPLATE_OUTPUT_PATH}/${plugin.id}.mdx`);
		await Bun.write(outputFile, output);
	}
}
