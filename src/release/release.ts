export interface GithubUser {
	login: string;
	id: number;
	node_id: string;
	avatar_url: string;
	gravatar_id: string;
	url: string;
	html_url: string;
	followers_url: string;
	following_url: string;
	gist_url: string;
	starred_url: string;
	subscriptions_url: string;
	organizations_url: string;
	repos_url: string;
	events_url: string;
	received_events: string;
	type: string;
	site_admin: boolean;
}

export interface GithubReleaseAsset {
	url: string;
	browser_download_url: string;
	id: number;
	node_id: string;
	name: string;
	label: string;
	state: string;
	content_type: string;
	size: number;
	download_count: number;
	created_at: Date;
	updated_at: Date;
	uploader: GithubUser;
}

export interface GithubReleaseReactions {
	url: string;
	total_count: number;
	'+1': number;
	'-1': number;
	laugh: number;
	hooray: number;
	confused: number;
	heart: number;
	rocket: number;
	eyes: number;
}

export interface GithubReleaseEntry {
	url: string;
	assets_url: string;
	upload_url: string;
	html_url: string;
	id: number;
	author: GithubUser;
	node_id: string;
	tag_name: string;
	target_commitish: string;
	name: string;
	draft: boolean;
	prerelease: boolean;
	created_at: Date;
	published_at: Date;
	assets: GithubReleaseAsset[];
	tarball_url: string;
	zipball_url: string;
	body: string;
	reactions: GithubReleaseReactions;
}

export interface ObsidianReleaseInfo {
	version: string;
	platform: "desktop" | "mobile";
	insider: boolean;
	date: Date;
	info: string;
	major_release: boolean;
}

export type GithubReleases = GithubReleaseEntry[];

export interface ReleaseAsset {
	name: string;
	download_count: number;
	size: number;
}

export interface ReleaseEntry {
	version: string;
	date: Date;
	assets: ReleaseAsset[];
}

export interface WeeklyReleaseGrowthEntry {
	date: Date;
	version: string;
	asset: string;
	downloads: number;
}

export const ALL_OS = ['macos', 'windows', 'linux'] as const;
