import { buildReleaseStats } from './release';

export async function buildStats() {
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
