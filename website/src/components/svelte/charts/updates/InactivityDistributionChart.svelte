<script lang="ts">
	import { GridY, Line, Plot } from 'svelteplot';
	import ChartWrapper from '../../ChartWrapper.svelte';

	interface Props {
		dataPoints: number[];
	}

	const { dataPoints }: Props = $props();

	const mappedDataPoints = dataPoints.map((inactivity, index) => ({
		inactivity,
		index: index / (dataPoints.length - 1),
	}));

	const maxInactivity = mappedDataPoints[0]?.inactivity || 0;

	const ticks: number[] = (() => {
		const ticks = [];
		for (let i = 0; i <= maxInactivity; i += 365) {
			ticks.push(i);
		}
		return ticks;
	})();

	function formatTick(tick: any): string {
		if (tick === 0) return '';
		const years = Math.floor(tick / 365);
		if (years === 1) return '1 year';
		return `${years} years`;
	}
</script>

<ChartWrapper>
	<Plot
		x={{ label: 'Distribution →', type: 'linear', percent: true }}
		y={{ label: '↑ Years of Inactivity', tickFormat: formatTick, ticks: ticks }}
		class="no-overflow-clip"
	>
		<GridY />
		<Line data={mappedDataPoints} x="index" y="inactivity" stroke="var(--sl-color-text-accent)" />
	</Plot>
</ChartWrapper>
