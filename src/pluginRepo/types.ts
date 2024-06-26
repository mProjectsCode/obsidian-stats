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
}
