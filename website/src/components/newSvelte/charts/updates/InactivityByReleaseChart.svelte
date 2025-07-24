<script lang="ts">
    import { BarY, Plot } from 'svelteplot';
    import type { PluginInactivityByReleaseDataPoint } from '../../../../../../data-wasm/pkg/data_wasm';

    interface Props {
        dataPoints: PluginInactivityByReleaseDataPoint[];
    }

    const { dataPoints }: Props = $props();

    const mappedDataPoints = dataPoints.map(point => {
        return [{
            date: point.date.substring(0, 7), // Extract YYYY-MM from date string
            type: '1 Year Inactivity',
            count: point.inactive_one_year,
        }, {
            date: point.date.substring(0, 7),
            type: '2 Years Inactivity',
            count: point.inactive_two_years,
        }, {
            date: point.date.substring(0, 7),
            type: '3 Years Inactivity',
            count: point.inactive_three_years,
        }, {
            date: point.date.substring(0, 7),
            type: '4 Years Inactivity',
            count: point.inactive_four_years,
        }, {
            date: point.date.substring(0, 7),
            type: '5 Years Inactivity',
            count: point.inactive_five_years,
        }];
    }).flat();
</script>

<Plot grid x={{type: 'band', label: 'Release Date →', tickRotate: 45}} y={{label: '↑ Plugin Percentage', domain: [0, 100]}} color={{ legend: true, scheme: "tableau10" }} class="no-overflow">
    <BarY data={mappedDataPoints} x="date" y="count" fill="type" />
</Plot>