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
					label: 'Plugins',
					collapsed: true,
					autogenerate: {
						directory: 'plugins',
					},
				},
				{
					label: 'Themes',
					collapsed: true,
					autogenerate: {
						directory: 'themes',
					},
				},
			],
		}),
		svelte(),
	],
	redirects: {
		'/globalstats/plugins/': '/obsidian-stats/home/about/',
	},
	outDir: 'dist/obsidian-stats',
});
