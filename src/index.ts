import {$, Verboseness} from './shellUtils.ts';
import process from 'process';
import { Commit, PluginData, PluginDownloadStats, PluginList,  } from './types.ts';
import {dateToString} from './utils.ts';

const PLUGIN_LIST_PATH = 'community-plugins.json';
const PLUGIN_STATS_PATH = 'community-plugin-stats.json';
const OBSIDIAN_RELEASES_PATH = 'obsidian-releases';
const OBSIDIAN_RELEASES_FULL_PATH = `${process.cwd()}/${OBSIDIAN_RELEASES_PATH}`;
const PLUGIN_DATA_PATH = `plugin-data.json`;
const TEMPLATE_FILE_PATH = 'src/template.txt';
const TEMPLATE_REPLACEMENT_STRING = 'PLUGIN_ID';
const TEMPLATE_OUTPUT_PATH = 'website/src/content/docs/plugins';

function dateDiffInDays(a: Date, b: Date): number {
	const _MS_PER_DAY = 1000 * 60 * 60 * 24;
	// Discard the time and time-zone information.
	const utc1 = Date.UTC(a.getFullYear(), a.getMonth(), a.getDate());
	const utc2 = Date.UTC(b.getFullYear(), b.getMonth(), b.getDate());

	return Math.floor((utc2 - utc1) / _MS_PER_DAY);
}

function gitLogToCommits(log: string): Commit[] {
	return log
		.split('\n')
		.filter(x => x.trim() !== '')
		.map(x => x.replaceAll('\"', ''))
		.map(x => x.split(' '))
		.map(x => ({date: x[0], hash: x[1]}));

}

async function getPluginListChanges(): Promise<Commit[]> {
	const pluginListChanges = await $(`git log --diff-filter=M --date-order --reverse --format="%ad %H" --date=iso-strict "${PLUGIN_LIST_PATH}"`, OBSIDIAN_RELEASES_FULL_PATH)

	return gitLogToCommits(pluginListChanges.stdout);
}

async function getPluginDownloadChanges(): Promise<Commit[]> {
	const pluginDownloadChanges = await $(`git log --diff-filter=M --date-order --reverse --format="%ad %H" --date=iso-strict "${PLUGIN_STATS_PATH}"`, OBSIDIAN_RELEASES_FULL_PATH)

	return gitLogToCommits(pluginDownloadChanges.stdout);
}

async function buildPluginStats() {
	// update submodule
	await $('git submodule update --remote')

	const pluginListChangeCommits = await getPluginListChanges();

	console.log(`Found ${pluginListChangeCommits.length} plugin list changes`);

	console.log(`Fetching plugin lists`);

	const pluginLists = (await Promise.all(pluginListChangeCommits.map(async (x, i) => {
		const pluginList = await $(`git cat-file -p ${x.hash}:${PLUGIN_LIST_PATH}`, OBSIDIAN_RELEASES_FULL_PATH, Verboseness.QUITET);
		try {
			const pluginListEntries = JSON.parse(pluginList.stdout);
			return new PluginList(pluginListEntries, x);
		} catch (e) {
			console.log(`Error parsing plugin list at commit ${x.hash}`);
			return undefined;
		}
	}))).filter(x => x !== undefined) as PluginList[];

	console.log(`Found ${pluginLists.length} plugin lists`);

	const pluginDownloadChangeCommits = await getPluginDownloadChanges();

	console.log(`Found ${pluginDownloadChangeCommits.length} plugin download changes`);

	const pluginDownloads = (await Promise.all(pluginDownloadChangeCommits.map(async (x, i) => {
		const pluginList = await $(`git cat-file -p ${x.hash}:${PLUGIN_STATS_PATH}`, OBSIDIAN_RELEASES_FULL_PATH, Verboseness.QUITET);
		try {
			const pluginDownloadStats = JSON.parse(pluginList.stdout);
			return new PluginDownloadStats(pluginDownloadStats, x);
		} catch (e) {
			console.log(`Error parsing plugin list at commit ${x.hash}`);
			return undefined;
		}
	}))).filter(x => x !== undefined) as PluginDownloadStats[];

	console.log(`Found ${pluginDownloads.length} plugin download stats`);

	const pluginData: PluginData[] = [];

	console.log(`Processing plugin list 1 of ${pluginLists.length}`);

	for (const entry of pluginLists[0].entries) {
		pluginData.push(new PluginData(entry.id, pluginLists[0].commit, entry));
	}

	for (let i = 1; i < pluginLists.length; i++) {
		console.log(`Processing plugin list ${i + 1} of ${pluginLists.length}`);

		const pluginList = pluginLists[i];

		for (const plugin of pluginData) {
			plugin.findChanges(pluginList);
		}

		for (const entry of pluginList.entries) {
			if (pluginData.find(x => x.id === entry.id) === undefined) {
				pluginData.push(new PluginData(entry.id, pluginList.commit, entry));
			}
		}
	}

	const startDate = new Date('2020-01-01');
	const endDate = new Date();

	startDate.setDate(startDate.getDate() + (7 - startDate.getDay()));

	for (let d = startDate; d <= endDate; d.setDate(d.getDate() + 7)) {
		const date = dateToString(d);
		console.log(`Processing downloads for week ${date}`);

		let pluginDownload;

		for (let j = 0; j < 6; j++) {
			const currentDate = new Date(d);
			currentDate.setDate(currentDate.getDate() + j);
			const currentDateString = dateToString(currentDate);

			pluginDownload = pluginDownloads.find(x => dateToString(x.date) === currentDateString);
			if (pluginDownload !== undefined) break;
		}

		if (pluginDownload === undefined) {
			console.log(`No plugin download stats found for ${date}`);
			continue;
		}

		for (const pluginDataEntry of pluginData) {
			pluginDataEntry.updateDownloadHistory(pluginDownload, date);
		}
	}

	let i = 0;
	for (const pluginDownload of pluginDownloads) {
		console.log(`Processing plugin versions for commit ${i + 1} of ${pluginDownloads.length}`);
		for (const pluginDataEntry of pluginData) {
			pluginDataEntry.updateVersionHistory(pluginDownload);
		}
		
		i++;
	}

	console.log(`Processed all plugins, writing to ${PLUGIN_DATA_PATH}`);

	const pluginDataFile = Bun.file(PLUGIN_DATA_PATH);

	await Bun.write(pluginDataFile, JSON.stringify(pluginData, null, 4));

	const templateFile = Bun.file(TEMPLATE_FILE_PATH);

	const template = await templateFile.text();

	for (const plugin of pluginData) {
		const output = template.replaceAll(TEMPLATE_REPLACEMENT_STRING, plugin.id);
		const outputFile = Bun.file(`${TEMPLATE_OUTPUT_PATH}/${plugin.id}.mdx`);
		await Bun.write(outputFile, output);
	}


}

await buildPluginStats();