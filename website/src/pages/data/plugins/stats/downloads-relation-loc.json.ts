import type { APIRoute } from 'astro';
import { getPluginDataArray } from '../../../../utils/data';
import type { DownloadRelationLocDataPoint } from '../../../../utils/dataUrlTypes';

export const GET: APIRoute = async () => {
	const data = await getPluginDataArray();
	const view = data.view();

	const individualDownloadDataPoints = view.individual_download_data(data);

	const downloadRelationDataPoints: DownloadRelationLocDataPoint[] = individualDownloadDataPoints
		.filter(x => x.downloads > 0 && x.version_count > 0 && x.total_loc > 0)
		.map(point => {
			return {
				id: point.id,
				name: point.name,
				downloads: point.downloads,
				total_loc: point.total_loc,
			};
		});

	return new Response(JSON.stringify(downloadRelationDataPoints), {
		headers: {
			'Content-Type': 'application/json',
		},
	});
};
