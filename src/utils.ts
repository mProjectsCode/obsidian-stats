import { type Commit, PerMonthDataPoint } from './types.ts';
import { reduce } from 'itertools-ts';

export class UserError extends Error {}

export function dateToString(date: Date): string {
	return `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2, '0')}-${date.getDate().toString().padStart(2, '0')}`;
}

export function utcStringToString(date: string): string {
	return dateToString(new Date(date));
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
		.map(x => x.replaceAll('"', ''))
		.map(x => x.split(' '))
		.map(x => ({ date: x[0], hash: x[1] }));
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
	if (factors.length === 0) return [];
	if (factors.length === 1) return [value];

	const totalWeight = factors.reduce((a, b) => a + b, 0);
	const actual = new Array(factors.length);
	const error = new Array(factors.length);
	const rounded = new Array(factors.length);

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

export interface AbstractDataInterface {
	id: string;
	addedCommit: Commit;
	removedCommit?: Commit;
}

/**
 * Returns true if the data has been removed (has a removed commit).
 *
 * @param data
 */
export function isRemoved(data: AbstractDataInterface): boolean {
	return data.removedCommit !== undefined;
}

export function filterRemoved<T extends AbstractDataInterface>(data: T[]): T[] {
	return data.filter(x => isRemoved(x));
}

export function iterateDataMonthly<T>(data: AbstractDataInterface[], fn: (d: Date, year: string, month: string) => T): T[] {
	const retData: T[] = [];

	const startDate = findEarliestData(data);
	const endDate = findLatestData(data);

	for (let d = startDate; d <= endDate; d.setMonth(d.getMonth() + 1)) {
		const year = d.getFullYear().toString();
		const month = (d.getMonth() + 1).toString().padStart(2, '0');

		retData.push(fn(d, year, month));
	}

	return retData;
}

export function iterateWeekly<T>(startDate: Date, endDate: Date, fn: (d: Date, date: string) => T): T[] {
	const retData: T[] = [];

	// advance the end date by one day, otherwise the last week will be missing sometimes
	endDate.setDate(endDate.getDate() + ((7 - endDate.getDay()) % 7) + 1);
	startDate.setDate(startDate.getDate() + ((7 - startDate.getDay()) % 7));

	for (let d = startDate; d <= endDate; d.setDate(d.getDate() + 7)) {
		const date = dateToString(d);

		retData.push(fn(d, date));
	}

	return retData;
}

export function findEarliestData(data: AbstractDataInterface[]): Date {
	const minAddedDate = reduce.toMin(data, d => d.addedCommit.date)?.addedCommit.date;
	const minRemovedDate = reduce.toMin(
		data.filter(x => x.removedCommit),
		d => d.removedCommit!.date,
	)?.addedCommit.date;

	if (minAddedDate === undefined && minRemovedDate === undefined) {
		throw new Error('No data found');
	}

	if (minAddedDate === undefined) {
		return new Date(minRemovedDate!);
	}

	if (minRemovedDate === undefined) {
		return new Date(minAddedDate);
	}

	if (minAddedDate < minRemovedDate) {
		return new Date(minAddedDate);
	} else {
		return new Date(minRemovedDate);
	}
}

export function findLatestData(data: AbstractDataInterface[]): Date {
	const maxAddedDate = reduce.toMax(data, d => d.addedCommit.date)?.addedCommit.date;
	const maxRemovedDate = reduce.toMax(
		data.filter(x => isRemoved(x)),
		d => d.removedCommit!.date,
	)?.addedCommit.date;

	if (maxAddedDate === undefined && maxRemovedDate === undefined) {
		throw new Error('No data found');
	}

	if (maxAddedDate === undefined) {
		return new Date(maxRemovedDate!);
	}

	if (maxRemovedDate === undefined) {
		return new Date(maxAddedDate);
	}

	if (maxAddedDate > maxRemovedDate) {
		return new Date(maxAddedDate);
	} else {
		return new Date(maxRemovedDate);
	}
}

export function getAddedDataForMonth<T extends AbstractDataInterface>(data: T[], year: string, month: string): T[] {
	return data.filter(x => {
		const addedDate = x.addedCommit.date.split('-');
		return addedDate[0] === year && addedDate[1] === month;
	});
}

export function getRemovedDataForMonth<T extends AbstractDataInterface>(data: T[], year: string, month: string): T[] {
	return filterRemoved(data).filter(x => {
		const removedDate = x.removedCommit!.date.split('-');
		return removedDate[0] === year && removedDate[1] === month;
	});
}
