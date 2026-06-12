<script lang="ts">
	import { BarY, GridY, Plot, Text } from 'svelteplot';
	import type { NamedDataPoint } from '../../../../../data-wasm/pkg/data_wasm';
	import ChartWrapper from '../ChartWrapper.svelte';
	import { toCompactString } from './chartUtils';

	interface Props {
		dataPoints: NamedDataPoint[];
		xLabel: string;
		yLabel: string;
		skewLabels?: boolean;
		percentages?: boolean;
		hideBarValues?: boolean;
		logScale?: boolean;
		yDomain?: [number, number];
	}

	const { dataPoints, xLabel, yLabel, yDomain, skewLabels = false, percentages = false, hideBarValues = false, logScale = false }: Props = $props();

	const mappedDataPoints = $derived(
		dataPoints as {
			name: string;
			value: number;
		}[],
	);

	function sortData(a: any, b: any) {
		return b.value - a.value; // Sort in descending order
	}

	const maxValue = $derived(Math.max(0, ...mappedDataPoints.map(point => point.value)));
	const domain = $derived(yDomain ?? [logScale ? 1 : 0, Math.max(1, maxValue * 1.12)]);
</script>

<ChartWrapper>
	<Plot
		x={{ type: 'band', label: `${xLabel} →`, tickRotate: skewLabels ? 45 : 0 }}
		y={{
			label: `↑ ${yLabel}`,
			domain,
			type: logScale ? 'log' : 'linear',
			tickFormat: percentages ? d => `${String(d)}%` : d => toCompactString(d),
		}}
		class="no-overflow-clip"
	>
		<GridY />
		{#if logScale}
			<BarY data={mappedDataPoints} x="name" y1={() => domain[0]} y2="value" fill="var(--sl-color-text-accent)" sort={sortData} />
		{:else}
			<BarY data={mappedDataPoints} x="name" y="value" fill="var(--sl-color-text-accent)" sort={sortData} />
		{/if}
		{#if !hideBarValues}
			<Text
				data={mappedDataPoints}
				x="name"
				y="value"
				fill="var(--sl-color-text-foreground)"
				text={d => (percentages ? `${d.value.toFixed(1)}%` : toCompactString(d.value))}
				lineAnchor="bottom"
				dy={-2}
			/>
		{/if}
	</Plot>
</ChartWrapper>
