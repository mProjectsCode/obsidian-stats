import { type ThemeDataInterface } from './theme.ts';
import { type PerMonthDataPoint } from '../types.ts';
import { filterNonRemoved, filterRemoved, getAddedDataForMonth, getRemovedDataForMonth, isRemoved, iterateDataMonthly } from '../utils.ts';

export function getThemeRemovedList(themes: ThemeDataInterface[]): ThemeDataInterface[] {
	return filterRemoved(themes).sort((a, b) => a.id.localeCompare(b.id));
}

export function getThemeRemovedRecentList(themes: ThemeDataInterface[]): ThemeDataInterface[] {
	return filterRemoved(themes)
		.sort((a, b) => new Date(b.removedCommit!.date).valueOf() - new Date(a.removedCommit!.date).valueOf())
		.slice(0, 15);
}

export function getThemePercentageRemovedByReleaseMonth(themes: ThemeDataInterface[]): PerMonthDataPoint[] {
	return iterateDataMonthly<PerMonthDataPoint>(themes, (d, year, month) => {
		const releasedThemes = getAddedDataForMonth(themes, year, month);
		const retiredThemes = filterRemoved(releasedThemes);

		return {
			year: year,
			month: month,
			value: (retiredThemes.length / releasedThemes.length) * 100,
		};
	});
}

export function getThemeCountAddedMonthly(themes: ThemeDataInterface[]): PerMonthDataPoint[] {
	return iterateDataMonthly<PerMonthDataPoint>(themes, (d, year, month) => {
		const releasedThemes = getAddedDataForMonth(themes, year, month);

		return {
			year: year,
			month: month,
			value: releasedThemes.length,
		};
	});
}

export function getThemeAddedRecentList(themes: ThemeDataInterface[]): ThemeDataInterface[] {
	return filterNonRemoved(themes)
		.sort((a, b) => b.addedCommit.date.localeCompare(a.addedCommit.date))
		.slice(0, 15);
}

export function getThemeCountMonthly(themes: ThemeDataInterface[]): PerMonthDataPoint[] {
	let total = 0;

	return iterateDataMonthly<PerMonthDataPoint>(themes, (d, year, month) => {
		const releasedThemes = getAddedDataForMonth(themes, year, month);

		total += releasedThemes.length;

		return {
			year: year,
			month: month,
			value: total,
		};
	});
}

export function getThemeCountWoRetiredMonthly(themes: ThemeDataInterface[]): PerMonthDataPoint[] {
	let total = 0;

	return iterateDataMonthly<PerMonthDataPoint>(themes, (d, year, month) => {
		const releasedThemes = getAddedDataForMonth(themes, year, month);
		const retiredThemes = getRemovedDataForMonth(themes, year, month);

		total += releasedThemes.length - retiredThemes.length;

		return {
			year: year,
			month: month,
			value: total,
		};
	});
}

export function getThemeCountRemovedMonthly(themes: ThemeDataInterface[]): PerMonthDataPoint[] {
	let total = 0;

	return iterateDataMonthly<PerMonthDataPoint>(themes, (d, year, month) => {
		const retiredThemes = getRemovedDataForMonth(themes, year, month);

		total += retiredThemes.length;

		return {
			year: year,
			month: month,
			value: total,
		};
	});
}

export function getThemeCountRemovedChangeMonthly(themes: ThemeDataInterface[]): PerMonthDataPoint[] {
	return iterateDataMonthly<PerMonthDataPoint>(themes, (d, year, month) => {
		const retiredThemes = getRemovedDataForMonth(themes, year, month);

		return {
			year: year,
			month: month,
			value: retiredThemes.length,
		};
	});
}
