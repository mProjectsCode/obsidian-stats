<script lang="ts">
	import Chart from 'chart.js/auto';
	import { onDestroy, onMount } from 'svelte';
	import {type DownloadDataPoint} from "../../../../src/types.ts";
	import { ThemeObserver } from './svelteUtils.ts';

	export let dataPoints: DownloadDataPoint[];

	let downloadChartEl: HTMLCanvasElement;
	let downloadGrowthChartEl: HTMLCanvasElement;

	let themeObserver: ThemeObserver;

	onMount(() => {
		themeObserver = new ThemeObserver();

		themeObserver.addChart(chartStyle => {
			Chart.defaults.color = chartStyle.text;
			Chart.defaults.borderColor = chartStyle.line;

			return new Chart(downloadChartEl!, {
				data: {
					labels: dataPoints.map(d => d.date),
					datasets: [
						{
							type: 'line',
							label: 'Downloads',
							data: dataPoints.map(d => d.downloads),
							borderColor: chartStyle.accent,
							backgroundColor: chartStyle.accent,
						},
					],
				},
				options: {
					scales: {
						y: {
							beginAtZero: true,
						},
					},
					aspectRatio: 3 / 2,
				},
			});
		});

		themeObserver.addChart(chartStyle => {
			Chart.defaults.color = chartStyle.text;
			Chart.defaults.borderColor = chartStyle.line;

			return new Chart(downloadGrowthChartEl!, {
				data: {
					labels: dataPoints.map(d => d.date),
					datasets: [
						{
							type: 'bar',
							label: 'Download Growth Week over Week',
							data: dataPoints.map(d => d.growth),
							borderColor: chartStyle.accent,
							backgroundColor: chartStyle.accent,
						},
					],
				},
				options: {
					scales: {
						y: {
							beginAtZero: true,
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
	<canvas bind:this={downloadChartEl} id="plugin-download-chart"></canvas>
</div>

<div class="chart-wrapper">
	<canvas bind:this={downloadGrowthChartEl} id="plugin-download-growth-chart"></canvas>
</div>

<style>
	.chart-wrapper {
		width: 100%;
		aspect-ratio: 3/2;
		position: relative;
	}
</style>
