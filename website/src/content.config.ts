import { defineCollection, z } from 'astro:content';
import { docsLoader } from "@astrojs/starlight/loaders";
import { docsSchema } from '@astrojs/starlight/schema';

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
};
