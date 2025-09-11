<script lang="ts">
	import { BarY, GridX, GridY, Plot } from 'svelteplot';
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
</script>

<ChartWrapper>
	<Plot
		color={{ legend: true, scheme: 'tableau10' }}
		x={{ type: 'band', label: `Version Number →`, tickRotate: 45, padding: 0 }}
		y={{ label: `↑ Changes` }}
		class="no-overflow-clip"
	>
		<GridY></GridY>
		<BarY data={mappedDataPoints} x="label" y="value" fill="stack" sort="index" />
	</Plot>
</ChartWrapper>
