<script lang="ts">
	import { BarY, Plot } from 'svelteplot';
	import type { RemovedByReleaseDataPoint } from '../../../../../../data-wasm/pkg/data_wasm';
	import { typeToString, type ItemType } from '../../../../utils/misc';

	interface Props {
		dataPoints: RemovedByReleaseDataPoint[];
		type: ItemType;
	}

	const { dataPoints, type }: Props = $props();

	const mappedDataPoints = dataPoints.map(point => {
		return {
			date: point.date.substring(0, 7), // Extract YYYY-MM from date string
			percentage: point.percentage,
		};
	});
</script>

<Plot
	grid
	x={{ type: 'band', label: 'Release Date →', tickRotate: 45 }}
	y={{ label: `↑ Percentage of Removed ${typeToString(type, true, true)}`, domain: [0, 100], tickFormat: d => `${String(d)}%` }}
	class="no-overflow-clip"
>
	<BarY data={mappedDataPoints} x="date" y="percentage" fill="var(--sl-color-text-accent)" />
</Plot>
