import type { APIRoute } from 'astro';
import { getCollection } from 'astro:content';
import { getPluginDataArray } from '../../../../utils/data';

export async function getStaticPaths() {
	const plugins = await getCollection('plugins');

	return plugins.map(plugin => {
		return {
			params: {
				plugin: plugin.id,
			},
		};
	});
}

export const GET: APIRoute = async ({ params }) => {
	const pluginId = params.plugin;
	if (!pluginId) {
		throw new Error(`Plugin ID is required.`);
	}

	const data = await getPluginDataArray();
	const view = data.view();
	const plugin = view.get_by_id(data, pluginId);
	if (!plugin) {
		throw new Error(`Plugin with ID ${pluginId} not found.`);
	}

	const pluginDownloadData = plugin.download_data_points();

	return new Response(JSON.stringify(pluginDownloadData), {
		headers: {
			'Content-Type': 'application/json',
		},
	});
};
