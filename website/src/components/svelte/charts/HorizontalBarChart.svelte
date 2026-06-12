<script lang="ts">
	import { BarX, GridX, Plot, Text } from 'svelteplot';
	import type { NamedDataPoint } from '../../../../../data-wasm/pkg/data_wasm';
	import ChartWrapper from '../ChartWrapper.svelte';
	import { toCompactString } from './chartUtils';

	interface Props {
		dataPoints: NamedDataPoint[];
		xLabel: string;
		yLabel: string;
		percentages?: boolean;
		logScale?: boolean;
		xDomain?: [number, number];
	}

	const { dataPoints, xLabel, yLabel, xDomain, logScale = false, percentages = false }: Props = $props();

	const mappedDataPoints = $derived.by(() =>
		(dataPoints as { name: string; value: number }[]).toSorted((a, b) => a.value - b.value || b.name.localeCompare(a.name)),
	);
	const names = $derived(mappedDataPoints.map(point => point.name));
	const maxValue = $derived(Math.max(0, ...mappedDataPoints.map(point => point.value)));
	const domain = $derived(xDomain ?? [logScale ? 1 : 0, Math.max(1, maxValue * 1.12)]);
</script>

<ChartWrapper>
	<Plot
		x={{
			label: `${xLabel} →`,
			domain,
			type: logScale ? 'log' : 'linear',
			tickFormat: percentages ? d => `${String(d)}%` : d => toCompactString(d),
		}}
		y={{ type: 'band', label: yLabel, domain: names }}
		class="no-overflow-clip"
	>
		<GridX />
		{#if logScale}
			<BarX data={mappedDataPoints} x1={() => domain[0]} x2="value" y="name" fill="var(--sl-color-text-accent)" />
		{:else}
			<BarX data={mappedDataPoints} x="value" y="name" fill="var(--sl-color-text-accent)" />
		{/if}
		<Text
			data={mappedDataPoints}
			x="value"
			y="name"
			fill="var(--sl-color-text-foreground)"
			text={d => (percentages ? `${d.value.toFixed(1)}%` : toCompactString(d.value))}
			textAnchor="start"
			dx={6}
		/>
	</Plot>
</ChartWrapper>
