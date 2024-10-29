<script lang="ts">
	import ThemeLink from '../helpers/themeLink.svelte';
	import GithubLink from '../helpers/githubLink.svelte';
	import Commit from '../helpers/commit.svelte';
	import { onMount } from 'svelte';
	import type { ThemeDataInterface } from '../../../../../src/theme/theme.ts';

	interface Props {
		data?: ThemeDataInterface[];
	}

	let { data = [] }: Props = $props();

	const nameSort = (x: ThemeDataInterface) => x.name.toLowerCase();
	const authorSort = (x: ThemeDataInterface) => x.currentEntry.author.toLowerCase();
	const repoSort = (x: ThemeDataInterface) => x.currentEntry.repo;
	const addedSort = (x: ThemeDataInterface) => x.addedCommit.date;
	const removedSort = (x: ThemeDataInterface) => (x.removedCommit?.date ? x.removedCommit.date : '');

	let sortCriteria: (x: ThemeDataInterface) => string = nameSort;

	let ascending = false;

	let sorted: ThemeDataInterface[] = $state([...data]);

	function sort(criteria: (x: ThemeDataInterface) => string): void {
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
			<th onclick={() => sort(nameSort)}>Name</th>
			<th onclick={() => sort(authorSort)}>Author</th>
			<th onclick={() => sort(repoSort)}>Repo</th>
			<th onclick={() => sort(addedSort)}>Added Date</th>
			<th onclick={() => sort(removedSort)}>Removed Date</th>
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
					>{#if theme.removedCommit}<Commit commit={theme.removedCommit}></Commit>{/if}</td
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
