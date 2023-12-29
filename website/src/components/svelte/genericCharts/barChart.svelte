<script lang="ts">
	import Chart from 'chart.js/auto';
	import { onDestroy, onMount } from 'svelte';
	import { ThemeObserver } from '../svelteUtils.ts';
	import { ALL_OS } from '../../../../../src/release/release.ts';

	export let dataPoints: number[];
	export let labels: string[];
	export let showDatalabels = false;
	export let seriesName = '';

	let downloadChartEl: HTMLCanvasElement;

	let themeObserver: ThemeObserver;

	onMount(() => {
		themeObserver = new ThemeObserver();

		themeObserver.addChart(chartStyle => {
			Chart.defaults.color = chartStyle.text;
			Chart.defaults.borderColor = chartStyle.line;

			console.log(labels, dataPoints);

			return new Chart(downloadChartEl!, {
				type: 'bar',
				data: {
					labels,
					datasets: [
						{
							label: seriesName,
							data: dataPoints,
							backgroundColor: chartStyle.accent,
						},
					],
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
							// ticks: {
							//     callback: (value: number) => {
							//         const weeks = Math.floor(value / 7);
							//         const days = value % 7;
							//
							//         return `${weeks}w ${days}d`;
							//     },
							// },
						},
					},
					plugins: {
						datalabels: {
							display: showDatalabels,
							color: chartStyle.text,
							formatter: (value, context) => {
								return value;
							},
						},
						legend: {
							labels: {
								color: chartStyle.text,
							},
						},
						// tooltip: {
						//     callbacks: {
						//         label: (context: any) => {
						//             const days = Math.ceil(context.raw);
						//             const weeks = Math.floor(days / 7);
						//             const remainingDays = days % 7;
						//
						//             return `${weeks}w ${remainingDays}d`;
						//         },
						//     },
						// },
					},
					aspectRatio: 1,
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
		aspect-ratio: 1;
		position: relative;
	}
</style>
