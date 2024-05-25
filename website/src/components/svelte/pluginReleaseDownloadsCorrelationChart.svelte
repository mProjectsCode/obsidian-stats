<script lang="ts">
	import Chart from 'chart.js/auto';
	import { onDestroy, onMount } from 'svelte';
	import { type DownloadReleaseCorrelationDataPoint } from '../../../../src/types.ts';
	import { ThemeObserver } from './svelteUtils.ts';
	import { CDate } from '../../../../src/date.ts';
	import { SimpleLinearRegression } from 'ml-regression-simple-linear';

	export let dataPoints: DownloadReleaseCorrelationDataPoint[];

	let downloadChartEl: HTMLCanvasElement;
	let downloadGrowthChartEl: HTMLCanvasElement;
	let downloadNameChartEl: HTMLCanvasElement;

	let themeObserver: ThemeObserver;

	let nameSortedData = dataPoints.toSorted((a, b) => a.name.localeCompare(b.name));

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

		// const now = Date.now();
		// const growthRegression = regression.logarithmic(dataPoints.map((d) => [now - d.initialReleaseDate, d.downloads]));
		//
		// console.log(dataPoints);
		// console.log(growthRegression);

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
						// {
						// 	label: 'Initial Release Time vs. Downloads',
						// 	data: growthRegression.points.map(d => ({ x: d[1], y: -(d[0] - now) })),
						// 	backgroundColor: 'rgba(255, 99, 132, 1)',
						// 	borderColor: 'rgba(255, 99, 132, 1)',
						// },
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
									return CDate.fromDate(new Date(value)).toMonthString();
								},
							},
						},
					},
					aspectRatio: 1,
					plugins: {
						tooltip: {
							callbacks: {
								label: item => {
									return `${item.raw.label} (${item.raw.x}, ${CDate.fromDate(new Date(item.raw.y))})`;
								},
							},
						},
					},
				},
			});
		});

		nameSortedData.forEach((d, i) => d.downloads = d.downloads ?? 0);
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
	<canvas bind:this={downloadChartEl} id="plugin-download-chart"></canvas>
</div>

<div class="chart-wrapper">
	<canvas bind:this={downloadGrowthChartEl} id="plugin-download-growth-chart"></canvas>
</div>

<p>
	The chart below shows the correlation between the name of the plugin and the number of downloads.
	The regression line is calculated using the Simple Linear Regression algorithm.
	It is evident that there is a slight correlation between the placement of the plugin in the alphabet and the number of downloads.
</p>

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
