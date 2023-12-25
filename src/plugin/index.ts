import {$, Verboseness} from "../shellUtils.ts";
import {PluginData, PluginDownloadStats, PluginList} from "./plugin.ts";
import CliProgress from "cli-progress";
import {dateToString, dateDiffInDays, gitLogToCommits} from "../utils.ts";
import {Commit} from "../types.ts";


import {
    PLUGIN_LIST_PATH, PLUGIN_STATS_PATH, PLUGIN_DATA_PATH, OBSIDIAN_RELEASES_FULL_PATH, PLUGIN_TEMPLATE_FILE_PATH,
    PLUGIN_TEMPLATE_REPLACEMENT_STRING, PLUGIN_TEMPLATE_REPLACEMENT_STRING_JSON, PLUGIN_TEMPLATE_OUTPUT_PATH,
} from "../constants.ts";


async function getPluginListChanges(): Promise<Commit[]> {
    const pluginListChanges = await $(`git log --diff-filter=M --date-order --reverse --format="%ad %H" --date=iso-strict "${PLUGIN_LIST_PATH}"`, OBSIDIAN_RELEASES_FULL_PATH);

    return gitLogToCommits(pluginListChanges.stdout);
}

async function getPluginLists(): Promise<PluginList[]> {
    const pluginListChangeCommits = await getPluginListChanges();

    const pluginLists = (await Promise.all(pluginListChangeCommits.map(async (x, i) => {
        const pluginList = await $(`git cat-file -p ${x.hash}:${PLUGIN_LIST_PATH}`, OBSIDIAN_RELEASES_FULL_PATH, Verboseness.QUITET);
        try {
            const pluginListEntries = JSON.parse(pluginList.stdout);
            return new PluginList(pluginListEntries, x);
        } catch (e) {
            console.log(`Error parsing plugin list at commit ${x.hash}`);
            return undefined;
        }
    }))).filter(x => x !== undefined) as PluginList[];

    console.log(`Found ${pluginLists.length} version of "community-plugins.json"`);

    return pluginLists;
}

async function getPluginDownloadChanges(): Promise<Commit[]> {
    const pluginDownloadChanges = await $(`git log --diff-filter=M --date-order --reverse --format="%ad %H" --date=iso-strict "${PLUGIN_STATS_PATH}"`, OBSIDIAN_RELEASES_FULL_PATH);

    return gitLogToCommits(pluginDownloadChanges.stdout);
}


function buildPluginData(pluginLists: PluginList[]): PluginData[] {
    let pluginData: PluginData[] = [];

    console.log(`Building plugin data`);

    const progress = new CliProgress.SingleBar({}, CliProgress.Presets.rect);
    progress.start(pluginLists.length, 0);

    for (const entry of pluginLists[0].entries) {
        pluginData.push(new PluginData(entry.id, pluginLists[0].commit, entry));
    }

    for (let i = 1; i < pluginLists.length; i++) {
        progress.increment();

        const pluginList = pluginLists[i];

        for (const plugin of pluginData) {
            plugin.findChanges(pluginList);
        }

        for (const entry of pluginList.entries) {
            if (pluginData.find(x => x.id === entry.id) === undefined) {
                pluginData.push(new PluginData(entry.id, pluginList.commit, entry));
            }
        }
    }

    progress.stop();

    return pluginData;
}


function updateWeeklyDownloadStats(pluginData: PluginData[], pluginDownloadStats: PluginDownloadStats[]) {
    console.log(`Updating weekly download stats`);

    const startDate = new Date('2020-01-01');
    const endDate = new Date();

    startDate.setDate(startDate.getDate() + (7 - startDate.getDay()));

    const progress = new CliProgress.SingleBar({}, CliProgress.Presets.rect);
    progress.start(Math.ceil(dateDiffInDays(startDate, endDate) / 7), 0);

    for (let d = startDate; d <= endDate; d.setDate(d.getDate() + 7)) {
        const date = dateToString(d);
        progress.increment();

        let pluginDownload;

        for (let j = 0; j < 6; j++) {
            const currentDate = new Date(d);
            currentDate.setDate(currentDate.getDate() + j);
            const currentDateString = dateToString(currentDate);

            pluginDownload = pluginDownloadStats.find(x => dateToString(x.date) === currentDateString);
            if (pluginDownload !== undefined) break;
        }

        if (pluginDownload === undefined) {
            // console.log(`No plugin download stats found for ${date}`);
            continue;
        }

        for (const pluginDataEntry of pluginData) {
            pluginDataEntry.updateDownloadHistory(pluginDownload, date);
        }
    }

    progress.stop();
}


function updateVersionHistory(pluginData: PluginData[], pluginDownloadStats: PluginDownloadStats[]) {

    console.log(`Updating version history`);

    const progress = new CliProgress.SingleBar({}, CliProgress.Presets.rect);
    progress.start(pluginDownloadStats.length, 0);
    for (const pluginDownload of pluginDownloadStats) {
        progress.increment();
        for (const pluginDataEntry of pluginData) {
            pluginDataEntry.updateVersionHistory(pluginDownload);
        }
    }
    progress.stop();

    console.log(`Sorting Versions`);
    for (const pluginDataEntry of pluginData) {
        pluginDataEntry.sortVersionHistory();
    }
}


async function getPluginDownloadStats(): Promise<PluginDownloadStats[]> {
    const pluginDownloadChangeCommits = await getPluginDownloadChanges();

    const pluginDownloadStats = (await Promise.all(pluginDownloadChangeCommits.map(async (x, i) => {
        const pluginList = await $(`git cat-file -p ${x.hash}:${PLUGIN_STATS_PATH}`, OBSIDIAN_RELEASES_FULL_PATH, Verboseness.QUITET);
        try {
            const pluginDownloadStats = JSON.parse(pluginList.stdout);
            return new PluginDownloadStats(pluginDownloadStats, x);
        } catch (e) {
            console.log(`Error parsing plugin list at commit ${x.hash}`);
            return undefined;
        }
    }))).filter(x => x !== undefined) as PluginDownloadStats[];

    console.log(`Found ${pluginDownloadStats.length} versions of "community-plugin-stats.json"`);

    return pluginDownloadStats;
}

export async function buildPluginStats() {
    const pluginLists = await getPluginLists();
    let pluginData = buildPluginData(pluginLists);

    const pluginDownloadStats = await getPluginDownloadStats();
    updateWeeklyDownloadStats(pluginData, pluginDownloadStats);
    updateVersionHistory(pluginData, pluginDownloadStats);

    pluginData = pluginData.filter(x => x !== undefined);

    console.log(`Processed all plugins, writing to ${PLUGIN_DATA_PATH}`);

    const pluginDataFile = Bun.file(PLUGIN_DATA_PATH);
    await Bun.write(pluginDataFile, JSON.stringify(pluginData, null, 4));

    const templateFile = Bun.file(PLUGIN_TEMPLATE_FILE_PATH);
    const template = await templateFile.text();

    for (const plugin of pluginData) {
        let output = template.replaceAll(PLUGIN_TEMPLATE_REPLACEMENT_STRING, plugin.id);
        output = output.replaceAll(PLUGIN_TEMPLATE_REPLACEMENT_STRING_JSON, JSON.stringify(plugin));
        const outputFile = Bun.file(`${PLUGIN_TEMPLATE_OUTPUT_PATH}/${plugin.id}.mdx`);
        await Bun.write(outputFile, output);
    }
}

await buildPluginStats();
