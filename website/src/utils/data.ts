import fs from 'fs/promises';
import { loadWasm, wasm } from './wasmLoader';

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

async function loadPluginData(): Promise<wasm.PluginDataArray> {
	const pluginDataChunks = await readChunksInDir('../../../data/out/plugin-data');
	const pluginRepoDataChunks = await readChunksInDir('../../../data/out/plugin-repo-data');

	await loadWasm();

	return wasm.load_plugin_data_from_chunks(pluginDataChunks, pluginRepoDataChunks);
}

async function loadThemeData(): Promise<wasm.ThemeDataArray> {
	const themeDataChunks = await readChunksInDir('../../../data/out/theme-data');

	await loadWasm();

	return wasm.load_theme_data_from_chunks(themeDataChunks);
}

async function readChunksInDir(dir: string): Promise<string[]> {
	const dirFiles = await fs.readdir(new URL(dir, import.meta.url));
	const jsonFiles = dirFiles.filter(file => file.endsWith('.json'));
	return Promise.all(jsonFiles.map(file => fs.readFile(new URL(`${dir}/${file}`, import.meta.url), 'utf-8')));
}
