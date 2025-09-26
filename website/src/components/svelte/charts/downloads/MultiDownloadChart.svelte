<script lang="ts">
	import type { MultiDownloadDataPoint, VersionDataPoint } from '../../../../../../data-wasm/pkg/data_wasm';
	import ChartWrapper from '../../ChartWrapper.svelte';
	import MultiDownloadChartInner from './MultiDownloadChartInner.svelte';

	interface Props {
		dataUrl: string;
	}

	const { dataUrl }: Props = $props();
</script>

<ChartWrapper {dataUrl}>
	{#snippet children(data: MultiDownloadDataPoint[] | undefined)}
		<MultiDownloadChartInner dataPoints={data ?? []} />
	{/snippet}
</ChartWrapper>
