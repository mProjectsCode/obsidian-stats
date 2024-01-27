import {PLUGIN_DATA_PATH} from '../constants.ts';
import {PluginDataInterface} from '../plugin/plugin.ts';
import fs from 'node:fs/promises';
import {$} from '../shellUtils.ts';

export async function clonePluginRepos() {
	const pluginData = await Bun.file(PLUGIN_DATA_PATH).json() as PluginDataInterface[];

	await fs.rm('pluginRepos/repos', {recursive: true, force: true});
	await fs.mkdir('pluginRepos/repos', {recursive: true});

	const skippedPlugins = [];
	const failedPlugins = [];

	for (const plugin of pluginData) {
		if (plugin.removedCommit) {
			skippedPlugins.push(plugin.id);
			continue;
		}

		const cloneRes = await $(`git clone https://github.com/${plugin.currentEntry.repo}.git pluginRepos/repos/${plugin.id} --depth 1`);
		if (cloneRes.stderr) {
			failedPlugins.push(plugin.id);
		}
	}

	console.log('Finished cloning repos');

	console.log('Skipped plugins:');
	for (const plugin of skippedPlugins) {
		console.log(plugin);

	}
	console.log('Failed plugins:');
	for (const plugin of failedPlugins) {
		console.log(plugin);
	}
}

export async function collectRepoData() {
	const pluginData = await Bun.file(PLUGIN_DATA_PATH).json() as PluginDataInterface[];

	for (const plugin of pluginData) {
		if (plugin.removedCommit) {
			continue;
		}

		const repoPath = `pluginRepos/repos/${plugin.id}`;
		if (!await fs.exists(repoPath)) {
			console.log(`Repo for plugin ${plugin.id} does not exist`);
			continue;
		}

		const files = await listFiles(repoPath);

		const excludedExtensions = ["LICENSE"]
		let fileCounts: Record<string, number> = {};
		let hasTestFiles = false;

		for (const file of files) {
			if (file.endsWith('.test.ts') || file.endsWith('.test.js')) {
				hasTestFiles = true;
			}

			const extension = file.split('/').at(-1)?.split('.').at(-1);
			if (!extension || excludedExtensions.includes(extension)) {
				continue;
			}

			if (!fileCounts[extension]) {
				fileCounts[extension] = 0;
			}

			fileCounts[extension]++;
		}

		console.log(plugin.id, files, fileCounts, hasTestFiles);
	}
}

async function listFiles(dir: string, pathDir: string = ''): Promise<string[]> {
	const files = await fs.readdir(dir, { withFileTypes: true });

	const filteredFiles = await Promise.all(files.map(async (file) => {
		const niceName = pathDir ? `${pathDir}/${file.name}` : file.name;

		if (file.isDirectory()) {
			if (file.name === '.git') {
				return [];
			}

			return await listFiles(`${dir}/${file.name}`, niceName);
		} else {
			return [niceName];
		}
	}));

	return filteredFiles.flat();
}