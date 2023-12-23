import {type Commit} from "./types.ts";

export class UserError extends Error {
}

export function dateToString(date: Date): string {
	return `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2, '0')}-${date.getDate().toString().padStart(2, '0')}`;
}

export function stringToBase64(str: string): string {
	return Buffer.from(str).toString('base64');
}

export function base64ToString(str: string): string {
	return Buffer.from(str, 'base64').toString();
}


export function dateDiffInDays(a: Date, b: Date): number {
	const _MS_PER_DAY = 1000 * 60 * 60 * 24;
	// Discard the time and time-zone information.
	const utc1 = Date.UTC(a.getFullYear(), a.getMonth(), a.getDate());
	const utc2 = Date.UTC(b.getFullYear(), b.getMonth(), b.getDate());

	return Math.floor((utc2 - utc1) / _MS_PER_DAY);
}

export function gitLogToCommits(log: string): Commit[] {
	return log
		.split('\n')
		.filter(x => x.trim() !== '')
		.map(x => x.replaceAll('\"', ''))
		.map(x => x.split(' '))
		.map(x => ({date: x[0], hash: x[1]}));

}
