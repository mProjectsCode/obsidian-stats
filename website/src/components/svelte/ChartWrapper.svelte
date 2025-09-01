<script lang="ts" generics="T">
	import type { Snippet } from 'svelte';

	interface Props {
		children: Snippet<[T | undefined]>;
		dataUrl?: string | undefined;
	}

	const { children, dataUrl }: Props = $props();
</script>

{#if import.meta.env.SSR}
	<div class="chart-placeholder"></div>
{:else}
	{@const dataPromise = dataUrl ? fetch(dataUrl).then(res => res.json() as T) : Promise.resolve(undefined)}
	{#await dataPromise}
		<div class="chart-placeholder"></div>
	{:then data}
		<div class="chart-wrapper">
			{@render children?.(data)}
		</div>
	{/await}
{/if}
