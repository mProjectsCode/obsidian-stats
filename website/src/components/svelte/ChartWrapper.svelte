<script lang="ts">
	const { children } = $props();
</script>

{#if import.meta.env.SSR}
	<div class="chart-placeholder"></div>
{:else}
	<div class="chart-wrapper">
		{@render children?.()}
	</div>
{/if}
