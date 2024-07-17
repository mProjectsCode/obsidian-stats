import { PluginDataInterface } from '../plugin/plugin';
import { PluginRepoExtractedData, PluginWarning, PluginWarningRemoved, PluginWarningSeverity } from './types';

export const warnings: ((plugin: PluginDataInterface, repo: PluginRepoExtractedData | undefined) => PluginWarning | undefined)[] = [
	inactivity,
	mismatchedData,
	license,
];

function inactivity(plugin: PluginDataInterface, repo: PluginRepoExtractedData | undefined): PluginWarning | undefined {
	if (plugin.removedCommit) {
		return {
			severity: PluginWarningSeverity.DANGER,
			id: 'removed',
			commit: plugin.removedCommit,
		};
	}

	const latestReleaseDateString = plugin.versionHistory.at(-1)?.initialReleaseDate ?? plugin.addedCommit.date;
	const latestReleaseDate = new Date(latestReleaseDateString);

	const outdatedDangerThreshold = new Date();
	outdatedDangerThreshold.setFullYear(outdatedDangerThreshold.getFullYear() - 1);
	const outdatedDanger = latestReleaseDate < outdatedDangerThreshold && !plugin.removedCommit;

	const outdatedWarningThreshold = new Date();
	outdatedDangerThreshold.setFullYear(outdatedDangerThreshold.getFullYear() - 2);
	const outdatedWarning = latestReleaseDate < outdatedWarningThreshold && !outdatedDanger && !plugin.removedCommit;

	if (outdatedDanger) {
		return {
			severity: PluginWarningSeverity.DANGER,
			id: 'inactivity-24-months',
			lastReleaseDate: latestReleaseDateString,
		};
	} else if (outdatedWarning) {
		return {
			severity: PluginWarningSeverity.CAUTION,
			id: 'inactivity-12-months',
			lastReleaseDate: latestReleaseDateString,
		};
	}
}

function mismatchedData(plugin: PluginDataInterface, repo: PluginRepoExtractedData | undefined): PluginWarning | undefined {
	if (repo) {
		const dataToCheck = [
			[plugin.currentEntry.name, repo.manifest.name, 'name'],
			[plugin.currentEntry.author, repo.manifest.author, 'author'],
			[plugin.currentEntry.description, repo.manifest.description, 'description'],
		];

		const mismatchedData = dataToCheck.filter(x => x[0] !== x[1]);

		if (mismatchedData.length > 0) {
			return {
				severity: PluginWarningSeverity.CAUTION,
				id: 'mismatched-manifest-data',
				data: mismatchedData.map(x => {
					return {
						field: x[2],
						manifestValue: x[1],
						communityListValue: x[0],
					};
				}),
			};
		}
	}
}

function license(plugin: PluginDataInterface, repo: PluginRepoExtractedData | undefined): PluginWarning | undefined {
	if (repo) {
		if (repo.license === 'explicitly unlicensed') {
			return {
				severity: PluginWarningSeverity.CAUTION,
				id: 'unlicensed',
			};
		}

		if (repo.license === 'no license') {
			return {
				severity: PluginWarningSeverity.CAUTION,
				id: 'no-license',
			};
		}
	}
}
