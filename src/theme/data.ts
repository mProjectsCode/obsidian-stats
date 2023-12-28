import {type ThemeDataInterface} from "./theme.ts";
import {type PerMonthDataPoint} from "../types.ts";

export function getRetiredThemes(themes: ThemeDataInterface[]): ThemeDataInterface[] {
    return themes.filter(theme => theme.removedCommit !== undefined).sort((a, b) => a.id.localeCompare(b.id));
}

export function getRecentRetiredThemes(themes: ThemeDataInterface[]): ThemeDataInterface[] {
    return themes
        .filter(theme => theme.removedCommit !== undefined)
        .sort((a, b) => new Date(b.removedCommit!.date).valueOf() - new Date(a.removedCommit!.date).valueOf())
        .slice(0, 15);
}

export function getPercentageOfRetiredThemesByReleaseMonth(themes: ThemeDataInterface[]): PerMonthDataPoint[] {
    const data: PerMonthDataPoint[] = [];

    const firstReleaseDate = new Date(themes[0].addedCommit.date);
    const lastReleaseDate = new Date(themes[themes.length - 1].addedCommit.date);

    for (let d = firstReleaseDate; d <= lastReleaseDate; d.setMonth(d.getMonth() + 1)) {
        const year = d.getFullYear().toString();
        const month = (d.getMonth() + 1).toString().padStart(2, '0');

        const releasedThemes = themes.filter(theme => {
            const releaseDate = new Date(theme.addedCommit.date);
            return releaseDate.getFullYear() === d.getFullYear() && releaseDate.getMonth() === d.getMonth();
        });

        const retiredThemes = releasedThemes.filter(plugin => plugin.removedCommit !== undefined);

        const percentage = (retiredThemes.length / releasedThemes.length) * 100;

        data.push({
            year: year,
            month: month,
            value: percentage,
        });
    }

    return data;
}

export function getNewThemeReleasesPerMonth(themes: ThemeDataInterface[]): PerMonthDataPoint[] {
    const data: PerMonthDataPoint[] = [];

    const firstReleaseDate = new Date(themes[0].addedCommit.date);
    const lastReleaseDate = new Date(themes[themes.length - 1].addedCommit.date);

    for (let d = firstReleaseDate; d <= lastReleaseDate; d.setMonth(d.getMonth() + 1)) {
        const year = d.getFullYear().toString();
        const month = (d.getMonth() + 1).toString().padStart(2, '0');

        const releasedThemes = themes.filter(theme => {
            const releaseDate = new Date(theme.addedCommit.date);
            return releaseDate.getFullYear() === d.getFullYear() && releaseDate.getMonth() === d.getMonth();
        });

        data.push({
            year: year,
            month: month,
            value: releasedThemes.length,
        });
    }

    return data;
}

export function getThemeCountPerMonth(themes: ThemeDataInterface[]): PerMonthDataPoint[] {
    const data: PerMonthDataPoint[] = [];

    const firstReleaseDate = new Date(themes[0].addedCommit.date);
    const lastReleaseDate = new Date(themes[themes.length - 1].addedCommit.date);

    let totalThemes = 0;

    for (let d = firstReleaseDate; d <= lastReleaseDate; d.setMonth(d.getMonth() + 1)) {
        const year = d.getFullYear().toString();
        const month = (d.getMonth() + 1).toString().padStart(2, '0');

        const releasedThemes = themes.filter(theme => {
            const releaseDate = new Date(theme.addedCommit.date);
            return releaseDate.getFullYear() === d.getFullYear() && releaseDate.getMonth() === d.getMonth();
        });

        totalThemes += releasedThemes.length;

        data.push({
            year: year,
            month: month,
            value: totalThemes,
        });
    }

    return data;
}

export function getThemeCountPerMonthWoRetiredThemes(themes: ThemeDataInterface[]): PerMonthDataPoint[] {
    const data: PerMonthDataPoint[] = [];

    const firstReleaseDate = new Date(themes[0].addedCommit.date);
    const lastReleaseDate = new Date(themes[themes.length - 1].addedCommit.date);

    let totalThemes = 0;

    for (let d = firstReleaseDate; d <= lastReleaseDate; d.setMonth(d.getMonth() + 1)) {
        const year = d.getFullYear().toString();
        const month = (d.getMonth() + 1).toString().padStart(2, '0');

        const releasedThemes = themes.filter(theme => {
            const releaseDate = new Date(theme.addedCommit.date);
            return releaseDate.getFullYear() === d.getFullYear() && releaseDate.getMonth() === d.getMonth();
        });

        const retiredThemes = themes
            .filter(theme => theme.removedCommit !== undefined)
            .filter(theme => {
                const removedDate = new Date(theme.removedCommit!.date);
                return removedDate.getFullYear() === d.getFullYear() && removedDate.getMonth() === d.getMonth();
            });

        totalThemes += releasedThemes.length - retiredThemes.length;

        data.push({
            year: year,
            month: month,
            value: totalThemes,
        });
    }

    return data;
}

export function getRetiredThemeCountPerMonth(themes: ThemeDataInterface[]): PerMonthDataPoint[] {
    const data: PerMonthDataPoint[] = [];

    const firstReleaseDate = new Date(themes[0].addedCommit.date);
    const lastReleaseDate = new Date(themes[themes.length - 1].addedCommit.date);

    let totalRetiredThemes = 0;

    for (let d = firstReleaseDate; d <= lastReleaseDate; d.setMonth(d.getMonth() + 1)) {
        const year = d.getFullYear().toString();
        const month = (d.getMonth() + 1).toString().padStart(2, '0');

        const retiredThemes = themes
            .filter(theme => theme.removedCommit !== undefined)
            .filter(theme => {
                const removedDate = new Date(theme.removedCommit!.date);
                return removedDate.getFullYear() === d.getFullYear() && removedDate.getMonth() === d.getMonth();
            });

        totalRetiredThemes += retiredThemes.length;

        data.push({
            year: year,
            month: month,
            value: totalRetiredThemes,
        });
    }

    return data;
}

export function getRetiredThemesPerMonth(themes: ThemeDataInterface[]): PerMonthDataPoint[] {
    const data: PerMonthDataPoint[] = [];

    const firstReleaseDate = new Date(themes[0].addedCommit.date);
    const lastReleaseDate = new Date(themes[themes.length - 1].addedCommit.date);

    for (let d = firstReleaseDate; d <= lastReleaseDate; d.setMonth(d.getMonth() + 1)) {
        const year = d.getFullYear().toString();
        const month = (d.getMonth() + 1).toString().padStart(2, '0');

        const retiredThemes = themes
            .filter(theme => theme.removedCommit !== undefined)
            .filter(theme => {
                const removedDate = new Date(theme.removedCommit!.date);
                return removedDate.getFullYear() === d.getFullYear() && removedDate.getMonth() === d.getMonth();
            });

        data.push({
            year: year,
            month: month,
            value: retiredThemes.length,
        });
    }

    return data;
}
