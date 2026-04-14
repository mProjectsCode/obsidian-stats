<script lang="ts">
	import { GridY, Line, Plot } from 'svelteplot';
	import ChartWrapper from '../../ChartWrapper.svelte';

	interface Props {
		dataPoints: number[];
	}

	const { dataPoints }: Props = $props();

	const mappedDataPoints = $derived.by(() => {
		if (dataPoints.length <= 1) {
			return dataPoints.map(score => ({ score, index: 0 }));
		}

		return dataPoints.map((score, index) => ({
			score,
			index: index / (dataPoints.length - 1),
		}));
	});
</script>

<ChartWrapper>
	<Plot
		x={{ label: 'Distribution →', type: 'linear', percent: true }}
		y={{ label: '↑ Minification Score', type: 'linear', domain: [0, 1] }}
		class="no-overflow-clip"
	>
		<GridY />
		<Line data={mappedDataPoints} x="index" y="score" stroke="var(--sl-color-text-accent)" />
	</Plot>
</ChartWrapper>
