import { Commit, DownloadHistory, EntryChange, VersionHistory } from '../types.ts';
import { uniqueConcat } from '../utils.ts';
import { Version } from '../version.ts';

export interface PluginListEntry {
	id: string;
	name: string;
	author: string;
	description: string;
	repo: string;
}

export class PluginList {
	entries: PluginListEntry[];
	commit: Commit;

	constructor(entries: PluginListEntry[], commit: Commit) {
		this.entries = entries;
		this.commit = commit;
	}
}

export type PluginDownloadStatsEntry = {
	downloads: number;
} & Record<string, number>;

export interface PluginDataInterface {
	id: string;
	addedCommit: Commit;
	removedCommit?: Commit;
	initialEntry: PluginListEntry;
	currentEntry: PluginListEntry;
	changeHistory: EntryChange[];
	downloadHistory: DownloadHistory;
	versionHistory: VersionHistory[];
}

export class PluginDownloadStats {
	entries: Record<string, PluginDownloadStatsEntry>;
	commit: Commit;

	constructor(entries: Record<string, PluginDownloadStatsEntry>, commit: Commit) {
		this.entries = entries;
		this.commit = commit;
	}

	getDateString(): string {
		return this.commit.date;
	}
}

export class PluginData {
	id: string;
	addedCommit: Commit;
	removedCommit?: Commit;
	initialEntry: PluginListEntry;
	currentEntry: PluginListEntry;
	changeHistory: EntryChange[];
	downloadHistory: DownloadHistory;
	versionHistory: VersionHistory[];
	#versionHistoryMap: Map<string, VersionHistory>;

	constructor(id: string, addedCommit: Commit, initialEntry: PluginListEntry) {
		this.id = id;
		this.addedCommit = addedCommit;
		this.initialEntry = initialEntry;
		this.currentEntry = initialEntry;
		this.changeHistory = [
			{
				property: 'Plugin Added',
				commit: addedCommit,
				oldValue: '',
				newValue: '',
			},
		];
		this.versionHistory = [];
		this.downloadHistory = {};

		this.#versionHistoryMap = new Map();
	}

	addChange(change: EntryChange) {
		this.changeHistory.push(change);
	}

	findChanges(pluginList: PluginList) {
		const newEntry = pluginList.entries.find(x => x.id === this.id);
		if (newEntry === undefined) {
			if (this.removedCommit === undefined) {
				this.removedCommit = pluginList.commit;

				this.changeHistory.push({
					property: 'Plugin Removed',
					commit: pluginList.commit,
					oldValue: '',
					newValue: '',
				});
			}
			return;
		} else {
			const keys = uniqueConcat(Object.keys(this.currentEntry), Object.keys(newEntry));

			if (this.removedCommit !== undefined) {
				// plugin was removed and added again
				this.removedCommit = undefined;

				this.changeHistory.push({
					property: 'Plugin Readded',
					commit: pluginList.commit,
					oldValue: '',
					newValue: '',
				});
			}

			for (const key of keys) {
				// @ts-expect-error TS7053
				const oldValue = this.currentEntry[key];
				// @ts-expect-error TS7053
				const newValue = newEntry[key];

				if (oldValue !== newValue) {
					this.changeHistory.push({
						property: key,
						commit: pluginList.commit,
						oldValue,
						newValue,
					} satisfies EntryChange);
				}
			}

			// const changes = keys
			// 	.map(key => {
			// 		// @ts-expect-error TS7053
			// 		const oldValue = this.currentEntry[key];
			// 		// @ts-expect-error TS7053
			// 		const newValue = newEntry[key];

			// 		if (oldValue !== newValue) {
			// 			return {
			// 				property: key,
			// 				commit: pluginList.commit,
			// 				oldValue,
			// 				newValue,
			// 			} satisfies EntryChange;
			// 		}
			// 	})
			// 	.filter(x => x !== undefined) as EntryChange[];
			// this.changeHistory.push(...changes);
			this.currentEntry = newEntry;
		}
	}

	updateDownloadHistory(pluginDownloadStats: PluginDownloadStats, date: string): boolean {
		const entry = pluginDownloadStats.entries[this.id];
		if (entry === undefined) {
			return false;
		}

		this.downloadHistory[date] = entry.downloads;
		return true;
	}

	updateVersionHistory(pluginDownloadStats: PluginDownloadStats) {
		const entry = pluginDownloadStats.entries[this.id];
		if (entry === undefined) {
			return;
		}

		for (const version of Object.keys(entry)) {
			if (version === 'downloads' || version === 'latest' || version === 'updated') {
				continue;
			}

			if (!Version.valid(version)) {
				continue;
			}

			if (!this.#versionHistoryMap.has(version)) {
				this.#versionHistoryMap.set(version, {
					version: version,
					initialReleaseDate: pluginDownloadStats.getDateString(),
				});
			}
		}
	}

	sortVersionHistory() {
		this.versionHistory = Array.from(this.#versionHistoryMap.values());
		this.versionHistory.sort((a, b) => (Version.lessThan(Version.fromString(a.version), Version.fromString(b.version)) ? -1 : 1));
	}

	getDownloadCount(): number {
		const values = Object.values(this.downloadHistory);
		return Math.max(...values, 0);
	}
}
