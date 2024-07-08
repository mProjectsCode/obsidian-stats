export interface PluginRepoData {
	id: string;
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
	license: string | undefined;
	manifest: PluginManifest;
}

export interface PluginManifest {
	author: string;
	minAppVersion: string;
	name: string;
	version: string;
	authorUrl?: string;
	fundingUrl?: string;

	description: string;
	id: string;
	isDesktopOnly: boolean;

	// Non standard fields
	helpUrl?: string;
}