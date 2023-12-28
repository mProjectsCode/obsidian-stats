<script lang="ts">
	import ThemeLink from './themeLink.svelte';
	import GithubLink from './githubLink.svelte';
	import Commit from './commit.svelte';
	import { onMount } from 'svelte';
	import type { ThemeDataInterface } from '../../../../src/theme/data.ts';
	import { utcStringToString } from '../../../../src/utils.ts';

	export let data: ThemeDataInterface[] = [];

	const nameSort = (x: ThemeDataInterface) => x.name.toLowerCase();
	const authorSort = (x: ThemeDataInterface) => x.currentEntry.author.toLowerCase();
	const repoSort = (x: ThemeDataInterface) => x.currentEntry.repo;
	const addedSort = (x: ThemeDataInterface) => utcStringToString(x.addedCommit.date);
	const removedSort = (x: ThemeDataInterface) => (x.removedCommit?.date ? utcStringToString(x.removedCommit.date) : '');

	let sortByAccessor: (x: ThemeDataInterface) => unknown = nameSort;

	let ascending = false;

	let sorted: ThemeDataInterface[] = [...data];

	function sort(accessor: (x: ThemeDataInterface) => unknown): void {
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
			<th on:click={() => sort(nameSort)}>Name</th>
			<th on:click={() => sort(authorSort)}>Author</th>
			<th on:click={() => sort(repoSort)}>Repo</th>
			<th on:click={() => sort(addedSort)}>Added Date</th>
			<th on:click={() => sort(removedSort)}>Removed Date</th>
		</tr>
	</thead>
	<tbody>
		{#each sorted as theme (theme.id)}
			<tr>
				<td><ThemeLink id={theme.id} name={theme.name}></ThemeLink></td>
				<td>{theme.currentEntry.author}</td>
				<td><GithubLink repo={theme.currentEntry.repo}></GithubLink></td>
				<td><Commit commit={theme.addedCommit}></Commit></td>
				<td
					>{#if theme.removedCommit}
						<Commit commit={theme.removedCommit}></Commit>{/if}</td
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
