import fs from 'fs/promises';
import { loadWasm, wasm } from './wasmLoader';

let data: wasm.FullPluginDataArray | null = null;
let loadingPromise: Promise<wasm.FullPluginDataArray> | null = null;

export async function getPluginDataArray(): Promise<wasm.FullPluginDataArray> {
    return loadData();
}

async function loadData(): Promise<wasm.FullPluginDataArray> {
    const pluginDataChunks = await readChunksInDir('../../../data/out/plugin-data');
    const pluginRepoDataChunks = await readChunksInDir('../../../data/out/plugin-repo-data');

    await loadWasm();

    return wasm.load_data_from_chunks(pluginDataChunks, pluginRepoDataChunks);
}

async function readChunksInDir(dir: string): Promise<string[]> {
    const dirFiles = await fs.readdir(new URL(dir, import.meta.url));
    const jsonFiles = dirFiles.filter(file => file.endsWith('.json'));
    return Promise.all(
        jsonFiles.map(file => fs.readFile(new URL(`${dir}/${file}`, import.meta.url), 'utf-8'))
    );
}