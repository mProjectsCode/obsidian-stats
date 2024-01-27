import {PLUGIN_DATA_PATH} from '../constants.ts';
import {PluginDataInterface} from '../plugin/plugin.ts';
import fs from 'node:fs/promises';
import {$} from '../shellUtils.ts';
import {arrayIntersect, uniqueConcat} from '../utils.ts';
import {PluginRepoData} from './types.ts';

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

async function getPackageManager(path: string): Promise<string | undefined> {
	if (await fs.exists(`${path}/yarn.lock`)) {
		return 'yarn';
	} else if (await fs.exists(`${path}/pnpm-lock.yaml`)) {
		return 'pnpm';
	} else if (await fs.exists(`${path}/bun.lockb`)) {
		return 'bun';
	} else if (await fs.exists(`${path}/package-lock.json`)) {
		return 'npm';
	}
	return undefined;
}

export async function collectRepoData() {
	const pluginData = await Bun.file(PLUGIN_DATA_PATH).json() as PluginDataInterface[];

	let allDependencies: string[] = [];

	for (const plugin of pluginData) {
		if (plugin.removedCommit) {
			continue;
		}

		const repoPath = `pluginRepos/repos/${plugin.id}`;
		if (!await fs.exists(repoPath)) {
			console.log(`Repo for plugin ${plugin.id} does not exist`);
			continue;
		}

		const data: PluginRepoData = {
			id: plugin.id,
			hasPackageJson: false,
			packageManager: undefined,
			dependencies: [],
			devDependencies: [],
			installedTestingFrameworks: [],
			hasTestFiles: false,
			fileCounts: {},
		}

		const files = await listFiles(repoPath);

		const excludedExtensions = ["LICENSE"]

		for (const file of files) {
			if (file.endsWith('.test.ts') || file.endsWith('.test.js') || file.endsWith('.spec.ts') || file.endsWith('.spec.js')) {
				data.hasTestFiles = true;
			}

			const extension = file.split('/').at(-1)?.split('.').at(-1);
			if (!extension || excludedExtensions.includes(extension)) {
				continue;
			}

			if (!data.fileCounts[extension]) {
				data.fileCounts[extension] = 0;
			}

			data.fileCounts[extension]++;
		}

		data.hasPackageJson = await fs.exists(`${repoPath}/package.json`);

		if (data.hasPackageJson) {
			const packageJson = await Bun.file(`${repoPath}/package.json`).json();

			data.packageManager = await getPackageManager(repoPath);
			const dependencies: Record<string, string> = packageJson.dependencies ?? {};
			const devDependencies: Record<string, string> = packageJson.devDependencies ?? {};
			data.dependencies = Object.keys(dependencies);
			data.devDependencies = Object.keys(devDependencies);
			const allDependencyNames = uniqueConcat([...data.dependencies], data.devDependencies);

			allDependencies = uniqueConcat(allDependencies, allDependencyNames);

			const testFrameworks: string[] = [
				"jest",
				"mocha",
				"vitest",
				"@types/bun",
			]

			data.installedTestingFrameworks = arrayIntersect(testFrameworks, allDependencyNames);
		}

		console.log(data);

		const writeFile = Bun.file(`pluginRepos/data/${plugin.id}.json`);
		await Bun.write(writeFile, JSON.stringify(data))
	}

	console.log(allDependencies);
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