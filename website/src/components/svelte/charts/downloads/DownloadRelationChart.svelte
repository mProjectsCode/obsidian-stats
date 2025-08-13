<script lang="ts">
	import { Dot, Plot } from 'svelteplot';
	import ChartWrapper from '../../ChartWrapper.svelte';

	interface Props {
		dataPoints: {
			date: Date;
			downloads: number;
			version_count: number;
		}[];
	}

	const { dataPoints }: Props = $props();
</script>

<ChartWrapper>
	<Plot grid x={{ label: 'Releases →', type: 'log' }} y={{ label: '↑ Downloads', type: 'log', domain: [1, 10_000_000] }}>
		<Dot data={dataPoints} x="version_count" y="downloads" opacity={0.3} stroke="var(--sl-color-text-accent)" />
	</Plot>
</ChartWrapper>

<ChartWrapper>
	<Plot grid x={{ label: 'Release Date →' }} y={{ label: '↑ Downloads', type: 'log', domain: [1, 10_000_000] }}>
		<Dot data={dataPoints} x="date" y="downloads" opacity={0.3} stroke="var(--sl-color-text-accent)" />
	</Plot>
</ChartWrapper>
