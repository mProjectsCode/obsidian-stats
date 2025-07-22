<script lang="ts">
    import { BarY, Dot, Line, Plot } from "svelteplot";
    import type { PluginRemovedByReleaseDataPoint } from "../../../../../../data-wasm/pkg/data_wasm";
    import { smooth } from "../chartUtils";

    interface Props {
        dataPoints: PluginRemovedByReleaseDataPoint[];
    }

    const { dataPoints }: Props = $props();

    const mappedDataPoints = dataPoints.map(point => {
        return {
            date: new Date(point.date),
            date_str: point.date.substring(0, 7), // YYYY-MM format
            percentage: point.percentage,
        };
    });

    const smoothedData = smooth(mappedDataPoints, 'percentage', 3);

    function formatDate(date: unknown): string {
        if (!(date instanceof Date)) {
            return '';
        }
        return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}`; // Format as YYYY-MM
    }

</script>

<Plot grid x={{type: 'band', label: 'Release Date →', tickRotate: 45, tickFormat: d => formatDate(d)}} y={{label: '↑ Percentage of Removed Plugins', domain: [0, 100]}} class="no-overflow">
    <BarY data={mappedDataPoints} x="date" y="percentage" fill="var(--sl-color-text-accent)" />
</Plot>