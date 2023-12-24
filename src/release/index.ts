import {GithubReleaseEntry, GithubReleases, ReleaseAsset, ReleaseEntry} from "./release.ts";
import {RELEASE_FULL_DATA_PATH, RELEASE_DAILY_DATA_PATH, RELEASE_STATS_URL, THEME_DATA_PATH} from "../constants.ts";


async function fetchReleaseStats() {
    const releases: GithubReleaseEntry[] = [];
    let currentPage: string | null = RELEASE_STATS_URL;
    while (currentPage) {
        const response = await fetch(currentPage, {
            headers: {
                "Authorization": `token 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`
            }
        }) as Response;

        if (!response.ok) {
            throw new Error("Error while fetching releases data: " + (await response.json() as any).message)
        }

        const releasesPage= await response.json() as GithubReleaseEntry[];

        releases.push(...releasesPage);

        let nextLink: string | null = null;
        const link = response.headers.get("link");
        // if (link) {
        //     const nextLinkSearch = link.split(",").find((link) => link.includes("rel=\"next\""));
        //     if (nextLinkSearch) {
        //         const nextLinkMatch = nextLinkSearch.match(/<(.+)>/);
        //         if (nextLinkMatch)
        //             nextLink = nextLinkMatch[1];
        //     }
        // }

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

    return releasesData;
}

async function computeDailyDownloads(prev: ReleaseEntry[], current: ReleaseEntry[], prevDate: Date, currentDate: Date) {

     // Get duration between releases in ~24h intervals
    const duration = Math.round((currentDate.getTime() - prevDate.getTime()) / 86400000);
    const date = new Date(prevDate.getTime());

    const dates: string[] = Array.from({length: duration}, (_, i) => {
        date.setDate(prevDate.getDate() + i + 1);
        return date.toISOString().split("T")[0];
    });


    const prevAssets = prev.flatMap(x => x.assets.map(y => ({...y, version: x.version})));
    const currentAssets = current.flatMap(x => x.assets.map(y => ({...y, version: x.version})));


    // Create daily downloads array for each day between releases
    let dailyDownloads: { date: string, version: string, asset: string, downloads: number }[][] = Array.from({length: duration}, () => []);

    for (const asset of currentAssets) {
        const prevAsset = prevAssets.find(x => x.name === asset.name && x.version === asset.version);
        let previousDownloadCount = prevAsset ? prevAsset.download_count : 0;
        const downloadsPerDay = Math.round(asset.download_count - previousDownloadCount / duration);

        for (let i = 0; i < duration; i++) {
            dailyDownloads[i].push({date: dates[i], version: asset.version, asset: asset.name, downloads: downloadsPerDay});
        }
    }

    const flattenedDailyDownloads = dailyDownloads.flat();
    return flattenedDailyDownloads.filter(x => x.downloads > 0);
}



export async function testDailyDownloads() {
    const previousFullDataFile = Bun.file("releases-prev-data.json");
    const previousReleaseData = JSON.parse(await previousFullDataFile.text()) as ReleaseEntry[];

    const currentFullDataFile = Bun.file("releases-full-data.json");
    const currentReleaseData = JSON.parse(await currentFullDataFile.text()) as ReleaseEntry[];

    const currentDate = new Date();
    const previousDate = new Date(currentDate.getTime() - 2 * 86400000);

    const dailyDownloads = await computeDailyDownloads(previousReleaseData, currentReleaseData, previousDate, currentDate);
    console.log(dailyDownloads);
}


export async function buildReleaseStats() {
    const releaseData = await fetchReleaseStats();

    const releaseFullDataFile = Bun.file(RELEASE_FULL_DATA_PATH);

    const lastModifiedDate = new Date(releaseFullDataFile.lastModified);
    const previousReleaseData = JSON.parse(await releaseFullDataFile.text()) as ReleaseEntry[];
    let currentDate = new Date();
    currentDate.setDate(currentDate.getDate() + 1);

    const dailyDownloads = await computeDailyDownloads(previousReleaseData, releaseData, lastModifiedDate, currentDate);


    const dailyDownloadsFile = Bun.file(RELEASE_DAILY_DATA_PATH);
    let dailyDownloadsText = await dailyDownloadsFile.text();
    dailyDownloadsText += dailyDownloads.map(x => `${x.date},${x.version},${x.asset},${x.downloads}`).join("\n");
    await Bun.write(dailyDownloadsFile, dailyDownloadsText);

    await Bun.write(releaseFullDataFile, JSON.stringify(releaseData, null, 4));
}

await buildReleaseStats();
