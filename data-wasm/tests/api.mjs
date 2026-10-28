// Run after `make wasm`: node data-wasm/tests/api.mjs
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import init, {
    load_plugin_data_from_chunks,
    load_release_data_from_chunks,
    load_latest_data_update_summary,
} from '../pkg/data_wasm.js';

await init({ module_or_path: await readFile(new URL('../pkg/data_wasm_bg.wasm', import.meta.url)) });

const entry = { id: 'test', name: 'Test', author: 'Author', description: '', repo: 'owner/test' };
const commit = { date: { year: 2025, month: 1, day: 1 }, hash: 'abc123' };
const data = load_plugin_data_from_chunks([JSON.stringify([{
    id: entry.id,
    added_commit: commit,
    initial_entry: entry,
    current_entry: entry,
}])], []);
const view = data.view();
const plugin = view.get(data, 0);

// Typed scalar, vector, enum and optional results retain their JavaScript shapes.
assert.deepEqual(plugin.added_commit(), { date: '2025-01-01', hash: 'abc123' });
assert.equal(plugin.removed_commit(), undefined);
assert.equal(plugin.funding_url(), undefined);
assert.equal(plugin.main_js_findings(), undefined);
assert.deepEqual(plugin.changes(), []);
assert.ok(Array.isArray(plugin.warnings()));
assert.equal(view.overview(data)[0].added_commit.hash, 'abc123');
assert.ok(Array.isArray(view.repo_data_points(data).bundlers));

// A failed export must leave the instance usable for subsequent calls.
assert.throws(() => view.license_data_points(data, '{'), /Failed to parse license data/);
assert.throws(() => load_latest_data_update_summary('{'), /Failed to parse latest data update summary/);
assert.equal(plugin.added_commit().hash, 'abc123');

const releases = load_release_data_from_chunks([], [], []);
assert.deepEqual(releases.get_changelog_changes(), []);
assert.deepEqual(releases.total_downloads_per_version_by_os(), []);
releases.free();
plugin.free();
view.free();
data.free();
console.log('WASM API checks passed');
