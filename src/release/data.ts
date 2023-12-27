import {ALL_OS, ReleaseEntry, WeeklyReleaseGrowthEntry} from "./release.ts";


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


function determineAssetOS(assetName: string) {
    if (assetName.endsWith('.asar.gz')) {
        return undefined;
    } else if (assetName.endsWith('.dmg')) {
        return 'macos';
    } else if (assetName.endsWith('.exe')) {
        return 'windows';
    } else {
        return 'linux';
    }
}


function getAllReleaseVersions(releaseData: ReleaseEntry[]) {
    return [...new Set(releaseData.map(x => x.version))];
}


export function getMajorReleaseVersions(releaseData: ReleaseEntry[]) {
    const allVersions = getAllReleaseVersions(releaseData);
    const majorVersions = [...new Set(allVersions.map(x => x.split('.').slice(0, 2).join('.')))];
    return majorVersions.map(x => allVersions.find(y => y.startsWith(x))!);
}


export function getDownloadsPerOS(releaseData: ReleaseEntry[]) {
    const downloadsPerOS = Object.fromEntries(ALL_OS.map(x =>
        [x, Object.fromEntries(getAllReleaseVersions(releaseData).map(x => [x, 0]))])
    );

    // Multiple assets can count towards the same OS
    for (const release of releaseData) {
        for (const asset of release.assets) {
            const os = determineAssetOS(asset.name);
            if (os !== undefined)
                downloadsPerOS[os][release.version] += asset.download_count;
        }
    }

    return downloadsPerOS;
}


export function getNormalisedDownloadsPerOS(releaseData: ReleaseEntry[]) {
    const downloadsPerOS = getDownloadsPerOS(releaseData);

    const releases = getAllReleaseVersions(releaseData);

    for (const release of releases) {
        const total = ALL_OS.reduce((acc, os) => acc + downloadsPerOS[os][release], 0);
        for (const os of ALL_OS) {
            downloadsPerOS[os][release] /= total;
            if (isNaN(downloadsPerOS[os][release]))
                downloadsPerOS[os][release] = 0;
        }
    }

    return downloadsPerOS as { [os in typeof ALL_OS[number]]: { [release: string]: number } };
}

export function getDownloadSizeOverTime()


export function getTimeBetweenReleases(releaseData: ReleaseEntry[]) {
    const releases = getAllReleaseVersions(releaseData);
    const releaseDates = Object.fromEntries(releases.map(x => [x, new Date(releaseData.find(y => y.version === x)!.date)]));

    // Get time in ms between releases
    const timeBetweenReleases: { [release: string]: number } = {};
    for (let i = 1; i < releases.length; i++) {
        const prev = releases[i - 1];
        const curr = releases[i];
        timeBetweenReleases[curr] = releaseDates[curr].getTime() - releaseDates[prev].getTime();
    }

    return timeBetweenReleases;
}

export function getTimeBetweenMajorReleases(releaseData: ReleaseEntry[]) {
    const majorReleases = getMajorReleaseVersions(releaseData);
    const majorReleaseData = releaseData.filter(x => majorReleases.includes(x.version));
    return getTimeBetweenReleases(majorReleaseData);
}

export function parseReleaseGrowth(csv: string): WeeklyReleaseGrowthEntry[] {
    console.log(csv);
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
            if (os !== undefined)
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
