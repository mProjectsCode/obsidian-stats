// .prettierrc.mjs
/** @type {import("prettier").Config} */
export default {
	plugins: ['prettier-plugin-astro', 'prettier-plugin-svelte'],

	printWidth: 160,
	useTabs: true,
	semi: true,
	singleQuote: true,
	arrowParens: 'avoid',

	overrides: [
		{
			files: '*.astro',
			options: {
				parser: 'astro',
			},
		},
	],
};
