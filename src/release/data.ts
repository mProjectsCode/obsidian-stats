import {ALL_OS, ReleaseAsset, ReleaseEntry} from "./release.ts";


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


function determineAssetOS(asset: ReleaseAsset) {
    const name = asset.name;

    if (name.endsWith('.asar.gz')) {
        return undefined;
    } else if (name.endsWith('.dmg')) {
        return 'macos';
    } else if (name.endsWith('.exe')) {
        return 'windows';
    } else {
        return 'linux';
    }
}


function getAllReleases(releaseData: ReleaseEntry[]) {
    // Returns all unique releases
    return [...new Set(releaseData.map(x => x.version))];
}


export function getDownloadsPerOS(releaseData: ReleaseEntry[]) {
    const downloadsPerOS = Object.fromEntries(ALL_OS.map(x =>
        [x, Object.fromEntries(getAllReleases(releaseData).map(x => [x, 0]))])
    );

    // Multiple assets can count towards the same OS
    for (const release of releaseData) {
        for (const asset of release.assets) {
            const os = determineAssetOS(asset);
            if (os !== undefined)
                downloadsPerOS[os][release.version] += asset.download_count;
        }
    }

    return downloadsPerOS;
}


export function getNormalisedDownloadsPerOS(releaseData: ReleaseEntry[]) {
    const downloadsPerOS = getDownloadsPerOS(releaseData);

    const releases = getAllReleases(releaseData);

    for (const release of releases) {
        const total = ALL_OS.reduce((acc, os) => acc + downloadsPerOS[os][release], 0);
        for (const os of ALL_OS) {
            downloadsPerOS[os][release] /= total;
            if (isNaN(downloadsPerOS[os][release]))
                downloadsPerOS[os][release] = 0;
        }
    }


    const test= downloadsPerOS as { [os in typeof ALL_OS[number]]: { [release: string]: number } };
    console.log(test);
    return test;
}
