<script lang="ts">
	import { BarY, Dot, Line, Plot, Text } from 'svelteplot';
	import type { NamedDataPoint } from '../../../../../data-wasm/pkg/data_wasm';

	interface Props {
		dataPoints: NamedDataPoint[];
		xLabel: string;
		yLabel: string;
		skewLabels?: boolean;
		percentages?: boolean;
		hideBarValues?: boolean;
		yDomain?: [number, number];
	}

	const { dataPoints, xLabel, yLabel, yDomain, skewLabels = false, percentages = false, hideBarValues = false }: Props = $props();

	const mappedDataPoints = dataPoints.map(point => {
		return {
			label: point.name,
			value: point.value,
		};
	});

	function sortData(a: any, b: any) {
		return b.value - a.value; // Sort in descending order
	}
</script>

<Plot
	grid
	x={{ type: 'band', label: `${xLabel} →`, tickRotate: skewLabels ? 45 : 0 }}
	y={{ label: `↑ ${yLabel}`, domain: yDomain, tickFormat: percentages ? d => `${String(d)}%` : d => String(d) }}
	class="no-overflow-clip"
>
	<BarY data={mappedDataPoints} x="label" y="value" fill="var(--sl-color-text-accent)" sort={sortData} />
	{#if !hideBarValues}
		<Text
			data={mappedDataPoints}
			x="label"
			y="value"
			fill="var(--sl-color-text-foreground)"
			text={d => (percentages ? `${d.value.toFixed(1)}%` : d.value.toFixed(1))}
			lineAnchor="bottom"
			dy={-2}
		/>
	{/if}
</Plot>
