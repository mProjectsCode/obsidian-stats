import init from '../../../data-wasm/pkg/data_wasm';
// import wasmbin from '../../../data-wasm/pkg/data_wasm_bg.wasm?raw';
import fs from 'fs/promises';

let wasm_promise: Promise<void> | undefined = undefined;

export async function loadWasm() {
	if (wasm_promise) {
		return wasm_promise;
	} else {
		wasm_promise = (async () => {
			const wasmbin = await fs.readFile(new URL('../../../data-wasm/pkg/data_wasm_bg.wasm', import.meta.url));

			await init({ module_or_path: wasmbin as any });
		})();

		return wasm_promise;
	}
}

export * as wasm from '../../../data-wasm/pkg/data_wasm';
