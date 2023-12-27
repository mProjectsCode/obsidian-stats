<script lang="ts">
    import Chart from 'chart.js/auto';
    import { onDestroy, onMount } from 'svelte';
    import { ThemeObserver } from './svelteUtils.ts';
    import {ALL_OS} from "../../../../src/release/release.ts";

    export let dataPoints: { [os in typeof ALL_OS[number]]: { [release: string]: number } };

    let downloadChartEl: HTMLCanvasElement;

    let themeObserver: ThemeObserver;

    const osColors: { [os in typeof ALL_OS[number]]: string } = {
        windows: '#0078d7',
        macos: '#f65314',
        linux: '#008272',
    };

    onMount(() => {
        const groupedDataPoints: { [release: string]: { [os in typeof ALL_OS[number]]: number } } = {};
        for (const release of Object.keys(dataPoints[ALL_OS[0]])) {
            groupedDataPoints[release] = {};
            for (const os of ALL_OS) {
                groupedDataPoints[release][os] = dataPoints[os][release];
            }
        }

        themeObserver = new ThemeObserver();

        themeObserver.addChart(chartStyle => {
            Chart.defaults.color = chartStyle.text;
            Chart.defaults.borderColor = chartStyle.line;

            return new Chart(downloadChartEl!, {
                type: 'bar',
                data: {
                    labels: Object.keys(dataPoints[ALL_OS[0]]),
                    datasets: ALL_OS.map(os => ({
                        label: os,
                        data: Object.values(dataPoints[os]),
                        backgroundColor: osColors[os],

                    })),
                },
                options: {
                    scales: {
                        x: {
                            stacked: true,
                        },
                        y: {
                            stacked: true,
                            beginAtZero: true,
                            ticks: {
                                format: {
                                    style: 'percent',
                                },
                            },
                        },
                    },
                    aspectRatio: 3 / 2,
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
    <canvas bind:this={downloadChartEl} id="release-download-chart"></canvas>
</div>

<style>
    .chart-wrapper {
        width: 100%;
        aspect-ratio: 3/2;
        position: relative;
    }
</style>
