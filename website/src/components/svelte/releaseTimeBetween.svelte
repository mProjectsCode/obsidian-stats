<script lang="ts">
	import Chart from 'chart.js/auto';
	import { onDestroy, onMount } from 'svelte';
	import { ThemeObserver } from './svelteUtils.ts';

	interface Props {
		dataPoints: { [version: string]: number };
	}

	let { dataPoints }: Props = $props();

	let downloadChartEl: HTMLCanvasElement | undefined = $state();

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
					datasets: [
						{
							label: 'Days between releases',
							data: Object.values(dataPoints).map(x => x / 24 / 60 / 60 / 1000),
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
							ticks: {
								callback: (value: string | number) => {
									const v = value as number;
									const weeks = Math.floor(v / 7);
									const days = v % 7;

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
	<canvas bind:this={downloadChartEl} id="release-time-between-chart"></canvas>
</div>

<style>
	.chart-wrapper {
		width: 100%;
		aspect-ratio: 1;
		position: relative;
	}
</style>
