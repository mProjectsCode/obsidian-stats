<script lang="ts">
	import { BarY, GridY, Plot } from 'svelteplot';
	import type { StackedNamedDataPoint } from '../../../../../data-wasm/pkg/data_wasm';
	import ChartWrapper from '../ChartWrapper.svelte';

	interface Props {
		dataPoints: StackedNamedDataPoint[];
		xLabel: string;
		yLabel: string;
		skewLabels?: boolean;
		percentages?: boolean;
		yDomain?: [number, number];
		xPadding?: number;
	}

	const { dataPoints, xLabel, yLabel, yDomain, skewLabels = false, percentages = false, xPadding = undefined }: Props = $props();

	const mappedDataPoints = dataPoints.map((point, index) => {
		return {
			index: index,
			label: point.name,
			value: point.value,
			stack: point.layer,
		};
	});
</script>

<ChartWrapper>
	<Plot
		color={{ legend: true, scheme: 'tableau10' }}
		x={{ type: 'band', label: `${xLabel} →`, tickRotate: skewLabels ? 45 : 0, padding: xPadding }}
		y={{ label: `↑ ${yLabel}`, domain: yDomain, tickFormat: percentages ? d => `${String(d)}%` : d => String(d) }}
		class="no-overflow-clip"
	>
		<GridY />
		<BarY data={mappedDataPoints} x="label" y="value" fill="stack" sort={{ channel: 'index' }} />
	</Plot>
</ChartWrapper>
