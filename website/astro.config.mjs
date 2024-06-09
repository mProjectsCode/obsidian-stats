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
				github: 'https://github.com/mProjectsCode/obsidian-stats',
			},
			components: {
				TableOfContents: './src/components/TableOfContents.astro',
				SocialIcons: './src/components/SocialIcons.astro',
			},
			sidebar: [
				{
					label: 'Home',
					autogenerate: {
						directory: 'home',
					},
				},
				{
					label: 'Plugin Stats',
					autogenerate: {
						directory: 'pluginStats',
					},
				},
				{
					label: 'Theme Stats',
					autogenerate: {
						directory: 'themeStats',
					},
				},
				{
					label: 'Release Stats',
					autogenerate: {
						directory: 'releaseStats',
					},
				},
			],
		}),
		svelte(),
	],
	redirects: {
		'/globalstats/plugins/': '/obsidian-stats/home/about/',
	},
});
