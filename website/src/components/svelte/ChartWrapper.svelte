<script lang="ts">
	const { children } = $props();
</script>

{#if import.meta.env.SSR}
	<div class="chart-placeholder"></div>
{:else}
	{@render children?.()}
{/if}
