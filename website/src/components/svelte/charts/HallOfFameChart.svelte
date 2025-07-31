<script lang="ts">
	import { Line, Plot, Pointer, Text, Dot } from 'svelteplot';
	import type { DownloadDataPoint, HallOfFameDataPoint } from '../../../../../data-wasm/pkg/data_wasm';

	interface Props {
		data: HallOfFameDataPoint[];
		showDots: boolean;
	}

	const { data, showDots }: Props = $props();

	const mappedData = data
		.map(plugin => {
			return plugin.data.map(point => {
				return {
					date: new Date(point.date),
					downloads: point.downloads as number,
					id: plugin.id,
				};
			});
		})
		.flat();

	function asAny(data: DownloadDataPoint[]): any[] {
		return data;
	}
</script>

<Plot grid color={{ legend: true, scheme: 'tableau10' }} class="no-overflow-clip">
	<Line data={mappedData} x="date" y="downloads" stroke="id" marker={showDots ? 'dot' : undefined} />
	<Pointer data={mappedData} x="date" z="id" maxDistance={30}>
		{#snippet children({ data })}
			<Text {data} fill="id" x="date" y="downloads" text={d => d.downloads.toFixed()} lineAnchor="bottom" dy={-7} />
			<Dot {data} x="date" y="downloads" fill="id" />
		{/snippet}
	</Pointer>
</Plot>
