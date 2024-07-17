import { PLUGIN_DATA_PATH } from '../constants.ts';
import { PluginDataInterface } from '../plugin/plugin.ts';
import fs from 'node:fs/promises';
import { $, Verboseness } from '../shellUtils.ts';
import { arrayIntersect, getPluginData, getPluginData_dataCollection, uniqueConcat } from '../utils.ts';
import { PluginManifest, PluginRepoData, PluginRepoExtractedData } from './types.ts';
import CliProgress from 'cli-progress';
import { warnings } from './warnings.ts';

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

function normalizeLicenseIdentifier(identifier: string | undefined | null): string {
	if (identifier === null || identifier === undefined || identifier === '') {
		return 'no license';
	}

	const lcIdentifier = identifier.toLowerCase();

	if (lcIdentifier.includes('tbd')) {
		return 'no license';
	}

	if (lcIdentifier.includes('mit')) {
		return 'MIT';
	}

	if (lcIdentifier.includes('see license')) {
		return 'see license file';
	}

	if (lcIdentifier.includes('agpl') && lcIdentifier.includes('3')) {
		return 'AGPL-3.0';
	}

	if (lcIdentifier.includes('gpl') && lcIdentifier.includes('3')) {
		return 'GPL-3.0';
	}

	if (lcIdentifier.includes('gnu') && lcIdentifier.includes('3')) {
		return 'GPL-3.0';
	}

	if (lcIdentifier.includes('agpl') && lcIdentifier.includes('2')) {
		return 'AGPL-2.0';
	}

	if (lcIdentifier.includes('gpl') && lcIdentifier.includes('2')) {
		return 'GPL-2.0';
	}

	if (lcIdentifier.includes('gnu') && lcIdentifier.includes('2')) {
		return 'GPL-2.0';
	}

	if (lcIdentifier.includes('gnu') || lcIdentifier.includes('gpl')) {
		return 'GPL-2.0';
	}

	if (lcIdentifier.includes('apache') && lcIdentifier.includes('2')) {
		return 'Apache-2.0';
	}

	if (lcIdentifier.includes('apache')) {
		return 'Apache-2.0';
	}

	if (lcIdentifier === 'the unlicense' || lcIdentifier === 'unlicense') {
		return 'Unlicense';
	}

	if (lcIdentifier === 'unlicensed') {
		return 'explicitly unlicensed';
	}

	if (lcIdentifier === 'isc') {
		return 'ISC';
	}

	if (lcIdentifier === 'mpl-2.0') {
		return 'MPL-2.0';
	}

	if (lcIdentifier === 'mpl-1.1') {
		return 'MPL-1.1';
	}

	if (lcIdentifier === 'mpl-1.0') {
		return 'MPL-1.0';
	}

	if (lcIdentifier.includes('bsd') && lcIdentifier.includes('3')) {
		return 'BSD-3-Clause';
	}

	if (lcIdentifier.includes('bsd') && lcIdentifier.includes('2')) {
		return 'BSD-2-Clause';
	}

	if (lcIdentifier.includes('bsd') && lcIdentifier.includes('0')) {
		return '0BSD';
	}

	if ((lcIdentifier.includes('bsd') && lcIdentifier.includes('4')) || lcIdentifier === 'bsd') {
		return 'BSD-4-Clause';
	}

	if (lcIdentifier === 'blueoak-1.0.0') {
		return 'BlueOak-1.0.0';
	}

	if (lcIdentifier.includes('cc0')) {
		return 'CC0';
	}

	if (lcIdentifier === 'wtfpl') {
		return 'CC0';
	}

	console.warn(`Unknown license: ${identifier}`);

	return 'unknown';
}

async function extractFromRepo(plugin: PluginDataInterface): Promise<PluginRepoExtractedData | undefined> {
	const repoPath = `pluginRepos/repos/${plugin.id}`;
	if (!(await fs.exists(repoPath))) {
		console.log(`Repo for plugin ${plugin.id} does not exist`);
		return undefined;
	}

	const manifest = (await Bun.file(`${repoPath}/manifest.json`).json()) as PluginManifest;

	const data: PluginRepoExtractedData = {
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
		license: 'unknown',
		manifest: manifest,
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

		const testFrameworks: string[] = ['jest', 'mocha', 'vitest', '@types/bun', 'bun-types'];

		const installedTestingFrameworks = new Set(arrayIntersect(testFrameworks, allDependencyNames));
		if (installedTestingFrameworks.has('bun-types')) {
			installedTestingFrameworks.delete('bun-types');
			installedTestingFrameworks.add('bun:test');
		}
		if (installedTestingFrameworks.has('@types/bun')) {
			installedTestingFrameworks.delete('@types/bun');
			installedTestingFrameworks.add('bun:test');
		}

		data.installedTestingFrameworks = Array.from(installedTestingFrameworks);

		const bundlers: string[] = ['esbuild', 'rollup', 'webpack', 'vite', 'turbo'];

		data.installedBundlers = arrayIntersect(bundlers, allDependencyNames);

		data.license = normalizeLicenseIdentifier(packageJson.license);
	}

	return data;
}

export async function collectRepoData() {
	await fs.rm('pluginRepos/data', { recursive: true, force: true });

	const pluginData = getPluginData_dataCollection();

	for (const plugin of pluginData) {
		const data: PluginRepoData = {
			id: plugin.id,
			repo: undefined,
			warnings: [],
		};

		if (!plugin.removedCommit) {
			data.repo = await extractFromRepo(plugin);
		}

		data.warnings = warnings.map(x => x(plugin, data.repo)).filter(x => x !== undefined);

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
