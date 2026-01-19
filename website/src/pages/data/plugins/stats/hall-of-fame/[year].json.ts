import type { APIRoute } from 'astro';
import { getPluginDataArray } from '../../../../../utils/data';

const YEARS = [2020, 2021, 2022, 2023, 2024, 2025, 2026];

const ROUTES = ['full', ...YEARS.map(year => [year.toString(), `${year}-new`]).flat()];

export async function getStaticPaths() {
	return YEARS.map(year => {
		return {
			params: {
				year: year,
			},
		};
	});
}

export const GET: APIRoute = async ({ params }) => {
	const year = params.year;
	if (!year) {
		throw new Error(`Year is required.`);
	}

	if (year === 'full') {
		const dataArray = await getPluginDataArray();
		const view = dataArray.view();
		const data = view.most_downloaded(dataArray, 10, null, false);

		return new Response(JSON.stringify(data), {
			headers: {
				'Content-Type': 'application/json',
			},
		});
	}

	const parts = year.split('-');
	const parsedYear = Number(parts[0]);
	const onlyNew = parts[1] === 'new';

	const dataArray = await getPluginDataArray();
	const view = dataArray.view();
	const data = view.most_downloaded(dataArray, 10, parsedYear, onlyNew);

	return new Response(JSON.stringify(data), {
		headers: {
			'Content-Type': 'application/json',
		},
	});
};
