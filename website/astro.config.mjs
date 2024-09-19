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
			logo: {
				light: './src/assets/logo_complex_light.svg',
				dark: './src/assets/logo_complex_dark.svg',
			},
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
					items: [
						{
							label: 'Overview',
							link: '/pluginstats',
						},
						{
							label: 'Hall of Fame',
							link: '/pluginstats/hall-of-fame',
						},
						{
							label: 'Downloads',
							link: '/pluginstats/downloads',
						},
						{
							label: 'Community Plugin List',
							link: '/pluginstats/community-plugin-list',
						},
						{
							label: 'Updates',
							link: '/pluginstats/updates',
						},
						{
							label: 'Repository Data',
							link: '/pluginstats/repo-data',
						},
						{
							label: 'Licenses',
							link: '/pluginstats/licenses',
						},
					],
				},
				{
					label: 'Theme Stats',
					items: [
						{
							label: 'Overview',
							link: '/themestats',
						},
						{
							label: 'Community Theme List',
							link: '/themestats/community-theme-list',
						},
					],
				},
				{
					label: 'Release Stats',
					autogenerate: {
						directory: 'releaseStats',
					},
				},
			],
			customCss: ['./src/styles.css'],
		}),
		svelte(),
	],
	redirects: {
		'/globalstats/plugins/': '/obsidian-stats/',
		'/pluginstats/halloffame/': '/obsidian-stats/pluginstats/hall-of-fame/',
		'/pluginstats/repodata/': '/obsidian-stats/pluginstats/repo-data/',
	},
});
