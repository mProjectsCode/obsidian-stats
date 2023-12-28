import {ALL_OS, WeeklyReleaseGrowthEntry} from "./release.ts";

import {addTableMethod, addVerb, escape, op, table} from "arquero";
import ColumnTable from "arquero/dist/types/table/column-table";

addTableMethod('distinctArray', (table, columnName: string) => {
    return table.rollup({ values: op.array_agg_distinct(columnName) }).get('values', 0);
}, { override: true });

addVerb('normalize', (table: ColumnTable, column: string) => {
    const sum = table.rollup({ sum: op.sum(column) }).get('sum', 0);
    return table.derive({ downloads: escape(d => d[column] / sum) });
}, [{ name: 'column', type: 'Expr' }], { override: true });

// Normalize by group
addVerb('normalizeBy', (table: ColumnTable, column: string, group: string) => {
    const sum = table.groupby(group).rollup({ sum: op.sum(column) }).orderby(group);
    return table
        .join(sum, [group])
        .derive({[column]: escape(d => d[column] / d['sum'])})
        .select(...table.columnNames())
        .impute({[column]: 0})
        .orderby(group);
}, [{ name: 'column', type: 'Expr' }, { name: 'group', type: 'Expr' }], { override: true });

addVerb("imputeAll", (tab: ColumnTable, columns: string[], nullable: string[]) => {
    const distinctValues = columns.map(column => table({[column]: tab.distinctArray(column)}));
    const allCombinations = distinctValues.reduce((acc, curr) => acc.cross(curr));

    return allCombinations
        .join_left(tab, columns.map(_ => columns))
        .impute(nullable.reduce((acc, curr) => ({...acc, [curr]: 0}), {}))
        .orderby(...columns);
}, [{ name: 'columns', type: 'ExprList' }, { name: 'nullable', type: 'ExprList' }], { override: true });

// | Distribution | OS | TYPE | IS | COMMENTS |
// | ---- | ---- | ---- | ---- | ---- |
// | obsidian-x.y.z.asar.gz | N/A | N/A | N/A | File downloaded by built-in updater |
// | Obsidian-x.y.z-universal.dmg | MacOS |  | x86/ARM |  |
// | Obsidian-x.y.z.AppImage | Linux |  | x86 |  |
// | Obsidian-x.y.z-arm64.AppImage | Linux |  | ARM |  |
// | obsidian-x.y.z-arm64.tar.gz | Linux |  | ARM |  |
// | obsidian_x.y.z_amd64.deb | Linux | Debian | x86 |  |
// | obsidian_x.y.z_amd64.snap | Linux | Snap | x86 |  |
// | obsidian-x.y.z.tar.gz | Linux |  | x86 |  |
// | obsidian-x.y.z-32.exe | Windows |  | x86-32 | Legacy 32bit |
// | obsidian-x.y.z-allusers.exe | Windows |  | x86 | Installed for all users |
// | obsidian-x.y.z.exe | Windows |  | x86 | Regular windows installer |
// | obsidian_x.y.z_arm64.exe | Windows |  | ARM |  |


export function determineAssetOS(assetName: string) {
    if (assetName.endsWith('.asar.gz')) {
        return null;
    } else if (assetName.endsWith('.dmg')) {
        return 'macos';
    } else if (assetName.endsWith('.exe')) {
        return 'windows';
    } else {
        return 'linux';
    }
}

export function determineAssetType(assetName: string) {
    const versionRegex = /\d+\.\d+\.\d+/;
    return assetName.slice(assetName.search(versionRegex) + versionRegex.exec(assetName)[0].length + 1);
}


export function determineAssetIS(assetName: string) {
    if (assetName.endsWith('.dmg'))
        return 'both';
    return assetName.includes('arm64') ? 'arm64' : 'x86';
}

export function determineAssetArchitecture(assetName: string) {
    if (assetName.includes('32'))
        return '32bit';
    else
        return '64bit';
}

export function fixVersion(version: string) {
    const parts = version.split(".");
    return parts.map((part) => part.padStart(2, "0")).join(".");
}


export function getMajorVersions(versions: string[]) {
    return [...new Set(versions.map(x => x.split('.').slice(0, 2).join('.')))].map(x => versions.find(y => y.startsWith(x))!);
}

export function parseReleaseGrowth(csv: string): WeeklyReleaseGrowthEntry[] {
    const lines = csv.split('\n').slice(1).map(x => x.split(','));
    return lines.map(x => ({
        date: new Date(x[0]),
        version: x[1],
        asset: x[2],
        downloads: parseInt(x[3])
    }));
}

export function getReleaseGrowthPerWeek(releaseData: WeeklyReleaseGrowthEntry[]) {
    // Group by week
    const groupedByWeek = new Map<number, WeeklyReleaseGrowthEntry[]>();

    for (const entry of releaseData) {
        if (groupedByWeek.has(entry.date.getTime())) {
            groupedByWeek.get(entry.date.getTime())!.push(entry);
        } else {
            groupedByWeek.set(entry.date.getTime(), [entry]);
        }
    }

    // Turn into array
    const groupedByWeekArray = Array.from(groupedByWeek.entries()).map(([date, entries]) => ({
        date: new Date(date),
        entries
    }));

    return groupedByWeekArray;
}

export function getReleaseGrowthPerWeekPerOS(releaseData: WeeklyReleaseGrowthEntry[]): { labels: string[], data: { [os in typeof ALL_OS[number]]: number[] } } {
    const releasesPerWeek = getReleaseGrowthPerWeek(releaseData);
    const uniqueWeeks = [...new Set(releasesPerWeek.map(x => x.date))];

    let releasesPerWeekPerOS: { date: Date, downloads: { [os in typeof ALL_OS[number]]: number } }[] = [];

    const datapoints: { [os in typeof ALL_OS[number]]: number[] } = Object.fromEntries(ALL_OS.map(x => [x, []]));


    for (const release of releasesPerWeek) {
        const downloadsPerOS = release.entries.reduce((acc, entry) => {
            const os = determineAssetOS(entry.asset);
            if (os !== null)
                acc[os] += entry.downloads;
            return acc;
        }, Object.fromEntries(ALL_OS.map(x => [x, 0])));

        for (const os of ALL_OS)
            datapoints[os].push(downloadsPerOS[os]);
    }

    const weeks = uniqueWeeks.map(x => x.toISOString().split('T')[0].slice(0, 7));
    return {
        data: datapoints,
        labels: weeks
    }
}
