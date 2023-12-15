import {$, Verboseness} from './shellUtils.ts';
import process from 'process';

const PLUGIN_LIST_PATH = 'community-plugins.json';
const PLUGIN_STATS_PATH = 'community-plugin-stats.json';
const OBSIDIAN_RELEASES_PATH = 'obsidian-releases';
const OBSIDIAN_RELEASES_FULL_PATH = `${process.cwd()}/${OBSIDIAN_RELEASES_PATH}`;
const PLUGIN_DATA_PATH = `plugin-data.json`;
const TEMPLATE_FILE_PATH = 'src/template.txt';
const TEMPLATE_REPLACEMENT_STRING = 'PLUGIN_ID';
const TEMPLATE_OUTPUT_PATH = 'website/src/content/docs/plugins';



export interface Commit {
	date: string;
	hash: string;
}

export interface PluginListEntry {
	id: string;
	name: string;
	author: string;
	description: string;
	repo: string;
}

class PluginList {
	entries: PluginListEntry[];
	commit: Commit;

	constructor(entries: PluginListEntry[], commit: Commit) {
		this.entries = entries;
		this.commit = commit;
	}
}

type PluginDownloadStatsEntry = {
	downloads: number
} & Record<string, number>

class PluginDownloadStats {
	entries: Record<string, PluginDownloadStatsEntry>;
	commit: Commit;
	date: Date;

	constructor(entries: Record<string, PluginDownloadStatsEntry>, commit: Commit) {
		this.entries = entries;
		this.commit = commit;
		this.date = new Date(commit.date);
	}

	getDateString(): string {
		return `${this.date.getFullYear()}-${this.date.getMonth()}-${this.date.getDate()}`;
	}
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

export class PluginData {
	id: string;
	addedCommit: Commit;
	removedCommit?: Commit;
	initialEntry: PluginListEntry;
	currentEntry: PluginListEntry;
	changeHistory: EntryChange[];
	downloadHistory: DownloadHistory | undefined
	versionHistory: VersionHistory[];

	constructor(id: string, addedCommit: Commit, initialEntry: PluginListEntry) {
		this.id = id;
		this.addedCommit = addedCommit;
		this.initialEntry = initialEntry;
		this.currentEntry = initialEntry;
		this.changeHistory = [];
		this.versionHistory = [];
	}

	addChange(change: EntryChange) {
		this.changeHistory.push(change);
	}

	findChanges(pluginList: PluginList) {
		const newEntry = pluginList.entries.find(x => x.id === this.id);
		if (newEntry === undefined) {
			this.removedCommit = pluginList.commit;
			return;
		} else {
			const keys = new Set(Object.keys(this.currentEntry));
			Object.keys(newEntry).forEach(x => keys.add(x));

			const changes = [...keys].map(key => {
				// @ts-expect-error TS7053
				const oldValue = this.currentEntry[key];
				// @ts-expect-error TS7053
				const newValue = newEntry[key];

				if (oldValue !== newValue) {
					return {
						property: key,
						commit: pluginList.commit,
						oldValue,
						newValue,
					} satisfies EntryChange;
				}
			}).filter(x => x !== undefined) as EntryChange[];
			this.changeHistory.push(...changes);
			this.currentEntry = newEntry;
		}
	}

	updateDownloadHistory(pluginDownloadStats: PluginDownloadStats) {
		const entry = Object.entries(pluginDownloadStats.entries).find(x => x[0] === this.id)?.[1];
		if (entry === undefined) {
			return;
		}

		if (this.downloadHistory === undefined) {
			this.downloadHistory = {
				[pluginDownloadStats.getDateString()]: entry.downloads,
			};
		} else {
			if (pluginDownloadStats.date.getDay() === 0) {
				this.downloadHistory[pluginDownloadStats.getDateString()] = entry.downloads;
			}
		}

		for (const version of Object.keys(entry)) {
			if (version === 'downloads' || version === 'latest' || version === 'updated') {
				continue;
			}

			const versionHistory = this.versionHistory.find(x => x.version === version);
			if (versionHistory === undefined) {
				this.versionHistory.push({
					version,
					initialReleaseDate: pluginDownloadStats.getDateString(),
				});
			}
		}
	}
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

	let i = 0;
	for (const pluginDataEntry of pluginData) {
		console.log(`Processing downloads for plugin ${i + 1} of ${pluginData.length}`);

		for (const pluginDownload of pluginDownloads) {
			pluginDataEntry.updateDownloadHistory(pluginDownload);
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