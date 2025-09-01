import type { APIRoute } from 'astro';
import { getPluginDataArray } from '../../../../utils/data';

export const GET: APIRoute = async () => {
	const data = await getPluginDataArray();
	const view = data.view();

	const downloadDataPoints = view.total_download_data(data);

	return new Response(JSON.stringify(downloadDataPoints), {
		headers: {
			'Content-Type': 'application/json',
		},
	});
};
