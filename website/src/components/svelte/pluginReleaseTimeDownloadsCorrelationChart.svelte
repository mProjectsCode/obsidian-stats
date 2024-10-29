<script lang="ts">
	import Chart from 'chart.js/auto';
	import { onDestroy, onMount } from 'svelte';
	import { type DownloadReleaseCorrelationDataPoint } from '../../../../src/types.ts';
	import { ThemeObserver } from './svelteUtils.ts';
	import { CDate } from '../../../../src/date.ts';

	interface Props {
		dataPoints: DownloadReleaseCorrelationDataPoint[];
	}

	let { dataPoints }: Props = $props();

	let downloadGrowthChartEl: HTMLCanvasElement | undefined = $state();

	let themeObserver: ThemeObserver;

	onMount(() => {
		themeObserver = new ThemeObserver();

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
									const i = item as { raw: { x: number; y: number; label: string } };
									return `${i.raw.label} (${i.raw.x}, ${CDate.fromDate(new Date(i.raw.y))})`;
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
	<canvas bind:this={downloadGrowthChartEl} id="plugin-download-growth-chart"></canvas>
</div>

<style>
	.chart-wrapper {
		width: 100%;
		aspect-ratio: 1;
		position: relative;
	}
</style>
