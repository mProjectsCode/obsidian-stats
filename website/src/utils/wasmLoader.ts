import init from '../../../data-wasm/pkg/data_wasm';
import path from 'path';
import fs from 'fs/promises';

let wasm_promise: Promise<void> | undefined = undefined;

export async function loadWasm() {
	if (wasm_promise) {
		return wasm_promise;
	} else {
		wasm_promise = (async () => {
			const file_path = path.resolve(process.cwd(), '../data-wasm/pkg/data_wasm_bg.wasm');
			const wasmbin = await fs.readFile(file_path);

			await init({ module_or_path: wasmbin as any });
		})();

		return wasm_promise;
	}
}

export * as wasm from '../../../data-wasm/pkg/data_wasm';
