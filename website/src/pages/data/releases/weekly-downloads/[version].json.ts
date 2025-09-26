import type { APIRoute } from 'astro';
import { getReleaseDataArray } from '../../../../utils/data';
import { Version } from '../../../../../../data-wasm/pkg/data_wasm';

export async function getStaticPaths() {
	const data = await getReleaseDataArray();
	const allMinorVersions = data.get_all_minor_versions();

	return allMinorVersions.map(version => {
		return {
			params: {
				version: version.to_fancy_string(),
			},
		};
	});
}

export const GET: APIRoute = async ({ params }) => {
	const versionStr = params.version;
	if (!versionStr) {
		throw new Error(`Version is required.`);
	}
	const version = Version.parse(versionStr);
	if (!version) {
		throw new Error(`Invalid version: ${versionStr}`);
	}

	const data = await getReleaseDataArray();

	const downloadDataPoints = data.weekly_download_data(version);

	return new Response(JSON.stringify(downloadDataPoints), {
		headers: {
			'Content-Type': 'application/json',
		},
	});
};
