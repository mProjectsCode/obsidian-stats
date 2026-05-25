import fs from 'fs/promises';
import path from 'path';
import { loadWasm, wasm } from './wasmLoader';

async function readChunksInDir(dir: string): Promise<string[]> {
	const dirFiles = await fs.readdir(path.resolve(process.cwd(), dir));
	const jsonFiles = dirFiles.filter(file => file.endsWith('.json'));
	return Promise.all(jsonFiles.map(file => fs.readFile(path.resolve(process.cwd(), dir, file), 'utf-8')));
}

async function readDataFile(file: string): Promise<string> {
	return fs.readFile(path.resolve(process.cwd(), file), 'utf-8');
}

// ---------------------------------
// PLUGIN DATA
// ---------------------------------

let pluginData: wasm.PluginDataArray | null = null;
let pluginDataPromise: Promise<wasm.PluginDataArray> | null = null;

export async function getPluginDataArray(): Promise<wasm.PluginDataArray> {
	if (pluginData) {
		return pluginData;
	}

	if (pluginDataPromise) {
		return pluginDataPromise;
	}

	pluginDataPromise = loadPluginData()
		.then(loadedData => {
			pluginData = loadedData;
			return pluginData;
		})
		.finally(() => {
			pluginDataPromise = null;
		});

	return pluginDataPromise;
}

async function loadPluginData(): Promise<wasm.PluginDataArray> {
	const pluginDataChunks = await readChunksInDir('../data/out/plugin-data');
	const pluginRepoDataChunks = await readChunksInDir('../data/out/plugin-repo-data');

	await loadWasm();

	return wasm.load_plugin_data_from_chunks(pluginDataChunks, pluginRepoDataChunks);
}

// ---------------------------------
// THEME DATA
// ---------------------------------

let themeData: wasm.ThemeDataArray | null = null;
let themeDataPromise: Promise<wasm.ThemeDataArray> | null = null;

export async function getThemeDataArray(): Promise<wasm.ThemeDataArray> {
	if (themeData) {
		return themeData;
	}

	if (themeDataPromise) {
		return themeDataPromise;
	}

	themeDataPromise = loadThemeData()
		.then(loadedData => {
			themeData = loadedData;
			return themeData;
		})
		.finally(() => {
			themeDataPromise = null;
		});

	return themeDataPromise;
}

async function loadThemeData(): Promise<wasm.ThemeDataArray> {
	const themeDataChunks = await readChunksInDir('../data/out/theme-data');

	await loadWasm();

	return wasm.load_theme_data_from_chunks(themeDataChunks);
}

// ---------------------------------
// RELEASE DATA
// ---------------------------------

let releaseData: wasm.ReleaseDataArray | null = null;
let releaseDataPromise: Promise<wasm.ReleaseDataArray> | null = null;

export async function getReleaseDataArray(): Promise<wasm.ReleaseDataArray> {
	if (releaseData) {
		return releaseData;
	}

	if (releaseDataPromise) {
		return releaseDataPromise;
	}

	releaseDataPromise = loadReleaseData()
		.then(loadedData => {
			releaseData = loadedData;
			return releaseData;
		})
		.finally(() => {
			releaseDataPromise = null;
		});

	return releaseDataPromise;
}

async function loadReleaseData(): Promise<wasm.ReleaseDataArray> {
	const releasesRawDataChunks = await readChunksInDir('../data/out/releases-github-raw');
	const releasesInterpolatedDataChunks = await readChunksInDir('../data/out/releases-github-interpolated');
	const releasesChangelogChunks = await readChunksInDir('../data/out/releases-changelog');

	await loadWasm();

	return wasm.load_release_data_from_chunks(releasesRawDataChunks, releasesInterpolatedDataChunks, releasesChangelogChunks);
}

// ---------------------------------
// LATEST DATA UPDATE SUMMARY
// ---------------------------------

let latestDataUpdateSummary: wasm.LatestDataUpdateSummary | null = null;
let latestDataUpdateSummaryPromise: Promise<wasm.LatestDataUpdateSummary> | null = null;

export async function getLatestDataUpdateSummary(): Promise<wasm.LatestDataUpdateSummary> {
	if (latestDataUpdateSummary) {
		return latestDataUpdateSummary;
	}

	if (latestDataUpdateSummaryPromise) {
		return latestDataUpdateSummaryPromise;
	}

	latestDataUpdateSummaryPromise = loadLatestDataUpdateSummary()
		.then(summary => {
			latestDataUpdateSummary = summary;
			return summary;
		})
		.finally(() => {
			latestDataUpdateSummaryPromise = null;
		});

	return latestDataUpdateSummaryPromise;
}

async function loadLatestDataUpdateSummary(): Promise<wasm.LatestDataUpdateSummary> {
	const latestDataUpdateSummary = await readDataFile('../data/out/state/latest-data-update-summary.json');

	await loadWasm();

	return wasm.load_latest_data_update_summary(latestDataUpdateSummary);
}

// ---------------------------------
// PLUGIN PAGE FRESHNESS DATA
// ---------------------------------

let pluginPageFreshnessData: wasm.PluginPageFreshnessData | null = null;
let pluginPageFreshnessDataPromise: Promise<wasm.PluginPageFreshnessData> | null = null;

export async function getPluginPageFreshnessData(): Promise<wasm.PluginPageFreshnessData> {
	if (pluginPageFreshnessData) {
		return pluginPageFreshnessData;
	}

	if (pluginPageFreshnessDataPromise) {
		return pluginPageFreshnessDataPromise;
	}

	pluginPageFreshnessDataPromise = loadPluginPageFreshnessData()
		.then(loadedData => {
			pluginPageFreshnessData = loadedData;
			return pluginPageFreshnessData;
		})
		.finally(() => {
			pluginPageFreshnessDataPromise = null;
		});

	return pluginPageFreshnessDataPromise;
}

export async function getPluginPageFreshness(pluginId: string): Promise<wasm.PluginPageFreshness> {
	const data = await getPluginPageFreshnessData();
	return data.get(pluginId);
}

async function loadPluginPageFreshnessData(): Promise<wasm.PluginPageFreshnessData> {
	const [latestDataUpdateSummary, cloneState, releaseState] = await Promise.all([
		readDataFile('../data/out/state/latest-data-update-summary.json'),
		readDataFile('../data/out/state/clone-state.json'),
		readDataFile('../data/out/state/plugin-release-enrichment-state.json'),
	]);

	await loadWasm();

	return wasm.load_plugin_page_freshness_data(latestDataUpdateSummary, cloneState, releaseState);
}
