import type { APIRoute } from 'astro';
import { getPluginDataArray } from '../../../utils/data';

export const GET: APIRoute = async () => {
	const dataArray = await getPluginDataArray();
	const view = dataArray.view();
	const data = view.overview(dataArray);

	return new Response(JSON.stringify(data), {
		headers: {
			'Content-Type': 'application/json',
		},
	});
};
