<script lang="ts">
	import type { DownloadDataPoint, VersionDataPoint } from '../../../../../../data-wasm/pkg/data_wasm';
	import ChartWrapper from '../../ChartWrapper.svelte';
	import DownloadChartInner from './DownloadChartInner.svelte';

	interface Props {
		dataUrl: string;
		versions?: VersionDataPoint[]; // Optional, for version markers
	}

	const { dataUrl, versions }: Props = $props();
</script>

<ChartWrapper {dataUrl}>
	{#snippet children(data: DownloadDataPoint[] | undefined)}
		<DownloadChartInner dataPoints={data ?? []} {versions} />
	{/snippet}
</ChartWrapper>
