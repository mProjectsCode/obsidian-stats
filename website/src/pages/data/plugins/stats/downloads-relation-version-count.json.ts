import type { APIRoute } from 'astro';
import { getPluginDataArray } from '../../../../utils/data';
import type { DownloadRelationVersionCountDataPoint } from '../../../../utils/dataUrlTypes';

export const GET: APIRoute = async () => {
	const data = await getPluginDataArray();
	const view = data.view();

	const individualDownloadDataPoints = view.individual_download_data(data);

	const downloadRelationDataPoints: DownloadRelationVersionCountDataPoint[] = individualDownloadDataPoints
		.filter(x => x.downloads > 0 && x.version_count > 0)
		.map(point => {
			return {
				id: point.id,
				name: point.name,
				downloads: point.downloads,
				version_count: point.version_count,
			};
		});

	return new Response(JSON.stringify(downloadRelationDataPoints), {
		headers: {
			'Content-Type': 'application/json',
		},
	});
};
