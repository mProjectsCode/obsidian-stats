<script lang="ts">
	import { Dot, Plot, RegressionY } from 'svelteplot';
	import type { IndividualDownloadDataPoint } from '../../../../../../data-wasm/pkg/data_wasm';

	interface Props {
		dataPoints: IndividualDownloadDataPoint[];
	}

	const { dataPoints }: Props = $props();

	const mappedData = dataPoints
		.filter(x => x.downloads > 0 && x.version_count > 0)
		.map(point => {
			return {
				id: point.id,
				date: new Date(point.date),
				downloads: point.downloads ?? null,
				version_count: point.version_count ?? null,
			};
		});
</script>

<Plot grid x={{ label: 'Releases →', type: 'log' }} y={{ label: '↑ Downloads', type: 'log', domain: [1, 10_000_000] }}>
	<Dot data={mappedData} x="version_count" y="downloads" opacity={0.3} stroke="var(--sl-color-text-accent)" />
</Plot>

<Plot grid x={{ label: 'Release Date →' }} y={{ label: '↑ Downloads', type: 'log', domain: [1, 10_000_000] }}>
	<Dot data={mappedData} x="date" y="downloads" opacity={0.3} stroke="var(--sl-color-text-accent)" />
</Plot>
