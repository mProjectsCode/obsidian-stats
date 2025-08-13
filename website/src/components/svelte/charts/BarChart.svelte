<script lang="ts">
	import { BarY, GridY, Plot, Text } from 'svelteplot';
	import type { NamedDataPoint } from '../../../../../data-wasm/pkg/data_wasm';
	import ChartWrapper from '../ChartWrapper.svelte';

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
		x={{ type: 'band', label: `${xLabel} →`, tickRotate: skewLabels ? 45 : 0 }}
		y={{ label: `↑ ${yLabel}`, domain: yDomain, tickFormat: percentages ? d => `${String(d)}%` : d => d.toLocaleString(undefined, { notation: 'compact' }) }}
		class="no-overflow-clip"
	>
		<GridY />
		<BarY data={mappedDataPoints} x="name" y="value" fill="var(--sl-color-text-accent)" sort={sortData} />
		{#if !hideBarValues}
			<Text
				data={mappedDataPoints}
				x="name"
				y="value"
				fill="var(--sl-color-text-foreground)"
				text={d =>
					percentages
						? `${d.value.toFixed(1)}%`
						: d.value.toLocaleString(undefined, {
								notation: 'compact',
							})}
				lineAnchor="bottom"
				dy={-2}
			/>
		{/if}
	</Plot>
</ChartWrapper>
