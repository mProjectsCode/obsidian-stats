<script lang="ts">
    import Chart from 'chart.js/auto';
    import { onDestroy, onMount } from 'svelte';
    import { ThemeObserver } from './svelteUtils.ts';
    import {ALL_OS} from "../../../../src/release/release.ts";

    export let dataPoints: { label: string, data: number[] }[];
    export let labels: string[];
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
                    labels: labels,
                    datasets: dataPoints.map((dataPoint, i) => {
                        return {
                            label: dataPoint.label,
                            data: dataPoint.data,
                            backgroundColor: colors[i],
                        };
                    }),
                },
                options: {
                    plugins: {
                        datalabels: {
                            display: showDatalabels,
                            color: chartStyle.text,
                            formatter: (value, context) => {
                                return value;
                            },
                        },
                    },
                    scales: {
                        x: {
                            stacked: true,
                        },
                        y: {
                            stacked: true,
                            beginAtZero: true,
                            ticks: isPercentual ? {
                                format: {
                                    style: 'percent',
                                },
                            } : undefined,
                        },
                    },
                    aspectRatio: 3 / 2,
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
    <canvas bind:this={downloadChartEl} id="release-download-chart"></canvas>
</div>

<style>
    .chart-wrapper {
        width: 100%;
        aspect-ratio: 3/2;
        position: relative;
    }
</style>
