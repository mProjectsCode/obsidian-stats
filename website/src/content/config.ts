import { defineCollection, z } from 'astro:content';
import { docsSchema, i18nSchema } from '@astrojs/starlight/schema';

export const collections = {
	docs: defineCollection({ schema: docsSchema({
		extend: z.object({
			links: z.object({
				text: z.string(),
				href: z.string(),
			}).array().optional(),
		})
	}) }),
	i18n: defineCollection({ type: 'data', schema: i18nSchema() }),
};
