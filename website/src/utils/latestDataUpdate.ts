import fs from 'node:fs/promises';
import path from 'node:path';

export interface CountShare {
	label: string;
	count: number;
	share: number;
}

export interface LatestDataUpdateSummary {
	refreshed_at_unix: number | null;
	clone_run_at_unix: number | null;
	release_run_at_unix: number | null;
	obsidian_release_fetch_at_unix: number | null;
	latest_plugin_download_snapshot_date: string | null;
	latest_obsidian_release_date: string | null;
	latest_obsidian_version: string | null;
	plugins: {
		total: number;
		active: number;
		removed: number;
		total_downloads: number;
		version_snapshots: number;
	};
	themes: {
		total: number;
		active: number;
		removed: number;
	};
	releases: {
		changelog_entries: number;
		github_release_snapshots: number;
		interpolated_release_snapshots: number;
		asset_count: number;
		download_snapshots: number;
	};
	clone: {
		tracked: number;
		ok: number;
		skipped: number;
		failed: number;
		success_rate: number;
		failed_plugins: string[];
	};
	release_acquisition: {
		tracked: number;
		ok: number;
		retained_previous_main_js: number;
		no_release: number;
		no_main_js_asset: number;
		error_count: number;
		main_js_coverage: number;
		main_js_coverage_rate: number;
		total_main_js_bytes: number;
		average_main_js_bytes: number;
		largest_main_js: {
			repo: string | null;
			size_bytes: number;
		};
		status_counts: CountShare[];
	};
	repo_analysis: {
		tracked: number;
		active_success: number;
		active_failures: number;
		removed_skipped: number;
		coverage_rate: number;
		failure_samples: string[];
		error_counts?: CountShare[];
	};
}

export async function getLatestDataUpdateSummary(): Promise<LatestDataUpdateSummary> {
	const absolutePath = path.resolve(process.cwd(), '../data/out/state/latest-data-update-summary.json');

	try {
		return JSON.parse(await fs.readFile(absolutePath, 'utf8')) as LatestDataUpdateSummary;
	} catch (error) {
		if ((error as NodeJS.ErrnoException).code === 'ENOENT') {
			throw new Error(
				'Missing data/out/state/latest-data-update-summary.json. Run the Rust data pipeline to generate this file.',
			);
		}

		throw error;
	}
}
