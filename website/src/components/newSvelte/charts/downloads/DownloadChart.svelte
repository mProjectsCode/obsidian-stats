<script lang="ts">
    import { Plot, Line, Frame, RectX, BrushX, GridX, RuleX, Dot, BarY, Pointer, RuleY, AxisX, AxisY } from 'svelteplot';
    import type { VersionDataPoint } from '../../../../../../data-wasm/pkg/data_wasm';
    import { smooth } from '../chartUtils';

    interface Props {
        dataPoints: {
            date: string; // e.g. "2023-10-01"
            downloads: number; // e.g. 100
            delta: number; // e.g. 10 (change from previous data point)
        }[];
        versions?: VersionDataPoint[]; // Optional, for version markers
    }

    const {
        dataPoints,
        versions,
    }: Props = $props();

    const mappedData = dataPoints.map(point => {
        return {
            date: new Date(point.date),
            downloads: point.downloads ?? null,
            delta: point.delta ?? null
        };
    });

    const smoothedDelta = smooth(mappedData, 'delta', 2);

    let brush = $state({
        enabled: false,
        x1: new Date(2024, 1, 1),
        x2: new Date(2025, 1, 1)
    });

    let zoomedToYear = $derived(
        brush.enabled && brush.x1 && brush.x2
            ? Math.abs(brush.x2.getTime() - brush.x1.getTime()) < 1000 * 60 * 60 * 24 * 365
            : false
    );

    const filteredData = $derived(
        brush.enabled
            ? mappedData.filter(
                  (d) =>
                      d.date >= brush.x1 &&
                      d.date <= brush.x2
              )
            : mappedData
    );

    const filteredSmoothedDelta = $derived(
        brush.enabled
            ? smoothedDelta.filter(
                  (d) =>
                      d.date >= brush.x1 &&
                      d.date <= brush.x2
              )
            : smoothedDelta
    );

    const mappedVersions = versions?.map(version => {
        return {
            date: new Date(version.date),
            version: version.version
        };
    });

    const filteredVersions = $derived(
        brush.enabled
            ? mappedVersions?.filter(
                  (v) =>
                      v.date >= brush.x1 &&
                      v.date <= brush.x2
              )
            : mappedVersions
    );

    const undefinedData = undefined as unknown as [];

</script>

<div style="touch-action: none">
    <Plot
        height={90}
        x={{ label: '', grid: true }}
        y={{ axis: false, label: '' }}>
        <Frame opacity={0.4} />
        <Line
            data={mappedData}
            x="date"
            y="downloads"
            opacity={0.3} />
        {#if brush.enabled}
            <RectX
                data={undefinedData}
                {...brush}
                fill="#33aaee"
                opacity={0.2} />
            <Line data={filteredData} x="date" y="downloads" />
        {/if}
        <BrushX
            bind:brush
            stroke={false}
            constrainToDomain />
    </Plot>
</div>

<Plot grid x={{label: 'Date →'}} y={{label: '↑ Downloads' }}>
    <AxisX />
    <AxisY />
    <Line data={filteredData} x={"date"} y={"downloads"} stroke={"var(--sl-color-text-accent)"}></Line>
    {#if filteredVersions}
        <RuleX
            data={filteredVersions}
            x={"date"}
            strokeOpacity={0.3}
            strokeDasharray={"5"}
        />
    {/if}
    <!-- <Pointer
        data={filteredData}
        x="date"
        y="downloads">
        {#snippet children({ data })}
            <RuleX {data} x="date" opacity="0.3" />
            <RuleY {data} y="downloads" opacity="0.3" />
            <AxisX
                data={data.map((d) => d.date)} />
            <AxisY
                data={data.map((d) => d.downloads)} />
        {/snippet}
    </Pointer> -->
</Plot>

<Plot grid x={{label: 'Date →'}} y={{label: '↑ Weekly Delta'}}>
    <AxisX />
    <AxisY />
    {#if zoomedToYear} 
        <Dot data={filteredData} x={"date"} y={"delta"} opacity={0.5} stroke={"var(--sl-color-text-accent)"} />
    {/if}
    <Line data={filteredData} x={"date"} y={"delta"} strokeDasharray={"5"} opacity={0.5} stroke={"var(--sl-color-text-accent)"} />
    <Line data={filteredSmoothedDelta} x={"date"} y={"delta"} stroke={"var(--sl-color-text-accent)"} />
    {#if filteredVersions}
        <RuleX
            data={filteredVersions}
            x={"date"}
            strokeOpacity={0.3}
            strokeDasharray={"5"}
        />
    {/if}
</Plot>
