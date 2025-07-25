<script lang="ts">
	import { Line, Plot } from 'svelteplot';
	import type { DownloadDataPoint, PluginYearlyDataPoint } from '../../../../../data-wasm/pkg/data_wasm';

	interface Props {
		data: PluginYearlyDataPoint[];
		showDots: boolean;
	}

	const { data, showDots }: Props = $props();

	function asAny(data: DownloadDataPoint[]): any[] {
		return data;
	}
</script>

<Plot grid color={{ legend: true, scheme: 'tableau10' }}>
	{#each data as plugin}
		<Line data={asAny(plugin.data)} x={d => new Date(d.date)} y={d => d.downloads} stroke={plugin.id} marker={showDots ? 'dot' : undefined} />
	{/each}
</Plot>
