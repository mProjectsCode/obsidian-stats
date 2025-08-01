<script lang="ts">
	import { BarX, BarY, Plot, HTMLTooltip } from 'svelteplot';
	import type { StackedNamedDataPoint } from '../../../../../data-wasm/pkg/data_wasm';

	interface Props {
		dataPoints: StackedNamedDataPoint[];
		skewLabels?: boolean;
		percentages?: boolean;
		yDomain?: [number, number];
	}

	const { dataPoints, yDomain, skewLabels = false, percentages = false }: Props = $props();

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
	x={{ label: '', domain: yDomain, tickFormat: percentages ? d => `${String(d)}%` : d => String(d) }}
	y={{ type: 'band', label: '', tickRotate: skewLabels ? 45 : 0 }}
	class="no-overflow-clip"
>
	<BarX data={mappedDataPoints} x="value" y="label" fill="stack" sort={{ channel: 'index' }} />
</Plot>
