// @ts-ignore
import project from 'virtual:starlight/project-context';
import type { PluginRepoData, PluginWarning } from '../../../src/pluginRepo/types';
import { promises as fs } from 'fs';
import { groupBy } from '../../../src/utils';
import { CDate } from '../../../src/date';
import type { PluginDataInterface } from '../../../src/plugin/plugin';

export const BASE_PATH = import.meta.url;
// console.log(BASE_PATH);

export function projectRelativeUrl(relativePath: string | URL): URL {
	return new URL(relativePath, project.root);
}

export async function getRepoData(): Promise<PluginRepoData[]> {
	const url = projectRelativeUrl('../pluginRepos/data/');
	const repoDataFiles = await fs.readdir(url);

	return await Promise.all(
		repoDataFiles.map(async file => {
			const content = await fs.readFile(new URL(`./${file}`, url), 'utf-8');
			return JSON.parse(content);
		}),
	);
}

export async function getPluginWarningPercentByReleaseMonth(plugins: PluginDataInterface[]) {
	const pluginDataGroupedByReleaseMonth = groupBy(plugins, x => {
		const date = CDate.fromString(x.addedCommit.date);
		return date.toMonthString();
	});

	const repoData = await getRepoData();
	const repoDataMap = new Map(repoData.map(x => [x.id, x]));

	return Object.entries(pluginDataGroupedByReleaseMonth)
		.map(([month, plugins]) => {
			let total = plugins.length;
			let warningCounts: Record<PluginWarning['id'], number> = {
				'inactivity-12-months': 0,
				'inactivity-24-months': 0,
				removed: 0,
				'mismatched-license': 0,
				'no-license': 0,
				unlicensed: 0,
				'mismatched-manifest-data': 0,
			};

			for (const plugin of plugins) {
				const repo = repoDataMap.get(plugin.id)!;

				for (const warning of repo.warnings) {
					warningCounts[warning.id] += 1;
				}
			}

			const warningPercent: Record<PluginWarning['id'], number> = Object.fromEntries(
				Object.entries(warningCounts).map(([key, value]) => {
					return [key, value / total];
				}),
			) as Record<PluginWarning['id'], number>;

			return {
				month,
				warningPercent,
			};
		})
		.sort((a, b) => a.month.localeCompare(b.month));
}
