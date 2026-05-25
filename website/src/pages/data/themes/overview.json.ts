import type { APIRoute } from 'astro';
import { getThemeDataArray } from '../../../utils/data';

export const GET: APIRoute = async () => {
	const dataArray = await getThemeDataArray();
	const view = dataArray.view();
	const data = view.overview(dataArray);

	return new Response(JSON.stringify(data), {
		headers: {
			'Content-Type': 'application/json',
		},
	});
};
