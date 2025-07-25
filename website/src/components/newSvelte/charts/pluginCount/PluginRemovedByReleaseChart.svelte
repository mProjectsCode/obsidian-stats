<script lang="ts">
	import { BarY, Dot, Line, Plot } from 'svelteplot';
	import type { PluginRemovedByReleaseDataPoint } from '../../../../../../data-wasm/pkg/data_wasm';
	import { smooth } from '../chartUtils';

	interface Props {
		dataPoints: PluginRemovedByReleaseDataPoint[];
	}

	const { dataPoints }: Props = $props();

	const mappedDataPoints = dataPoints.map(point => {
		return {
			date: point.date.substring(0, 7), // Extract YYYY-MM from date string
			percentage: point.percentage,
		};
	});
</script>

<Plot grid x={{ type: 'band', label: 'Release Date →', tickRotate: 45 }} y={{ label: '↑ Percentage of Removed Plugins', domain: [0, 100] }} class="no-overflow">
	<BarY data={mappedDataPoints} x="date" y="percentage" fill="var(--sl-color-text-accent)" />
</Plot>
