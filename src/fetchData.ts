import { buildThemeStats } from './theme';
import { buildReleaseStats } from './release';
import { $ } from './shellUtils.ts';

export async function buildStats() {
	await $('git submodule update --remote');

	console.log('');
	console.log('================');
	console.log('   THEME DATA   ');
	console.log('================');
	console.log('');

	try {
		await buildThemeStats();
	} catch (e) {
		console.error(e);
	}

	console.log('');
	console.log('==================');
	console.log('   RELEASE DATA   ');
	console.log('==================');
	console.log('');

	try {
		await buildReleaseStats();
	} catch (e) {
		console.error(e);
	}
}

await buildStats();
