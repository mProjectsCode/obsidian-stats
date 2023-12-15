<script lang="ts">
	import Chart from 'chart.js/auto';
	import {onMount} from 'svelte';
	import {type DownloadDataPoint} from '../../utils/utils';

	export let dataPoints: DownloadDataPoint[];

    let downloadChartEl: HTMLCanvasElement;
    let downloadGrowthChartEl: HTMLCanvasElement;

    onMount(() => {
        const style = getComputedStyle(document.body);
        const accentColor = style.getPropertyValue('--sl-color-accent-high');

        Chart.defaults.borderColor = style.getPropertyValue('--sl-color-hairline-light');
        Chart.defaults.color = style.getPropertyValue('--sl-color-text');

        new Chart(downloadChartEl!, {
            data: {
                labels: dataPoints.map(d => d.date),
                datasets: [
                    {
                        type: 'line',
                        showLine: false,
                        label: 'Downloads',
                        data: dataPoints.map(d => d.downloads),
                        backgroundColor: accentColor,
                        borderColor: accentColor,
                    },
                ]
            },
            options: {
                scales: {
                    y: {
                        beginAtZero: true,
                    }
                },
                aspectRatio: 3/2
            }
        });

        new Chart(downloadGrowthChartEl!, {
            data: {
                labels: dataPoints.map(d => d.date),
                datasets: [
                    {
                        type: 'line',
                        showLine: false,
                        label: 'Download Growth Week over Week',
                        data: dataPoints.map(d => d.growth),
                        backgroundColor: accentColor,
                        borderColor: accentColor,
                    },
                ]
            },
            options: {
                scales: {
                    y: {
                        beginAtZero: true,
                    }
                },
                aspectRatio: 3/2
            }
        });
    });
</script>

<style>
    .chart-wrapper {
        width: 100%;
        aspect-ratio: 3/2;
        position: relative;
    }
</style>

<div class="chart-wrapper">
    <canvas bind:this={downloadChartEl} id="plugin-download-chart"></canvas>
</div>

<div class="chart-wrapper">
    <canvas bind:this={downloadGrowthChartEl} id="plugin-download-growth-chart"></canvas>
</div>