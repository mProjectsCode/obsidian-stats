<script lang="ts">
	import { Line, Plot } from 'svelteplot';
	import ChartWrapper from '../../ChartWrapper.svelte';

	interface Props {
		dataPoints: number[];
	}

	const { dataPoints }: Props = $props();

	const mappedData = dataPoints.map((y, index) => {
		return {
			index: index,
			y: y,
		};
	});
</script>

<ChartWrapper>
	<Plot grid x={{ label: 'Rank →', type: 'linear' }} y={{ label: '↑ Lines of Code', type: 'log' }}>
		<Line data={mappedData} x="index" y="y" stroke="var(--sl-color-text-accent)" />
	</Plot>
</ChartWrapper>
