<script lang="ts">
	import type { DownloadRelationVersionCountDataPoint } from '../../../../utils/dataUrlTypes';
	import ChartWrapper from '../../ChartWrapper.svelte';
	import DownloadRelationVersionCountChartInner from './DownloadRelationVersionCountChartInner.svelte';
</script>

<ChartWrapper dataUrl="/obsidian-stats/data/plugins/stats/downloads-relation-version-count.json">
	{#snippet children(data: DownloadRelationVersionCountDataPoint[] | undefined)}
		<DownloadRelationVersionCountChartInner dataPoints={data ?? []} />
	{/snippet}
</ChartWrapper>
