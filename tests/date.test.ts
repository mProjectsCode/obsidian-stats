import { test, expect } from 'bun:test';
import { CDate } from '../src/date';

test('biggerThan', () => {
	const date1 = new CDate(2019, 1, 1);
	const date2 = new CDate(2019, 1, 2);
	const date3 = new CDate(2019, 2, 1);
	const date4 = new CDate(2020, 1, 1);

	expect(date2.biggerThan(date1)).toBeTrue();
	expect(date3.biggerThan(date2)).toBeTrue();
	expect(date4.biggerThan(date3)).toBeTrue();

	expect(date1.biggerThan(date2)).toBeFalse();
	expect(date2.biggerThan(date3)).toBeFalse();
	expect(date3.biggerThan(date4)).toBeFalse();
});

test('smallerThan', () => {
	const date1 = new CDate(2019, 1, 1);
	const date2 = new CDate(2019, 1, 2);
	const date3 = new CDate(2019, 2, 1);
	const date4 = new CDate(2020, 1, 1);

	expect(date2.smallerThan(date1)).toBeFalse();
	expect(date3.smallerThan(date2)).toBeFalse();
	expect(date4.smallerThan(date3)).toBeFalse();

	expect(date1.smallerThan(date2)).toBeTrue();
	expect(date2.smallerThan(date3)).toBeTrue();
	expect(date3.smallerThan(date4)).toBeTrue();
});

test('inRange', () => {
	const date1 = new CDate(2019, 1, 1);
	const date2 = new CDate(2019, 1, 2);
	const date3 = new CDate(2019, 2, 1);

	expect(date2.inRange(date1, date3)).toBeTrue();
});

test('inRange 2', () => {
	const date1 = new CDate(2019, 1, 1);
	const date2 = new CDate(2019, 1, 2);
	const date3 = new CDate(2020, 1, 1);

	expect(date2.inRange(date1, date3)).toBeTrue();
});
