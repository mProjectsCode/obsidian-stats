<script lang="ts">
	import { Dot, GridY, Plot, RegressionY } from 'svelteplot';
	import ChartWrapper from '../../ChartWrapper.svelte';

	interface Props {
		dataPoints: number[];
	}

	const { dataPoints }: Props = $props();

	const mappedData = dataPoints.map((downloads, index) => {
		return {
			index: index,
			downloads: downloads,
		};
	});
</script>

<ChartWrapper>
	<Plot x={{ label: 'Alphabetical Order →', ticks: [] }} y={{ label: '↑ Downloads', type: 'log', domain: [10, 10_000_000] }}>
		<GridY />
		<Dot data={mappedData} x="index" y="downloads" opacity={0.3} />
		<RegressionY data={mappedData} x="index" y="downloads" stroke="var(--sl-color-text-accent)" />
	</Plot>
</ChartWrapper>
