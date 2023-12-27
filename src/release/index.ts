import {GithubReleaseEntry, GithubReleases, ReleaseAsset, ReleaseEntry} from "./release.ts";
import {RELEASE_FULL_DATA_PATH, RELEASE_WEEKLY_DATA_PATH, RELEASE_STATS_URL, THEME_DATA_PATH} from "../constants.ts";
import {dateDiffInDays, dateToString, distributeValueEqually, getNextMondays} from "../utils.ts";


async function fetchReleaseStats() {
    const releases: GithubReleaseEntry[] = [];
    let currentPage: string | null = RELEASE_STATS_URL;
    while (currentPage) {
        // TODO: Investigate how authentication can be securely added here (for less risk of getting rate limited)
        const response = await fetch(currentPage) as Response;

        if (!response.ok) {
            throw new Error("Error while fetching releases data: " + (await response.json() as any).message)
        }

        const releasesPage= await response.json() as GithubReleaseEntry[];

        releases.push(...releasesPage);

        let nextLink: string | null = null;
        const link = response.headers.get("link");
        if (link) {
            const nextLinkSearch = link.split(",").find((link) => link.includes("rel=\"next\""));
            if (nextLinkSearch) {
                const nextLinkMatch = nextLinkSearch.match(/<(.+)>/);
                if (nextLinkMatch)
                    nextLink = nextLinkMatch[1];
            }
        }

        currentPage = nextLink;
    }



    const releasesData: ReleaseEntry[] = [];
    for (const release of releases) {
        const assets: ReleaseAsset[] = [];
        for (const asset of release.assets) {
            assets.push({
                name: asset.name,
                download_count: asset.download_count,
                size: asset.size
            });
        }

        releasesData.push({
            version: release.tag_name,
            date: new Date(release.published_at),
            assets: assets
        });
    }

    releasesData.sort((a, b) => a.date.getTime() - b.date.getTime());

    return releasesData;
}

async function computeWeeklyDownloads(prev: ReleaseEntry[], current: ReleaseEntry[], startDate: Date, endDate: Date) {
    // Get all mondays at UTC midnight between the two dates
    const dates = getNextMondays(startDate, endDate);

    //  Equally divide the downloadso over the given interval
    const startWeekCover = (dates[0].getTime() - startDate.getTime()) / (7 * 86400000);
    const endWeekCover = 1 - (dates[dates.length - 1].getTime() - endDate.getTime()) / (7 * 86400000);

    const totalWeekCover = startWeekCover + dates.length - 2 + endWeekCover;
    const factors = [
        startWeekCover / totalWeekCover,
        ...Array.from({length: dates.length - 2}, () => 1 / totalWeekCover),
        endWeekCover / totalWeekCover
    ];

    const prevAssets = prev.flatMap(x => x.assets.map(y => ({...y, version: x.version})));
    const currentAssets = current.flatMap(x => x.assets.map(y => ({...y, version: x.version})));


    // Create weekly downloads array for each day between releases
    let weeklyDownloads: { date: string, version: string, asset: string, downloads: number }[][] = Array.from({length: dates.length}, () => []);

    for (const asset of currentAssets) {
        const prevAsset = prevAssets.find(x => x.name === asset.name && x.version === asset.version);
        let previousDownloadCount = prevAsset ? prevAsset.download_count : 0;

        const assetDownloads = asset.download_count - previousDownloadCount;
        const distributedValues = distributeValueEqually(assetDownloads, factors);

        for (let i = 0; i < dates.length; i++) {
            weeklyDownloads[i].push({
                date: dateToString(dates[i]),
                version: asset.version,
                asset: asset.name,
                downloads: distributedValues[i]
            });
        }
    }

    const flattenedWeeklyDownloads = weeklyDownloads.flat();
    return flattenedWeeklyDownloads.filter(x => x.downloads > 0);
}



export async function testWeeklyDownloads() {
    const previousFullDataFile = Bun.file("releases-prev-data.json");
    const previousReleaseData = JSON.parse(await previousFullDataFile.text()) as ReleaseEntry[];

    const currentFullDataFile = Bun.file("releases-full-data.json");
    const currentReleaseData = JSON.parse(await currentFullDataFile.text()) as ReleaseEntry[];

    const endDate = new Date();
    const previousDate = new Date(endDate.getTime() - 0 * 86400000);

    const weeklyDownloads = await computeWeeklyDownloads(previousReleaseData, currentReleaseData, previousDate, endDate);
    const weeklyDownloadsFile = Bun.file(RELEASE_WEEKLY_DATA_PATH);
    let weeklyDownloadsText = await weeklyDownloadsFile.text();

    weeklyDownloadsText = combineWeeklyDownloads(weeklyDownloadsText, weeklyDownloads);

    await Bun.write(weeklyDownloadsFile, weeklyDownloadsText);
}



function combineWeeklyDownloads(prev: string, curr: { date: string, version: string, asset: string, downloads: number }[]) {
    let splitWeeklyDownloads = prev.split("\n");
    if (splitWeeklyDownloads[0] === "date,version,asset,downloads")
        splitWeeklyDownloads = splitWeeklyDownloads.slice(1);
    if (splitWeeklyDownloads[splitWeeklyDownloads.length - 1] === "")
        splitWeeklyDownloads = splitWeeklyDownloads.slice(0, -1);

    const previousWeeklyDownloads: Map<string, number> = new Map(
        splitWeeklyDownloads
            .map(x => {
                const [date, version, asset, downloads] = x.split(",");
                return [`${date},${version},${asset}`, parseInt(downloads)];
            })
    );

    for (const download of curr) {
        const key = `${download.date},${download.version},${download.asset}`;
        if (previousWeeklyDownloads.has(key)) {
            previousWeeklyDownloads.set(key, previousWeeklyDownloads.get(key)! + download.downloads);
        } else {
            previousWeeklyDownloads.set(key, download.downloads);
        }
    }



    // Generate csv text by just adding downloads to key
    return "date,version,asset,downloads\n" +
        Array.from(previousWeeklyDownloads.entries())
            .map(([key, downloads]) => `${key},${downloads}`)
            .join("\n");
}


export async function buildReleaseStats() {
    const releaseData = await fetchReleaseStats();

    const releaseFullDataFile = Bun.file(RELEASE_FULL_DATA_PATH);

    const lastModifiedDate = new Date(releaseFullDataFile.lastModified);
    const previousReleaseData = JSON.parse(await releaseFullDataFile.text()) as ReleaseEntry[];
    const endDate = new Date();

    let weeklyDownloads = await computeWeeklyDownloads(previousReleaseData, releaseData, lastModifiedDate, endDate);

    const weeklyDownloadsFile = Bun.file(RELEASE_WEEKLY_DATA_PATH);
    const weeklyDownloadsText = combineWeeklyDownloads(await weeklyDownloadsFile.text(), weeklyDownloads);

    await Bun.write(weeklyDownloadsFile, weeklyDownloadsText);

    await Bun.write(releaseFullDataFile, JSON.stringify(releaseData, null, 4));
}

await buildReleaseStats();
// await testWeeklyDownloads();
