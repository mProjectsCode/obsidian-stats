import { PLUGIN_DATA_PATH } from '../constants.ts';
import { PluginDataInterface } from '../plugin/plugin.ts';
import fs from 'node:fs/promises';
import { $, Verboseness } from '../shellUtils.ts';
import { arrayIntersect, uniqueConcat } from '../utils.ts';
import { PluginRepoData } from './types.ts';
import CliProgress from 'cli-progress';

export async function clonePluginRepos() {
	const pluginData = (await Bun.file(PLUGIN_DATA_PATH).json()) as PluginDataInterface[];

	await fs.rm('pluginRepos/repos', { recursive: true, force: true });
	await fs.mkdir('pluginRepos/repos', { recursive: true });

	const skippedPlugins = [];
	const failedPlugins = [];

	console.log('Starting Cloning Repos');

	const progress = new CliProgress.SingleBar({}, CliProgress.Presets.rect);
	progress.start(pluginData.length, 0);

	for (const plugin of pluginData) {
		if (plugin.removedCommit) {
			skippedPlugins.push(plugin.id);
			continue;
		}

		const res = await $(`git clone https://github.com/${plugin.currentEntry.repo}.git pluginRepos/repos/${plugin.id} --depth 1`, undefined, Verboseness.QUITET);

		progress.increment();

		if (res.stderr) {
			failedPlugins.push(plugin.id);
		}
	}

	progress.stop();

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

async function getPackageManager(files: string[]): Promise<string | undefined> {
	if (files.includes(`yarn.lock`)) {
		return 'yarn';
	} else if (files.includes(`pnpm-lock.yaml`)) {
		return 'pnpm';
	} else if (files.includes(`bun.lockb`)) {
		return 'bun';
	} else if (files.includes(`deno.lock`)) {
		return 'deno';
	} else if (files.includes(`package-lock.json`)) {
		return 'npm';
	}
	return undefined;
}

export async function collectRepoData() {
	await fs.rm('pluginRepos/data', { recursive: true, force: true });

	const pluginData = (await Bun.file(PLUGIN_DATA_PATH).json()) as PluginDataInterface[];

	let allDependencies: string[] = [];

	for (const plugin of pluginData) {
		if (plugin.removedCommit) {
			continue;
		}

		const repoPath = `pluginRepos/repos/${plugin.id}`;
		if (!(await fs.exists(repoPath))) {
			console.log(`Repo for plugin ${plugin.id} does not exist`);
			continue;
		}

		const data: PluginRepoData = {
			id: plugin.id,
			usesTypescript: true,
			hasPackageJson: false,
			packageManager: undefined,
			dependencies: [],
			devDependencies: [],
			installedTestingFrameworks: [],
			installedBundlers: [],
			hasTestFiles: false,
			hasBetaManifest: false,
			fileCounts: {},
			license: undefined,
		};

		const files = await listFiles(repoPath);

		const excludedExtensions = ['LICENSE'];

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

		if (!data.fileCounts['ts'] && !data.fileCounts['tsx']) {
			data.usesTypescript = false;
		}

		data.hasBetaManifest = await fs.exists(`${repoPath}/manifest-beta.json`);

		data.hasPackageJson = await fs.exists(`${repoPath}/package.json`);

		if (data.hasPackageJson) {
			const packageJson = await Bun.file(`${repoPath}/package.json`).json();

			data.packageManager = await getPackageManager(files);
			const dependencies: Record<string, string> = packageJson.dependencies ?? {};
			const devDependencies: Record<string, string> = packageJson.devDependencies ?? {};
			data.dependencies = Object.keys(dependencies);
			data.devDependencies = Object.keys(devDependencies);
			const allDependencyNames = uniqueConcat([...data.dependencies], data.devDependencies);

			allDependencies = uniqueConcat(allDependencies, allDependencyNames);

			const testFrameworks: string[] = ['jest', 'mocha', 'vitest', '@types/bun'];

			data.installedTestingFrameworks = arrayIntersect(testFrameworks, allDependencyNames);

			const bundlers: string[] = ['esbuild', 'rollup', 'webpack', 'vite', 'turbo'];

			data.installedBundlers = arrayIntersect(bundlers, allDependencyNames);

			data.license = packageJson.license;
		}

		const writeFile = Bun.file(`pluginRepos/data/${plugin.id}.json`);
		await Bun.write(writeFile, JSON.stringify(data));
	}
}

async function listFiles(dir: string, pathDir: string = ''): Promise<string[]> {
	const files = await fs.readdir(dir, { withFileTypes: true });

	const filteredFiles = await Promise.all(
		files.map(async file => {
			const niceName = pathDir ? `${pathDir}/${file.name}` : file.name;

			if (file.isDirectory()) {
				if (file.name === '.git') {
					return [];
				}

				return await listFiles(`${dir}/${file.name}`, niceName);
			} else {
				return [niceName];
			}
		}),
	);

	return filteredFiles.flat();
}
