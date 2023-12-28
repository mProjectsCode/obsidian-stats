<script lang="ts">
    import Chart from 'chart.js/auto';
    import ChartDataLabels from 'chartjs-plugin-datalabels';

    Chart.register(ChartDataLabels);

    import { onDestroy, onMount } from 'svelte';
    import { ThemeObserver } from './svelteUtils.ts';

    let downloadChartEl: HTMLCanvasElement;

    let themeObserver: ThemeObserver;

    export let dataPoints: { label: string, value: number }[];
    export let colors = [
        'rgba(255, 99, 132, 1)',   // Red
        'rgba(54, 162, 235, 1)',   // Blue
        'rgba(255, 205, 86, 1)',   // Yellow
        'rgba(75, 192, 192, 1)',   // Teal
        'rgba(255, 159, 64, 1)',   // Orange
        'rgba(153, 102, 255, 1)',  // Purple
        'rgba(255, 77, 166, 1)',   // Pink
        'rgba(102, 204, 255, 1)',  // Light Blue
        'rgba(255, 128, 0, 1)',    // Orange
        'rgba(70, 191, 189, 1)',   // Turquoise
        'rgba(128, 133, 233, 1)',  // Lavender
        'rgba(177, 238, 147, 1)',  // Lime Green
        'rgba(255, 184, 77, 1)',   // Mustard
        'rgba(145, 232, 225, 1)',  // Aqua
        'rgba(236, 112, 99, 1)',   // Salmon
    ];
    export let isPercentual = false;
    export let showDatalabels = false;


    onMount(() => {
        themeObserver = new ThemeObserver();

        themeObserver.addChart(chartStyle => {
            Chart.defaults.color = chartStyle.text;
            Chart.defaults.borderColor = chartStyle.line;

            // Prevent reuse of colors


            return new Chart(downloadChartEl, {
                type: 'pie',
                data: {
                    labels: dataPoints.map(({ label }) => label),
                    datasets: [
                        {
                            label: 'Downloads',
                            data: dataPoints.map(({ value }) => value),
                            backgroundColor: colors,
                        },
                    ],
                },
                options: {
                    tooltips: {
                        enabled: true,
                    },
                    plugins: {
                        legend: {
                            labels: {
                                color: chartStyle.text,
                            },
                        },
                        datalabels: {
                            display: showDatalabels,
                            color: chartStyle.text,
                            formatter: (value, context) => {
                                if (isPercentual) {
                                    return `${(100 * value as number).toFixed(2)}%`;
                                }

                                return value;
                            },
                        },
                        tooltip: {
                            callbacks: {
                                label: (context) => {
                                    if (isPercentual) {
                                        return `${context.label}: ${(100 * context.parsed as number).toFixed(2)}%`;
                                    } else {
                                        return `${context.label}: ${context.parsed}`;
                                    }
                                },
                            },
                        },
                    },
                },
            });
        });

        themeObserver.initObserver();
    });

    onDestroy(() => {
        themeObserver?.destroy();
    });
</script>

<div class="chart-wrapper">
    <canvas bind:this={downloadChartEl} id="release-time-between-chart"></canvas>
</div>

<style>
    .chart-wrapper {
        width: 100%;
        aspect-ratio: 3/2;
        position: relative;
    }
</style>
