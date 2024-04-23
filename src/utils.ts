import { type Commit, PerMonthDataPoint } from './types.ts';
import { reduce } from 'itertools-ts';
import { CDate } from './date.ts';

export function prettyDateString(date: string): string {
	return CDate.fromDate(new Date(date)).toString();
}

export function gitLogToCommits(log: string): Commit[] {
	return log
		.split('\n')
		.filter(x => x.trim() !== '')
		.map(x => {
			let y = x.replaceAll('"', '').split(' ');
			return {
				date: y[0].split('T')[0],
				hash: y[1],
			} satisfies Commit;
		});
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

	CDate.iterateMonthly(startDate, endDate, date => {
		const year = date.year.toString();
		const month = date.month.toString().padStart(2, '0');

		retData.push(fn(date.toDate(), year, month));
	});

	return retData;
}

export function findEarliestData(data: AbstractDataInterface[]): CDate {
	const minAddedDate = reduce.toMin(data, d => d.addedCommit.date)?.addedCommit.date;
	const minRemovedDate = reduce.toMin(
		data.filter(x => x.removedCommit),
		d => d.removedCommit!.date,
	)?.addedCommit.date;

	if (minAddedDate === undefined && minRemovedDate === undefined) {
		throw new Error('No data found');
	}

	if (minAddedDate === undefined) {
		return CDate.fromString(minRemovedDate!);
	}

	if (minRemovedDate === undefined) {
		return CDate.fromString(minAddedDate);
	}

	if (minAddedDate < minRemovedDate) {
		return CDate.fromString(minAddedDate);
	} else {
		return CDate.fromString(minRemovedDate);
	}
}

export function findLatestData(data: AbstractDataInterface[]): CDate {
	const maxAddedDate = reduce.toMax(data, d => d.addedCommit.date)?.addedCommit.date;
	const maxRemovedDate = reduce.toMax(
		data.filter(x => isRemoved(x)),
		d => d.removedCommit!.date,
	)?.addedCommit.date;

	if (maxAddedDate === undefined && maxRemovedDate === undefined) {
		throw new Error('No data found');
	}

	if (maxAddedDate === undefined) {
		return CDate.fromString(maxRemovedDate!);
	}

	if (maxRemovedDate === undefined) {
		return CDate.fromString(maxAddedDate);
	}

	if (maxAddedDate > maxRemovedDate) {
		return CDate.fromString(maxAddedDate);
	} else {
		return CDate.fromString(maxRemovedDate);
	}
}

export function getAddedDataForMonth<T extends AbstractDataInterface>(data: T[], year: string, month: string): T[] {
	return data.filter(x => x.addedCommit.date.startsWith(`${year}-${month}`));
}

export function getRemovedDataForMonth<T extends AbstractDataInterface>(data: T[], year: string, month: string): T[] {
	return filterRemoved(data).filter(x => x.removedCommit!.date.startsWith(`${year}-${month}`));
}

export function uniqueConcat<T>(a: T[], b: T[]): T[] {
	for (const x of b) {
		if (!a.includes(x)) {
			a.push(x);
		}
	}

	return a;
}

export function arrayIntersect<T>(a: T[], b: T[]): T[] {
	return a.filter(x => b.includes(x));
}

export function encodeName(name: string): string {
	return JSON.stringify(name).slice(1, -1);
}

export function groupBy<T, K extends keyof any>(list: T[], getKey: (item: T) => K): Record<K, T[]> {
	return list.reduce<Record<K, T[]>>(
		(previous, currentItem) => {
			const group = getKey(currentItem);
			if (!previous[group]) {
				previous[group] = [];
			}
			previous[group].push(currentItem);
			return previous;
		},
		{} as Record<K, T[]>,
	);
}

export function multiGroupBy<T, K extends keyof any>(list: T[], getKey: (item: T) => K[]): Record<K, T[]> {
	return list.reduce<Record<K, T[]>>(
		(previous, currentItem) => {
			const group = getKey(currentItem);
			for (const subGroup of group) {
				if (!previous[subGroup]) {
					previous[subGroup] = [];
				}
				previous[subGroup].push(currentItem);
			}

			return previous;
		},
		{} as Record<K, T[]>,
	);
}
