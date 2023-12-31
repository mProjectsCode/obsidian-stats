<script lang="ts">
	import type { PluginDataInterface } from '../../../../../src/plugin/data.ts';
	import PluginLink from '../helpers/pluginLink.svelte';
	import GithubLink from '../helpers/githubLink.svelte';
	import Commit from '../helpers/commit.svelte';
	import { onMount } from 'svelte';

	export let data: PluginDataInterface[] = [];

	const idSort = (x: PluginDataInterface) => x.id.toLowerCase();
	const nameSort = (x: PluginDataInterface) => x.currentEntry.name.toLowerCase();
	const authorSort = (x: PluginDataInterface) => x.currentEntry.author.toLowerCase();
	const repoSort = (x: PluginDataInterface) => x.currentEntry.repo;
	const addedSort = (x: PluginDataInterface) => x.addedCommit.date;
	const removedSort = (x: PluginDataInterface) => (x.removedCommit?.date ? x.removedCommit.date : '');

	let sortByAccessor: (x: PluginDataInterface) => unknown = idSort;

	let ascending = false;

	let sorted: PluginDataInterface[] = [...data];

	function sort(accessor: (x: PluginDataInterface) => unknown): void {
		if (accessor === sortByAccessor) {
			ascending = !ascending;
		} else {
			sortByAccessor = accessor;
			ascending = true;
		}

		const sortModifier = ascending ? 1 : -1;
		sorted = sorted.sort((a, b) => {
			const _a = sortByAccessor(a);
			const _b = sortByAccessor(b);

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
		sort(sortByAccessor);
	});
</script>

<table>
	<thead>
		<tr>
			<th on:click={() => sort(idSort)}>Id</th>
			<th on:click={() => sort(nameSort)}>Name</th>
			<th on:click={() => sort(authorSort)}>Author</th>
			<th on:click={() => sort(repoSort)}>Repo</th>
			<th on:click={() => sort(addedSort)}>Added Date</th>
			<th on:click={() => sort(removedSort)}>Removed Date</th>
		</tr>
	</thead>
	<tbody>
		{#each sorted as plugin (plugin.id)}
			<tr>
				<td><PluginLink id={plugin.id}></PluginLink></td>
				<td>{plugin.currentEntry.name}</td>
				<td>{plugin.currentEntry.author}</td>
				<td><GithubLink repo={plugin.currentEntry.repo}></GithubLink></td>
				<td><Commit commit={plugin.addedCommit}></Commit></td>
				<td
					>{#if plugin.removedCommit}
						<Commit commit={plugin.removedCommit}></Commit>{/if}</td
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
