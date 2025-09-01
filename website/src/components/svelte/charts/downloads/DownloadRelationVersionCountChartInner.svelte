<script lang="ts">
	import { Dot, Plot, Pointer, Text } from 'svelteplot';

	interface Props {
		dataPoints: {
			id: string;
			name: string;
			downloads: number;
			version_count: number;
		}[];
	}

	const { dataPoints }: Props = $props();
</script>

<Plot grid x={{ label: 'Releases →', type: 'log' }} y={{ label: '↑ Downloads', type: 'log', domain: [1, 10_000_000] }} class="no-overflow-clip">
	<Dot data={dataPoints} x="version_count" y="downloads" opacity={0.3} stroke="var(--sl-color-text-accent)" />
	<Pointer data={dataPoints} x="version_count" y="downloads" maxDistance={30}>
		{#snippet children({ data })}
			<Text {data} fill="var(--sl-color-text-accent)" x="version_count" y="downloads" text={d => d.name} lineAnchor="bottom" dy={-7} />
			<Dot {data} x="version_count" y="downloads" fill="var(--sl-color-text-accent)" />
		{/snippet}
	</Pointer>
</Plot>
