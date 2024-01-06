import { Commit, EntryChange } from '../types.ts';
import slug from 'slug';
import { uniqueConcat } from '../utils.ts';

export interface ThemeListEntry {
	name: string;
	author: string;
	repo: string;
	screenshot: string;
	modes: ['dark'] | ['light'] | ['dark', 'light'] | ['light', 'dark'];
	legacy?: boolean;
}

export class ThemeList {
	entries: ThemeListEntry[];
	commit: Commit;

	constructor(entries: ThemeListEntry[], commit: Commit) {
		this.entries = entries;
		this.commit = commit;
	}
}

export interface ThemeDataInterface {
	id: string;
	name: string;
	addedCommit: Commit;
	removedCommit?: Commit;
	initialEntry: ThemeListEntry;
	currentEntry: ThemeListEntry;
	changeHistory: EntryChange[];
}

let themeIdCounter: Record<string, number> = {};

function themeNameToId(name: string): string {
	let id = slug(name);
	if (themeIdCounter[id] === undefined) {
		themeIdCounter[id] = 1;
	} else {
		id += `-${themeIdCounter[id]}`;
		themeIdCounter[id] += 1;
	}

	return id;
}

export class ThemeData {
	id: string;
	name: string;
	addedCommit: Commit;
	removedCommit?: Commit;
	initialEntry: ThemeListEntry;
	currentEntry: ThemeListEntry;
	changeHistory: EntryChange[];

	constructor(name: string, addedCommit: Commit, initialEntry: ThemeListEntry) {
		this.name = name;
		this.id = themeNameToId(name);
		this.addedCommit = addedCommit;
		this.initialEntry = initialEntry;
		this.currentEntry = initialEntry;
		this.changeHistory = [
			{
				property: 'Theme Added',
				commit: addedCommit,
				oldValue: '',
				newValue: '',
			},
		];
	}

	findChanges(themeList: ThemeList) {
		const newEntry = themeList.entries.find(x => x.name === this.name);
		if (newEntry === undefined) {
			if (this.removedCommit === undefined) {
				this.removedCommit = themeList.commit;

				this.changeHistory.push({
					property: 'Theme Removed',
					commit: themeList.commit,
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
					property: 'Theme Readded',
					commit: themeList.commit,
					oldValue: '',
					newValue: '',
				});
			}

			const changes = keys
				.map(key => {
					// @ts-expect-error TS7053
					const oldValue = this.currentEntry[key];
					// @ts-expect-error TS7053
					const newValue = newEntry[key];

					if (Array.isArray(oldValue) && Array.isArray(newValue)) {
						if (oldValue.length !== newValue.length) {
							return {
								property: key,
								commit: themeList.commit,
								oldValue: oldValue.join(', '),
								newValue: newValue.join(', '),
							} satisfies EntryChange;
						}

						for (let i = 0; i < oldValue.length; i++) {
							if (oldValue[i] !== newValue[i]) {
								return {
									property: key,
									commit: themeList.commit,
									oldValue: oldValue.join(', '),
									newValue: newValue.join(', '),
								} satisfies EntryChange;
							}
						}

						return undefined;
					}

					if (oldValue !== newValue) {
						return {
							property: key,
							commit: themeList.commit,
							oldValue,
							newValue,
						} satisfies EntryChange;
					}
				})
				.filter(x => x !== undefined) as EntryChange[];
			this.changeHistory.push(...changes);
			this.currentEntry = newEntry;
		}
	}
}
