<script lang="ts">
	import type { PluginDataInterface } from '../../../../../src/plugin/plugin.ts';
	import PluginLink from '../helpers/pluginLink.svelte';
	import GithubLink from '../helpers/githubLink.svelte';
	import Commit from '../helpers/commit.svelte';
	import { onMount } from 'svelte';

	interface Props {
		data?: PluginDataInterface[];
	}

	let { data = [] }: Props = $props();

	const idSort = (x: PluginDataInterface) => x.id.toLowerCase();
	const nameSort = (x: PluginDataInterface) => x.currentEntry.name.toLowerCase();
	const authorSort = (x: PluginDataInterface) => x.currentEntry.author.toLowerCase();
	const repoSort = (x: PluginDataInterface) => x.currentEntry.repo;
	const addedSort = (x: PluginDataInterface) => x.addedCommit.date;
	const removedSort = (x: PluginDataInterface) => (x.removedCommit?.date ? x.removedCommit.date : '');

	let sortCriteria: (x: PluginDataInterface) => string = idSort;

	let ascending = false;

	let sorted: PluginDataInterface[] = $state([...data]);

	function sort(criteria: (x: PluginDataInterface) => string): void {
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
