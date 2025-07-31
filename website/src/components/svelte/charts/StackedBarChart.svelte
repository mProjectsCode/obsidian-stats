<script lang="ts">
	import { BarY, Plot } from 'svelteplot';
	import type { StackedNamedDataPoint } from '../../../../../data-wasm/pkg/data_wasm';

	interface Props {
		dataPoints: StackedNamedDataPoint[];
		xLabel: string;
		yLabel: string;
		skewLabels?: boolean;
		percentages?: boolean;
		yDomain?: [number, number];
	}

	const { dataPoints, xLabel, yLabel, yDomain, skewLabels = false, percentages = false }: Props = $props();

	const mappedDataPoints = dataPoints.map((point, index) => {
		return {
			index: index,
			label: point.name,
			value: point.value,
			stack: point.layer,
		};
	});
</script>

<Plot
	grid
	color={{ legend: true, scheme: 'tableau10' }}
	x={{ type: 'band', label: `${xLabel} →`, tickRotate: skewLabels ? 45 : 0 }}
	y={{ label: `↑ ${yLabel}`, domain: yDomain, tickFormat: percentages ? d => `${String(d)}%` : d => String(d) }}
	class="no-overflow-clip"
>
	<BarY data={mappedDataPoints} x="label" y="value" fill="stack" sort={{ channel: 'index' }} />
</Plot>
