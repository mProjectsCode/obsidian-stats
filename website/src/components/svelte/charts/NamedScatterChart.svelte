<script lang="ts">
	import { Dot, GridY, Plot } from 'svelteplot';
	import type { NamedDataPoint } from '../../../../../data-wasm/pkg/data_wasm';
	import ChartWrapper from '../ChartWrapper.svelte';

	interface Props {
		dataPoints: NamedDataPoint[];
		xLabel: string;
		yLabel: string;
		skewLabels?: boolean;
		percentages?: boolean;
		yDomain?: [number, number];
	}

	const { dataPoints, xLabel, yLabel, yDomain, skewLabels = false, percentages = false }: Props = $props();

	const mappedDataPoints = dataPoints as {
		name: string;
		value: number;
	}[];

	function sortData(a: any, b: any) {
		return b.value - a.value; // Sort in descending order
	}
</script>

<ChartWrapper>
	<Plot
		x={{ label: `${xLabel} →`, tickRotate: skewLabels ? 45 : 0 }}
		y={{ label: `↑ ${yLabel}`, domain: yDomain, tickFormat: percentages ? d => `${String(d)}%` : d => String(d), type: 'log' }}
		class="no-overflow-clip"
	>
		<GridY />
		<Dot data={mappedDataPoints} x="name" y="value" stroke="var(--sl-color-text-accent)" />
	</Plot>
</ChartWrapper>
