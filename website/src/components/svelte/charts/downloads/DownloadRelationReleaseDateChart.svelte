<script lang="ts">
	import type { DownloadRelationReleaseDateDataPoint } from '../../../../utils/dataUrlTypes';
	import ChartWrapper from '../../ChartWrapper.svelte';
	import DownloadRelationReleaseDateChartInner from './DownloadRelationReleaseDateChartInner.svelte';
</script>

<ChartWrapper dataUrl="/obsidian-stats/data/plugins/stats/downloads-relation-release-date.json">
	{#snippet children(data: DownloadRelationReleaseDateDataPoint[] | undefined)}
		<DownloadRelationReleaseDateChartInner dataPoints={data ?? []} />
	{/snippet}
</ChartWrapper>
