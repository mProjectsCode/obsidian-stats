<script lang="ts">
    import { Dot, GridY, Plot, RegressionY } from 'svelteplot';
    import type { IndividualDownloadDataPoint } from '../../../../../data-wasm/pkg/data_wasm';

    interface Props {
        dataPoints: IndividualDownloadDataPoint[];
    }

    const { dataPoints }: Props = $props();

    const mappedData = dataPoints.filter(x => x.downloads > 0 && x.version_count > 0).sort((a, b) => a.name.localeCompare(b.name)).map((point, i) => {
        return {
            index: i,
            id: point.id,
            name: point.name,
            downloads: point.downloads ?? null,
            version_count: point.version_count ?? null
        };
    });
</script>

<Plot x={{label: 'Alphabetical Order →', ticks: [] }} y={{label: '↑ Downloads', type: 'log', domain: [10, 10_000_000]}}>
    <GridY />
    <Dot data={mappedData} x="index" y="downloads" opacity={0.3} />
    <RegressionY
        data={mappedData}
        x="index"
        y="downloads"
        stroke="var(--sl-color-text-accent)" />
</Plot>