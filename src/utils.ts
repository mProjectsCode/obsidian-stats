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


/**
 * Maps a date to the next monday at 1am UTC
 * @param date
 */
export function getNextMonday(date: Date): Date {
	const dayOfWeek = date.getUTCDay();
	const daysUntilMonday = (dayOfWeek === 0 ? 1 : 8) - dayOfWeek;
	const nextMonday = new Date(date.getTime() + daysUntilMonday * 24 * 60 * 60 * 1000);
	nextMonday.setUTCHours(1, 0, 0, 0);
	return nextMonday;
}


/**
 * Returns all mondays at 1am UTC between the two dates
 * @param start - Start date
 * @param end - End date
 * @returns Array of next mondays at 1am UTC between the two dates
 * @example
 * 		getNextMondays(new Date("2023-12-25T00:00:00"), new Date("2024-01-01T00:00:00"))
 * 		// [Date("2023-12-25T01:00:00"), Date("2024-01-01T01:00:00")]
 * 		getNextMondays(new Date("2023-12-25T00:00:00"), new Date("2023-12-25T01:00:00"))
 * 		// [Date("2023-12-25T01:00:00")]
 * 		getNextMondays(new Date("2023-12-10T00:00:00"), new Date("2023-12-25T00:00:00"))
 * 		// [Date("2023-12-11T01:00:00"), Date("2023-12-18T01:00:00"), Date("2023-12-25T01:00:00")]
 */
export function getNextMondays(start: Date, end: Date): Date[] {
	const result = [];
	const startDate = new Date(start.getTime());
	const endDate = new Date(end.getTime());

	let currentDate = getNextMonday(startDate);

	while (currentDate <= endDate) {
		result.push(new Date(currentDate));
		currentDate.setDate(currentDate.getUTCDate() + 7);
	}

	result.push(getNextMonday(endDate));

	return result;
}


/**
 * Distribute a value proportionally to a set of factors.
 * @param value The value to distribute.
 * @param factors The factors to use for the distribution.
 * @returns The distributed value.
 * @remark Adapted from https://stackoverflow.com/questions/1925691/proportionately-distribute-prorate-a-value-across-a-set-of-values
 */
export function distributeValueEqually(value: number, factors: number[]) {
	const totalWeight = factors.reduce((a, b) => a + b, 0);
	const actual = new Array(factors.length),
		  error = new Array(factors.length),
		  rounded = new Array(factors.length);

	let added = 0;
	for (let i = 0; i < factors.length; i++) {
		actual[i] = value * (factors[i] / totalWeight);
		rounded[i] = Math.floor(actual[i]);
		error[i] = actual[i] - rounded[i];
		added += rounded[i];
	}

	while (added < value) {
		let maxError = 0;
		let maxErrorIndex = -1;
		for (let e = 0; e < factors.length; e++) {
			if (error[e] > maxError) {
				maxError = error[e];
				maxErrorIndex = e;
			}
		}

		rounded[maxErrorIndex]++;
		error[maxErrorIndex]--;

		added++;
	}

	return rounded;
}
