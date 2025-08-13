<script lang="ts">
	import { Dot, Plot, Pointer, Text, GridY } from 'svelteplot';
	import type { StackedNamedDataPoint } from '../../../../../../data-wasm/pkg/data_wasm';
	import ChartWrapper from '../../ChartWrapper.svelte';

	interface Props {
		dataPoints: StackedNamedDataPoint[];
	}

	const { dataPoints }: Props = $props();

	const mappedDataPoints = dataPoints.map((point, index) => {
		return {
			index: index,
			label: point.name,
			value: point.value,
			stack: point.layer,
		};
	});

	function formatSize(value: any): string {
		return `${(value / 1_000_000).toFixed()} MB`;
	}
</script>

<ChartWrapper>
	<Plot
		color={{ legend: true, scheme: 'tableau10' }}
		x={{ type: 'band', label: `Version Number →`, tickRotate: 45 }}
		y={{ label: `↑ Asset Size`, tickFormat: formatSize }}
		class="no-overflow-clip"
	>
		<GridY />
		<Dot data={mappedDataPoints} x="label" y="value" stroke="stack" sort="index" />
		<Pointer data={mappedDataPoints} x="label" z="stack" maxDistance={2}>
			{#snippet children({ data })}
				<Text {data} fill="stack" x="label" y="value" text={d => formatSize(d.value)} lineAnchor="bottom" dy={-7} />
				<Dot {data} x="label" y="value" fill="stack" />
			{/snippet}
		</Pointer>
	</Plot>
</ChartWrapper>
