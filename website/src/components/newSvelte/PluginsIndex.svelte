<script lang="ts">
	import PluginLink from '../svelte/helpers/pluginLink.svelte';
	import GithubLink from '../svelte/helpers/githubLink.svelte';
	import Commit from '../svelte/helpers/commit.svelte';
	import { onMount } from 'svelte';
	import type { PluginOverviewDataPoint } from '../../../../data-wasm/pkg/data_wasm';

	interface Props {
		data?: PluginOverviewDataPoint[];
	}

	let { data = [] }: Props = $props();

	const idSort = (x: PluginOverviewDataPoint) => x.id.toLowerCase();
	const nameSort = (x: PluginOverviewDataPoint) => x.name.toLowerCase();
	const authorSort = (x: PluginOverviewDataPoint) => x.author.toLowerCase();
	const repoSort = (x: PluginOverviewDataPoint) => x.repo;
	const addedSort = (x: PluginOverviewDataPoint) => x.added_commit.date;
	const removedSort = (x: PluginOverviewDataPoint) => (x.removed_commit?.date ? x.removed_commit.date : '');

	let sortCriteria: (x: PluginOverviewDataPoint) => string = idSort;

	let ascending = false;

	let sorted: PluginOverviewDataPoint[] = $state([...data]);

	function sort(criteria: (x: PluginOverviewDataPoint) => string): void {
		if (criteria === sortCriteria) {
			ascending = !ascending;
		} else {
			sortCriteria = criteria;
			ascending = true;
		}

		const sortModifier = ascending ? 1 : -1;
		sorted = sorted.sort((a, b) => {
			const _a = sortCriteria(a);
			const _b = sortCriteria(b);

			if (_a < _b) {
				return -1 * sortModifier;
			} else if (_a > _b) {
				return 1 * sortModifier;
			} else {
				return 0;
			}
		});
	}

	onMount(() => {
		sort(sortCriteria);
	});
</script>

<table>
	<thead>
		<tr>
			<th onclick={() => sort(idSort)}>Id</th>
			<th onclick={() => sort(nameSort)}>Name</th>
			<th onclick={() => sort(authorSort)}>Author</th>
			<th onclick={() => sort(repoSort)}>Repo</th>
			<th onclick={() => sort(addedSort)}>Added Date</th>
			<th onclick={() => sort(removedSort)}>Removed Date</th>
		</tr>
	</thead>
	<tbody>
		{#each sorted as plugin (plugin.id)}
			<tr>
				<td><PluginLink id={plugin.id}></PluginLink></td>
				<td>{plugin.name}</td>
				<td>{plugin.author}</td>
				<td><GithubLink repo={plugin.repo}></GithubLink></td>
				<td><Commit commit={plugin.added_commit}></Commit></td>
				<td
					>{#if plugin.removed_commit}<Commit commit={plugin.removed_commit}></Commit>{/if}</td
				>
			</tr>
		{/each}
	</tbody>
</table>

<style>
	th {
		cursor: pointer;
	}
</style>
