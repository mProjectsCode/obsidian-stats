import { PluginDataInterface } from '../plugin/plugin.ts';
import fs from 'node:fs/promises';
import { $, Verboseness } from '../shellUtils.ts';
import { arrayIntersect, getPluginData_dataCollection, uniqueConcat } from '../utils.ts';
import { PluginManifest, PluginRepoData, PluginRepoExtractedData } from './types.ts';
import CliProgress from 'cli-progress';
import { warnings } from './warnings.ts';
import { LicenseComparer } from '../license/licenseCompare.ts';
import { ORR_CommunityPluginDeprecations, ORR_CommunityPluginRemoved } from '../types.ts';

export async function clonePluginRepos() {
	const pluginData = getPluginData_dataCollection();

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

	return identifier;
}

async function extractFromRepo(plugin: PluginDataInterface, licenseComparer: LicenseComparer): Promise<PluginRepoExtractedData | undefined> {
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
		license: 'not found',
		licenseFile: 'not found',
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

	const licenseFile = await tryReadLicense(repoPath);
	if (licenseFile !== undefined) {
		data.licenseFile = licenseComparer.compare(licenseFile) ?? 'unknown';
		if (data.license !== data.licenseFile) {
			console.log(`License mismatch for ${plugin.id}: ${data.license} vs ${data.licenseFile}`);
		}
	}

	return data;
}

async function tryReadLicense(repoPath: string): Promise<string | undefined> {
	if (await fs.exists(`${repoPath}/LICENSE`)) {
		return await fs.readFile(`${repoPath}/LICENSE`, 'utf-8');
	} else if (await fs.exists(`${repoPath}/LICENSE.md`)) {
		return await fs.readFile(`${repoPath}/LICENSE.md`, 'utf-8');
	} else if (await fs.exists(`${repoPath}/LICENSE.txt`)) {
		return await fs.readFile(`${repoPath}/LICENSE.txt`, 'utf-8');
	} else if (await fs.exists(`${repoPath}/LICENSE.MD`)) {
		return await fs.readFile(`${repoPath}/LICENSE.MD`, 'utf-8');
	} else if (await fs.exists(`${repoPath}/LICENSE.TXT`)) {
		return await fs.readFile(`${repoPath}/LICENSE.TXT`, 'utf-8');
	}

	return undefined;
}

export async function collectRepoData() {
	await fs.rm('pluginRepos/data', { recursive: true, force: true });

	const pluginRemovedList = (await Bun.file('obsidian-releases/community-plugins-removed.json').json()) as ORR_CommunityPluginRemoved[];
	const pluginDeprecations = (await Bun.file('obsidian-releases/community-plugin-deprecation.json').json()) as ORR_CommunityPluginDeprecations;

	const pluginData = getPluginData_dataCollection();

	const licenseComparer = new LicenseComparer();
	await licenseComparer.init();

	for (const plugin of pluginData) {
		const data: PluginRepoData = {
			id: plugin.id,
			repo: undefined,
			warnings: [],
			removalReason: undefined,
			deprecatedVersions: [],
		};

		const removedListEntry = pluginRemovedList.find(x => x.id === plugin.id);
		if (removedListEntry) {
			data.removalReason = removedListEntry.reason;
		}

		if (pluginDeprecations[plugin.id]) {
			data.deprecatedVersions = pluginDeprecations[plugin.id];
		}

		if (!plugin.removedCommit) {
			data.repo = await extractFromRepo(plugin, licenseComparer);
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
