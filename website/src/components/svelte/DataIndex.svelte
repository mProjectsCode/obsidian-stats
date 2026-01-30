<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import type { OverviewDataPoint } from '../../../../data-wasm/pkg/data_wasm';
	import type { ItemType } from '../../utils/misc';
	import VirtualTable from './VirtualTable.svelte';

	interface Props {
		data?: OverviewDataPoint[];
		type: ItemType;
	}

	const { data = [], type }: Props = $props();

	type SortBy = 'id' | 'name' | 'author' | 'repo' | 'added' | 'removed';

	let sortBy: SortBy = $state('id');
	let ascending = $state(false);
	let searchQuery = $state('');
	// svelte-ignore state_referenced_locally
	let filtered: OverviewDataPoint[] = $state([...data]);
	let worker: Worker | null = null;

	function initWorker(): void {
		worker = new Worker(new URL('../../workers/tableWorker.ts', import.meta.url), {
			type: 'module',
		});

		worker.onmessage = (e: MessageEvent) => {
			if (e.data.type === 'result') {
				// Build filtered array from indices
				filtered = e.data.indices.map((idx: number) => data[idx]);
			}
		};

		// Send data once during initialization
		worker.postMessage({
			type: 'init',
			data: $state.snapshot(data),
		});
	}

	function processData(): void {
		if (!worker) return;

		// Only send sort/filter criteria, not the data
		worker.postMessage({
			type: 'process',
			sortBy: sortBy,
			ascending: ascending,
			searchQuery: searchQuery,
		});
	}

	function sort(newSortBy: SortBy): void {
		if (newSortBy === sortBy) {
			ascending = !ascending;
		} else {
			sortBy = newSortBy;
			ascending = false;
		}
		processData();
	}

	function getSortIndicator(criteria: SortBy): string {
		if (criteria === sortBy) {
			return ascending ? ' ↑' : ' ↓';
		}
		return '';
	}

	$effect(() => {
		searchQuery;
		processData();
	});

	onMount(() => {
		initWorker();
		processData();
	});

	onDestroy(() => {
		worker?.terminate();
	});
</script>

<div class="table-controls">
	<input type="text" placeholder="Search... (e.g. name:tasks author:clare)" bind:value={searchQuery} />
	<span class="count">{filtered.length} of {data.length} items</span>
</div>

<VirtualTable items={filtered} colCount={6} colWidths={['16%', '22%', '18%', '22%', '11%', '11%']} minWidth={'72rem'} itemHeight={50} height={600}>
	{#snippet header()}
		<div
			class="vt-cell vt-header-cell"
			role="button"
			tabindex="0"
			onclick={() => sort('id')}
			onkeydown={e => (e.key === 'Enter' || e.key === ' ') && sort('id')}
		>
			Id{getSortIndicator('id')}
		</div>
		<div
			class="vt-cell vt-header-cell"
			role="button"
			tabindex="0"
			onclick={() => sort('name')}
			onkeydown={e => (e.key === 'Enter' || e.key === ' ') && sort('name')}
		>
			Name{getSortIndicator('name')}
		</div>
		<div
			class="vt-cell vt-header-cell"
			role="button"
			tabindex="0"
			onclick={() => sort('author')}
			onkeydown={e => (e.key === 'Enter' || e.key === ' ') && sort('author')}
		>
			Author{getSortIndicator('author')}
		</div>
		<div
			class="vt-cell vt-header-cell"
			role="button"
			tabindex="0"
			onclick={() => sort('repo')}
			onkeydown={e => (e.key === 'Enter' || e.key === ' ') && sort('repo')}
		>
			Repo{getSortIndicator('repo')}
		</div>
		<div
			class="vt-cell vt-header-cell"
			role="button"
			tabindex="0"
			onclick={() => sort('added')}
			onkeydown={e => (e.key === 'Enter' || e.key === ' ') && sort('added')}
		>
			Added Date{getSortIndicator('added')}
		</div>
		<div
			class="vt-cell vt-header-cell"
			role="button"
			tabindex="0"
			onclick={() => sort('removed')}
			onkeydown={e => (e.key === 'Enter' || e.key === ' ') && sort('removed')}
		>
			Removed Date{getSortIndicator('removed')}
		</div>
	{/snippet}

	{#snippet row(datum: OverviewDataPoint, index: number)}
		<div class="vt-cell">
			{#if type === 'plugin'}
				<a href={'/obsidian-stats/plugins/' + datum.id}>{datum.id}</a>
			{:else}
				<a href={'/obsidian-stats/themes/' + datum.id}>{datum.name}</a>
			{/if}
		</div>
		<div class="vt-cell"><span>{datum.name}</span></div>
		<div class="vt-cell"><span>{datum.author}</span></div>
		<div class="vt-cell">
			<a href={'https://github.com/' + datum.repo} target="_blank">{datum.repo}</a>
		</div>
		<div class="vt-cell">
			<a href={'https://github.com/obsidianmd/obsidian-releases/commit/' + datum.added_commit.hash} target="_blank">{datum.added_commit.date}</a>
		</div>
		<div class="vt-cell">
			{#if datum.removed_commit}
				<a href={'https://github.com/obsidianmd/obsidian-releases/commit/' + datum.removed_commit.hash} target="_blank">{datum.removed_commit.date}</a>
			{/if}
		</div>
	{/snippet}
</VirtualTable>

<style>
	.table-controls {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1rem;
		gap: 1rem;
	}

	.table-controls input {
		flex: 1;
		border: 1px solid var(--sl-color-gray-5);
		border-radius: 0.5rem;
		padding-inline-start: 0.75rem;
		padding-inline-end: 0.5rem;
		background-color: var(--sl-color-black);
		color: var(--sl-color-gray-2);
		font-size: var(--sl-text-sm);
		width: 100%;
		height: 40px;
	}

	.table-controls input::placeholder {
		color: var(--sl-color-gray-3);
	}

	.table-controls input:hover {
		border-color: var(--sl-color-gray-2);
		color: var(--sl-color-white);
	}

	.table-controls input:focus {
		outline: none;
		border-color: var(--sl-color-gray-2);
		color: var(--sl-color-white);
	}

	.count {
		white-space: nowrap;
		font-size: 0.9rem;
		opacity: 0.7;
	}

	.vt-header-cell {
		cursor: pointer;
		user-select: none;
		font-weight: 600;
	}
</style>
