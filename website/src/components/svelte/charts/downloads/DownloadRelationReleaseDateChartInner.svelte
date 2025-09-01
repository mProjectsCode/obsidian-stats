<script lang="ts">
	import { Dot, Plot, Pointer, Text } from 'svelteplot';

	interface Props {
		dataPoints: {
			id: string;
			name: string;
			downloads: number;
			date: string;
		}[];
	}

	const { dataPoints }: Props = $props();

	const mappedData = dataPoints.map(d => ({
		...d,
		date: new Date(d.date),
	}));
</script>

<Plot grid x={{ label: 'Release Date →' }} y={{ label: '↑ Downloads', type: 'log', domain: [1, 10_000_000] }} class="no-overflow-clip">
	<Dot data={mappedData} x="date" y="downloads" opacity={0.3} stroke="var(--sl-color-text-accent)" />
	<Pointer data={mappedData} x="date" y="downloads" maxDistance={30}>
		{#snippet children({ data })}
			<Text {data} fill="var(--sl-color-text-accent)" x="date" y="downloads" text={d => d.name} lineAnchor="bottom" dy={-7} />
			<Dot {data} x="date" y="downloads" fill="var(--sl-color-text-accent)" />
		{/snippet}
	</Pointer>
</Plot>
