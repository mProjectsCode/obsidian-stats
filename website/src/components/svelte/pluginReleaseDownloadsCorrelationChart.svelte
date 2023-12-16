<script lang="ts">
	import Chart from 'chart.js/auto';
	import { onDestroy, onMount } from 'svelte';
	import { dateToString, type DownloadReleaseCorrelationDataPoint } from '../../utils/utils';
	import { ThemeObserver } from './svelteUtils.ts';

	export let dataPoints: DownloadReleaseCorrelationDataPoint[];

	let downloadChartEl: HTMLCanvasElement;
	let downloadGrowthChartEl: HTMLCanvasElement;

	let themeObserver: ThemeObserver;

	onMount(() => {
		themeObserver = new ThemeObserver();

		themeObserver.addChart(chartStyle => {
			Chart.defaults.color = chartStyle.text;
			Chart.defaults.borderColor = chartStyle.line;

			return new Chart(downloadChartEl!, {
				type: 'scatter',
				data: {
					labels: dataPoints.map(d => d.id),
					datasets: [
						{
							label: 'Releases vs. Downloads',
							data: dataPoints.map(d => ({ x: d.downloads, y: d.releases, label: d.id })),
							backgroundColor: chartStyle.accent,
							borderColor: chartStyle.accent,
						},
					],
				},
				options: {
					scales: {
						x: {
							type: 'logarithmic',
							position: 'bottom',
						},
						y: {
							type: 'linear',
							position: 'left',
							min: 1,
						},
					},
					aspectRatio: 1,
					plugins: {
						tooltip: {
							callbacks: {
								label: (item, data) => {
									return `${item.raw.label} (${item.raw.x}, ${item.raw.y})`;
								},
							},
						},
					},
				},
			});
		});

		themeObserver.addChart(chartStyle => {
			Chart.defaults.color = chartStyle.text;
			Chart.defaults.borderColor = chartStyle.line;

			return new Chart(downloadGrowthChartEl!, {
				type: 'scatter',
				data: {
					labels: dataPoints.map(d => d.id),
					datasets: [
						{
							label: 'Initial Release Time vs. Downloads',
							data: dataPoints.map(d => ({ x: d.downloads, y: d.initialReleaseDate, label: d.id })),
							backgroundColor: chartStyle.accent,
							borderColor: chartStyle.accent,
						},
					],
				},
				options: {
					scales: {
						x: {
							type: 'logarithmic',
							position: 'bottom',
						},
						y: {
							type: 'linear',
							position: 'left',
							max: Date.now(),
							ticks: {
								stepSize: 1000 * 60 * 60 * 24 * 90,
								autoSkip: false,
								includeBounds: false,
								callback: value => {
									const date = new Date(value);
									const dateString = dateToString(date);

									return dateString.substring(0, dateString.length - 3);
								},
							},
						},
					},
					aspectRatio: 1,
					plugins: {
						tooltip: {
							callbacks: {
								label: item => {
									return `${item.raw.label} (${item.raw.x}, ${dateToString(new Date(item.raw.y))})`;
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
	<canvas bind:this={downloadChartEl} id="plugin-download-chart"></canvas>
</div>

<div class="chart-wrapper">
	<canvas bind:this={downloadGrowthChartEl} id="plugin-download-growth-chart"></canvas>
</div>

<style>
	.chart-wrapper {
		width: 100%;
		aspect-ratio: 1;
		position: relative;
	}
</style>
