import { Commit } from '../types';

export interface PluginRepoData {
	id: string;
	repo: PluginRepoExtractedData | undefined;
	warnings: PluginWarning[];
	removalReason: string | undefined;
	deprecatedVersions: string[];
}

export interface PluginRepoExtractedData {
	usesTypescript: boolean;
	hasPackageJson: boolean;
	packageManager: string | undefined;
	dependencies: string[];
	devDependencies: string[];
	installedTestingFrameworks: string[];
	installedBundlers: string[];
	hasTestFiles: boolean;
	hasBetaManifest: boolean;
	fileCounts: Record<string, number>;
	license: string;
	licenseFile: string;
	manifest: PluginManifest;
}

export interface PluginManifest {
	author: string;
	minAppVersion: string;
	name: string;
	version: string;
	authorUrl?: string;
	fundingUrl?: string | Record<string, string>;

	description: string;
	id: string;
	isDesktopOnly: boolean;

	// Non standard fields
	helpUrl?: string;
}

export enum PluginWarningSeverity {
	CAUTION = 'caution',
	DANGER = 'danger',
}

export interface PluginWarningInactivity6Months {
	severity: PluginWarningSeverity;
	id: 'inactivity-12-months';
	lastReleaseDate: string;
}

export interface PluginWarningInactivity12Months {
	severity: PluginWarningSeverity;
	id: 'inactivity-24-months';
	lastReleaseDate: string;
}

export interface PluginWarningRemoved {
	severity: PluginWarningSeverity;
	id: 'removed';
	commit: Commit;
}

export interface PluginWarningMismatchedManifestData {
	severity: PluginWarningSeverity;
	id: 'mismatched-manifest-data';
	data: {
		field: string;
		manifestValue: string;
		communityListValue: string;
	}[];
}

export interface PluginWarningUnlicensed {
	severity: PluginWarningSeverity;
	id: 'unlicensed';
}

export interface PluginWarningNoLicense {
	severity: PluginWarningSeverity;
	id: 'no-license';
}

export interface PluginWarningMismatchedLicense {
	severity: PluginWarningSeverity;
	id: 'mismatched-license';
}

export type PluginWarning =
	| PluginWarningInactivity12Months
	| PluginWarningInactivity6Months
	| PluginWarningRemoved
	| PluginWarningMismatchedManifestData
	| PluginWarningUnlicensed
	| PluginWarningNoLicense
	| PluginWarningMismatchedLicense;

type NonNullableFields<T> = {
	[P in keyof T]: NonNullable<T[P]>;
};

type WithRequired<Type, Key extends keyof Type> = Omit<Type, Key> & NonNullableFields<Pick<Type, Key>>;

export type PluginRepoDataNonNull = WithRequired<PluginRepoData, 'repo'>;
