import fs from 'fs/promises';
import { loadWasm, wasm } from './wasmLoader';

async function readChunksInDir(dir: string): Promise<string[]> {
	const dirFiles = await fs.readdir(new URL(dir, import.meta.url));
	const jsonFiles = dirFiles.filter(file => file.endsWith('.json'));
	return Promise.all(jsonFiles.map(file => fs.readFile(new URL(`${dir}/${file}`, import.meta.url), 'utf-8')));
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
	const pluginDataChunks = await readChunksInDir('../../../data/out/plugin-data');
	const pluginRepoDataChunks = await readChunksInDir('../../../data/out/plugin-repo-data');

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
	const themeDataChunks = await readChunksInDir('../../../data/out/theme-data');

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
	const releasesRawDataChunks = await readChunksInDir('../../../data/out/releases-github-raw');
	const releasesInterpolatedDataChunks = await readChunksInDir('../../../data/out/releases-github-interpolated');
	const releasesChangelogChunks = await readChunksInDir('../../../data/out/releases-changelog');

	await loadWasm();

	return wasm.load_release_data_from_chunks(releasesRawDataChunks, releasesInterpolatedDataChunks, releasesChangelogChunks);
}
