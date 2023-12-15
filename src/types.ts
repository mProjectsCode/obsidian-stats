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

export class PluginList {
	entries: PluginListEntry[];
	commit: Commit;

	constructor(entries: PluginListEntry[], commit: Commit) {
		this.entries = entries;
		this.commit = commit;
	}
}

export type PluginDownloadStatsEntry = {
	downloads: number
} & Record<string, number>

export class PluginDownloadStats {
	entries: Record<string, PluginDownloadStatsEntry>;
	commit: Commit;
	date: Date;

	constructor(entries: Record<string, PluginDownloadStatsEntry>, commit: Commit) {
		this.entries = entries;
		this.commit = commit;
		this.date = new Date(commit.date);
	}

	getDateString(): string {
		return `${this.date.getFullYear()}-${(this.date.getMonth() + 1).toString().padStart(2, '0')}-${this.date.getDate().toString().padStart(2, '0')}`;
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
	downloadHistory: DownloadHistory;
	versionHistory: VersionHistory[];

	constructor(id: string, addedCommit: Commit, initialEntry: PluginListEntry) {
		this.id = id;
		this.addedCommit = addedCommit;
		this.initialEntry = initialEntry;
		this.currentEntry = initialEntry;
		this.changeHistory = [];
		this.versionHistory = [];
        this.downloadHistory = {};
	}

	addChange(change: EntryChange) {
		this.changeHistory.push(change);
	}

	findChanges(pluginList: PluginList) {
		const newEntry = pluginList.entries.find(x => x.id === this.id);
		if (newEntry === undefined) {
			if (this.removedCommit === undefined) {
				this.removedCommit = pluginList.commit;
			}
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

	updateDownloadHistory(pluginDownloadStats: PluginDownloadStats, date: string) {
		const entry = Object.entries(pluginDownloadStats.entries).find(x => x[0] === this.id)?.[1];
		if (entry === undefined) {
			return;
		}

		this.downloadHistory[date] = entry.downloads;
	}

	updateVersionHistory(pluginDownloadStats: PluginDownloadStats) {
		const entry = Object.entries(pluginDownloadStats.entries).find(x => x[0] === this.id)?.[1];
		if (entry === undefined) {
			return;
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