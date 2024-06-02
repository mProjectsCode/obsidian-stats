<script lang="ts">
	import Chart from 'chart.js/auto';
	import { onDestroy, onMount } from 'svelte';
	import { type DownloadReleaseCorrelationDataPoint } from '../../../../src/types.ts';
	import { ThemeObserver } from './svelteUtils.ts';
	import { CDate } from '../../../../src/date.ts';
	import { SimpleLinearRegression } from 'ml-regression-simple-linear';

	export let dataPoints: DownloadReleaseCorrelationDataPoint[];

	let downloadNameChartEl: HTMLCanvasElement;

	let themeObserver: ThemeObserver;

	let nameSortedData = dataPoints.slice().sort((a, b) => a.name.localeCompare(b.name));

	onMount(() => {
		themeObserver = new ThemeObserver();

		nameSortedData.forEach((d, i) => (d.downloads = d.downloads ?? 0));
		const xData = nameSortedData.map((_, i) => i);
		const yData = nameSortedData.map(d => d.downloads);
		const nameRegression = new SimpleLinearRegression(xData, yData);
		const nameRegressionScore = nameRegression.score(xData, yData);

		themeObserver.addChart(chartStyle => {
			Chart.defaults.color = chartStyle.text;
			Chart.defaults.borderColor = chartStyle.line;

			return new Chart(downloadNameChartEl!, {
				type: 'scatter',
				data: {
					labels: nameSortedData.map(d => d.name),
					datasets: [
						{
							label: 'Plugins sorted by name vs. Downloads',
							data: nameSortedData.map((d, i) => ({ x: i, y: d.downloads, label: d.name })),
							backgroundColor: chartStyle.accent,
							borderColor: chartStyle.accent,
						},
						{
							label: `Regression Line (${nameRegressionScore.r2})`,
							data: nameSortedData.map((_, i) => ({ x: i, y: nameRegression.predict(i), label: 'Regression' })),
							backgroundColor: 'rgba(255, 99, 132, 1)',
							borderColor: 'rgba(255, 99, 132, 1)',
							type: 'line',
						},
					],
				},
				options: {
					scales: {
						x: {
							type: 'linear',
							position: 'bottom',
							max: nameSortedData.length,
						},
						y: {
							type: 'logarithmic',
							position: 'left',
						},
					},
					aspectRatio: 1,
					plugins: {
						tooltip: {
							callbacks: {
								label: item => {
									return `${item.raw.label} (${item.raw.y})`;
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
	<canvas bind:this={downloadNameChartEl} id="plugin-download-name-chart"></canvas>
</div>

<style>
	.chart-wrapper {
		width: 100%;
		aspect-ratio: 1;
		position: relative;
	}
</style>
