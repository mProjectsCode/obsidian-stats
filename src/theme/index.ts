import { $, Verboseness } from '../shellUtils.ts';
import { Commit } from '../types.ts';
import { ThemeData, ThemeList } from './theme.ts';
import CliProgress from 'cli-progress';

import { encodeName, gitLogToCommits } from '../utils.ts';

import {
	THEME_LIST_PATH,
	THEME_DATA_PATH,
	OBSIDIAN_RELEASES_FULL_PATH,
	THEME_TEMPLATE_FILE_PATH,
	THEME_TEMPLATE_REPLACEMENT_STRING,
	THEME_TEMPLATE_REPLACEMENT_STRING_JSON,
	THEME_TEMPLATE_OUTPUT_PATH,
	PLUGIN_LIST_PATH,
	THEME_TEMPLATE_REPLACEMENT_STRING_NAME,
} from '../constants.ts';

async function getThemeListChanges(): Promise<Commit[]> {
	console.log(`Looking for changes to "${OBSIDIAN_RELEASES_FULL_PATH}/${THEME_LIST_PATH}"`);

	const themeListChanges = await $(
		`git log --diff-filter=M --date-order --reverse --format="%ad %H" --date=iso-strict "${THEME_LIST_PATH}"`,
		OBSIDIAN_RELEASES_FULL_PATH,
	);

	const commits = gitLogToCommits(themeListChanges.stdout);

	console.log(`Found ${commits.length} commits to "${THEME_LIST_PATH}"`);

	return commits;
}

async function getThemeLists(): Promise<ThemeList[]> {
	const themeListChangeCommits = await getThemeListChanges();

	const themeLists = (
		await Promise.all(
			themeListChangeCommits.map(async (x, i) => {
				const themeList = await $(`git cat-file -p ${x.hash}:${THEME_LIST_PATH}`, OBSIDIAN_RELEASES_FULL_PATH, Verboseness.QUITET);
				try {
					const themeListEntries = JSON.parse(themeList.stdout);
					return new ThemeList(themeListEntries, x);
				} catch (e) {
					console.warn(`Error parsing theme list at commit ${x.hash}`);
					return undefined;
				}
			}),
		)
	).filter(x => x !== undefined) as ThemeList[];

	console.log(`Found ${themeLists.length} version of "${THEME_LIST_PATH}"`);

	if (themeLists.length === 0) {
		throw new Error(`No theme lists found`);
	}

	return themeLists;
}

function buildThemeData(themeLists: ThemeList[]): ThemeData[] {
	let themeData: ThemeData[] = [];

	console.log(`Building theme data`);

	const progress = new CliProgress.SingleBar({}, CliProgress.Presets.rect);
	progress.start(themeLists.length, 0);

	progress.increment();
	for (const entry of themeLists[0].entries) {
		themeData.push(new ThemeData(entry.name, themeLists[0].commit, entry));
	}

	for (let i = 1; i < themeLists.length; i++) {
		progress.increment();

		const themeList = themeLists[i];

		for (const theme of themeData) {
			theme.findChanges(themeList);
		}

		for (const entry of themeList.entries) {
			if (themeData.find(x => x.name === entry.name) === undefined) {
				themeData.push(new ThemeData(entry.name, themeList.commit, entry));
			}
		}
	}

	progress.stop();

	return themeData;
}

export async function buildThemeStats() {
	const themeLists = await getThemeLists();
	let themeData = buildThemeData(themeLists);

	console.log(`Processed all themes writing to ${THEME_DATA_PATH}`);

	const pluginDataFile = Bun.file(THEME_DATA_PATH);
	await Bun.write(pluginDataFile, JSON.stringify(themeData, null, '\t'));

	// const templateFile = Bun.file(THEME_TEMPLATE_FILE_PATH);
	// const template = await templateFile.text();
	//
	// for (const theme of themeData) {
	// 	let output = template.replaceAll(THEME_TEMPLATE_REPLACEMENT_STRING, theme.name);
	// 	output = output.replaceAll(THEME_TEMPLATE_REPLACEMENT_STRING_NAME, encodeName(theme.currentEntry.name));
	// 	output = output.replaceAll(THEME_TEMPLATE_REPLACEMENT_STRING_JSON, JSON.stringify(theme));
	// 	const outputFile = Bun.file(`${THEME_TEMPLATE_OUTPUT_PATH}/${theme.id}.mdx`);
	// 	await Bun.write(outputFile, output);
	// }
}
