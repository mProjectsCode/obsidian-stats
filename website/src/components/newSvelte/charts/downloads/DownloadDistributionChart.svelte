<script lang="ts">
    import { Dot, Line, Plot, RegressionY } from 'svelteplot';
    import type { IndividualDownloadDataPoint } from '../../../../../../data-wasm/pkg/data_wasm';

    interface Props {
        dataPoints: IndividualDownloadDataPoint[];
    }

    const { dataPoints }: Props = $props();

    const mappedData = dataPoints.filter(x => x.downloads > 0 && x.version_count > 0).sort((a, b) => b.downloads - a.downloads).map((point, i) => {
        return {
            rank: i,
            id: point.id,
            downloads: point.downloads,
        };
    });


</script>

<Plot grid x={{label: 'Rank →', type: 'linear'}} y={{label: '↑ Downloads', type: 'log', domain: [1, 10_000_000]}}>
    <Line data={mappedData} x="rank" y="downloads" stroke="var(--sl-color-text-accent)" />
</Plot>