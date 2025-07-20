<script lang="ts">
    import { Line, Plot } from "svelteplot";
    import type { PluginYearlyDataPoint } from "../../../../../data-wasm/pkg/data_wasm";

    interface Props {
        data: PluginYearlyDataPoint[];
        showDots: boolean;
    }

    const { data, showDots }: Props = $props();

</script>

<!-- <table>
    <thead>
        <tr>
            <th>Plugin Id</th>
            <th>Plugin</th>
            <th>Total Downloads</th>
        </tr>
    </thead>
    <tbody>
        {#each data as plugin}
            <tr>
                <td><a href="/obsidian-stats/plugins/{plugin.id}">{plugin.id}</a></td>
                <td>{plugin.name}</td>
                <td>{plugin.downloads_new}</td>
            </tr>
        {/each}
    </tbody>
</table> -->

<Plot grid color={{ legend: true, scheme: "tableau10" }}>
    {#each data as plugin }
        <Line 
            data={plugin.data as any[]}
            x={d => new Date(d.date)}
            y={d => d.downloads}
            stroke={plugin.id}
            marker={showDots ? "dot" : undefined} 
        />
    {/each}
</Plot>