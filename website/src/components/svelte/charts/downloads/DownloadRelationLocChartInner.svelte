<script lang="ts">
	import { Dot, Plot, Pointer, Text } from 'svelteplot';

	type DataPoint = {
		id: string;
		name: string;
		downloads: number;
		total_loc: number;
	};

	interface Props {
		dataPoints: DataPoint[];
	}

	const { dataPoints }: Props = $props();
</script>

<Plot grid x={{ label: 'Total Lines of Code →', type: 'log' }} y={{ label: '↑ Downloads', type: 'log', domain: [1, 10_000_000] }} class="no-overflow-clip">
	<Dot data={dataPoints} x="total_loc" y="downloads" opacity={0.3} stroke="var(--sl-color-text-accent)" />
	<Pointer data={dataPoints} x="total_loc" y="downloads" maxDistance={30}>
		{#snippet children({ data })}
			<Text {data} fill="var(--sl-color-text-accent)" x="total_loc" y="downloads" text={d => d.name} lineAnchor="bottom" dy={-7} />
			<Dot {data} x="total_loc" y="downloads" fill="var(--sl-color-text-accent)" />
		{/snippet}
	</Pointer>
</Plot>
