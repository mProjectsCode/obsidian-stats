import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

import svelte from '@astrojs/svelte';

// https://astro.build/config
export default defineConfig({
	site: 'https://www.moritzjung.dev',
	base: '/obsidian-stats',
	integrations: [
		starlight({
			title: 'Obsidian Stats',
			social: {
				github: 'https://github.com/withastro/starlight',
			},
			sidebar: [
				{
					label: 'Global Stats',
					autogenerate: {
						directory: 'globalStats',
					},
				},
				{
					label: 'Plugins',
					autogenerate: {
						directory: 'plugins',
					},
				},
			],
		}),
		svelte(),
	],
});
