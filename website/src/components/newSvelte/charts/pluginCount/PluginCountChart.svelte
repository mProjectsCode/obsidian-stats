<script lang="ts">
    import { Dot, Line, Plot } from "svelteplot";
    import type { PluginCountMonthlyDataPoint } from "../../../../../../data-wasm/pkg/data_wasm";

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
</script>

<Plot grid color={{ legend: true, scheme: "tableau10" }}>
    <Line data={mappedDataPoints} x="date" y="total" stroke="Total Plugin Count" marker="dot"  />
    <Line data={mappedDataPoints} x="date" y="total_with_removed" stroke="Plugin Count with Removed Plugins" marker="dot" />
</Plot>