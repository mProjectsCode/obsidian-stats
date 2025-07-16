import { defineCollection, z } from 'astro:content';
import { docsLoader } from "@astrojs/starlight/loaders";
import { docsSchema } from '@astrojs/starlight/schema';
import { getPluginDataArray } from './utils/data';

export const collections = {
	docs: defineCollection({
		loader: docsLoader(),
		schema: docsSchema({
			extend: z.object({
				links: z
					.object({
						text: z.string(),
						href: z.string(),
					})
					.array()
					.optional(),
			}),
		}),
	}),
	plugins: defineCollection({
		loader: async () => {
			const data = await getPluginDataArray();
			const view = data.view();
			const ids = view.get_ids(data);

			return ids.map(id => ({
				id: id,
			}));
		},
		schema: z.object({
			id: z.string(),
		}),
	})
};
