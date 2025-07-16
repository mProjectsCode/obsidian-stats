import init from '../../../data-wasm/pkg/data_wasm';
// import wasmbin from '../../../data-wasm/pkg/data_wasm_bg.wasm?raw';
import fs from 'fs/promises';

export async function loadWasm() {
    const wasmbin = await fs.readFile(new URL('../../../data-wasm/pkg/data_wasm_bg.wasm', import.meta.url));

	await init({ module_or_path: wasmbin });
}

export * as wasm from '../../../data-wasm/pkg/data_wasm';