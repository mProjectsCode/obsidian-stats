<script lang="ts">
	import type { DownloadRelationLocDataPoint } from '../../../../utils/dataUrlTypes';
	import ChartWrapper from '../../ChartWrapper.svelte';
	import DownloadRelationLocChartInner from './DownloadRelationLocChartInner.svelte';
</script>

<ChartWrapper dataUrl="/obsidian-stats/data/plugins/stats/downloads-relation-loc.json">
	{#snippet children(data: DownloadRelationLocDataPoint[] | undefined)}
		<DownloadRelationLocChartInner dataPoints={data ?? []} />
	{/snippet}
</ChartWrapper>
