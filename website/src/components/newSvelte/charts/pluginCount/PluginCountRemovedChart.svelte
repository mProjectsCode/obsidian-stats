<script lang="ts">
    import { Dot, Line, Plot } from "svelteplot";
    import type { PluginCountMonthlyDataPoint } from "../../../../../../data-wasm/pkg/data_wasm";
    import { smooth } from "../chartUtils";

    interface Props {
        dataPoints: PluginCountMonthlyDataPoint[];
    }

    const { dataPoints }: Props = $props();

    const mappedDataPoints = dataPoints.filter(x => x.total_with_removed > 0).map(point => {
        return {
            date: new Date(point.date),
            total: point.total,
            total_with_removed: point.total_with_removed,
            new: point.new,
            new_removed: point.new_removed
        };
    });

    const smoothedData = smooth(mappedDataPoints, 'new_removed', 3);

</script>

<Plot grid y={{label: '↑ Plugins Removed per Month'}}>
    <Line data={smoothedData} x="date" y="new_removed" stroke="var(--sl-color-text-accent)" />
    <Line data={mappedDataPoints} x="date" y="new_removed" strokeDasharray={"5"} opacity={0.3} />
    <Dot data={mappedDataPoints} x="date" y="new_removed" opacity={0.3} />
</Plot>