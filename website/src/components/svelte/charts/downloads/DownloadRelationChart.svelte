<script lang="ts">
	import { Dot, Plot } from 'svelteplot';
	import ChartWrapper from '../../ChartWrapper.svelte';

	interface Props {
		dataPoints: {
			date: Date;
			downloads: number;
			version_count: number;
			total_loc: number;
		}[];
	}

	const { dataPoints }: Props = $props();
</script>

<ChartWrapper>
	<Plot grid x={{ label: 'Releases →', type: 'log' }} y={{ label: '↑ Downloads', type: 'log', domain: [1, 10_000_000] }}>
		<Dot data={dataPoints} x="version_count" y="downloads" opacity={0.3} stroke="var(--sl-color-text-accent)" />
	</Plot>
</ChartWrapper>

<ChartWrapper>
	<Plot grid x={{ label: 'Total Lines of Code →', type: 'log' }} y={{ label: '↑ Downloads', type: 'log', domain: [1, 10_000_000] }}>
		<Dot data={dataPoints.filter(x => x.total_loc > 0)} x="total_loc" y="downloads" opacity={0.3} stroke="var(--sl-color-text-accent)" />
	</Plot>
</ChartWrapper>

<ChartWrapper>
	<Plot grid x={{ label: 'Release Date →' }} y={{ label: '↑ Downloads', type: 'log', domain: [1, 10_000_000] }}>
		<Dot data={dataPoints} x="date" y="downloads" opacity={0.3} stroke="var(--sl-color-text-accent)" />
	</Plot>
</ChartWrapper>
