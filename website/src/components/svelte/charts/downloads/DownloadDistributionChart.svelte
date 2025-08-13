<script lang="ts">
	import { Line, Plot } from 'svelteplot';
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
	<Plot grid x={{ label: 'Rank →', type: 'linear' }} y={{ label: '↑ Downloads', type: 'log', domain: [1, 10_000_000] }}>
		<Line data={mappedData} x="index" y="downloads" stroke="var(--sl-color-text-accent)" />
	</Plot>
</ChartWrapper>
