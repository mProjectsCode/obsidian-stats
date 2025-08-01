<script lang="ts">
	import { onMount } from 'svelte';
	import type { OverviewDataPoint } from '../../../../data-wasm/pkg/data_wasm';
	import type { ItemType } from '../../utils/misc';

	interface Props {
		data?: OverviewDataPoint[];
		type: ItemType;
	}

	let { data = [], type }: Props = $props();

	const idSort = (x: OverviewDataPoint) => x.id.toLowerCase();
	const nameSort = (x: OverviewDataPoint) => x.name.toLowerCase();
	const authorSort = (x: OverviewDataPoint) => x.author.toLowerCase();
	const repoSort = (x: OverviewDataPoint) => x.repo;
	const addedSort = (x: OverviewDataPoint) => x.added_commit.date;
	const removedSort = (x: OverviewDataPoint) => (x.removed_commit?.date ? x.removed_commit.date : '');

	let sortCriteria: (x: OverviewDataPoint) => string = idSort;

	let ascending = false;

	let sorted: OverviewDataPoint[] = $state([...data]);

	function sort(criteria: (x: OverviewDataPoint) => string): void {
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
		{#each sorted as datum (datum.id)}
			<tr>
				<td>
					{#if type === 'plugin'}
						<a href={'/obsidian-stats/plugins/' + datum.id}>{datum.id}</a>
					{:else}
						<a href={'/obsidian-stats/themes/' + datum.id}>{datum.name}</a>
					{/if}
				</td>
				<td>{datum.name}</td>
				<td>{datum.author}</td>
				<td>
					<a href={'https://github.com/' + datum.repo} target="_blank">{datum.repo}</a>
				</td>
				<td>
					<a href={'https://github.com/obsidianmd/obsidian-releases/commit/' + datum.added_commit.hash} target="_blank">{datum.added_commit.date}</a>
				</td>
				<td>
					{#if datum.removed_commit}
						<a href={'https://github.com/obsidianmd/obsidian-releases/commit/' + datum.removed_commit.hash} target="_blank">{datum.removed_commit.date}</a>
					{/if}
				</td>
			</tr>
		{/each}
	</tbody>
</table>

<style>
	th {
		cursor: pointer;
	}
</style>
