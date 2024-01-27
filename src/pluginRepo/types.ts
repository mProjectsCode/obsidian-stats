export interface PluginRepoData {
	id: string,
	hasPackageJson: boolean,
	packageManager: string | undefined,
	dependencies: string[],
	devDependencies: string[],
	installedTestingFrameworks: string[],
	hasTestFiles: boolean,
	fileCounts: Record<string, number>,
}
