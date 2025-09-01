<script lang="ts">
	import type { HallOfFameDataPoint } from '../../../../../../data-wasm/pkg/data_wasm';
	import ChartWrapper from '../../ChartWrapper.svelte';
	import HallOfFameChartInner from './HallOfFameChartInner.svelte';

	interface Props {
		showDots: boolean;
		dataUrl: string;
	}

	const { showDots, dataUrl }: Props = $props();
</script>

<ChartWrapper {dataUrl}>
	{#snippet children(data: HallOfFameDataPoint[] | undefined)}
		<HallOfFameChartInner data={data ?? []} {showDots} />
	{/snippet}
</ChartWrapper>
