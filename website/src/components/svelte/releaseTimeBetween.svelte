<script lang="ts">
    import Chart from 'chart.js/auto';
    import { onDestroy, onMount } from 'svelte';
    import { ThemeObserver } from './svelteUtils.ts';

    export let dataPoints: { [version: string]: number };

    let downloadChartEl: HTMLCanvasElement;

    let themeObserver: ThemeObserver;

    onMount(() => {
        themeObserver = new ThemeObserver();

        themeObserver.addChart(chartStyle => {
            Chart.defaults.color = chartStyle.text;
            Chart.defaults.borderColor = chartStyle.line;

            return new Chart(downloadChartEl!, {
                type: 'bar',
                data: {
                    labels: Object.keys(dataPoints),
                    datasets: [{
                        label: 'Days between releases',
                        data: Object.values(dataPoints).map(x => x / 24 / 60 / 60 / 1000),
                        backgroundColor: chartStyle.accent,
                    }],
                },
                options: {
                    scales: {
                        x: {
                            grid: {
                                display: false,
                            },
                        },
                        y: {
                            grid: {
                                color: chartStyle.line,
                            },
                            // Set ticks to weeks + days
                            ticks: {
                                callback: (value: number) => {
                                    const weeks = Math.floor(value / 7);
                                    const days = value % 7;

                                    return `${weeks}w ${days}d`;
                                },
                            },
                        },
                    },
                    plugins: {
                        legend: {
                            labels: {
                                color: chartStyle.text,
                            },
                        },
                        tooltip: {
                            callbacks: {
                                label: (context: any) => {
                                    const days = Math.ceil(context.raw);
                                    const weeks = Math.floor(days / 7);
                                    const remainingDays = days % 7;

                                    return `${weeks}w ${remainingDays}d`;
                                },
                            },
                        },
                    },
                },
            });
        });

        console.log('added charts');

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
