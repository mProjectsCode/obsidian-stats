<script lang="ts">
	import Chart from 'chart.js/auto';
	import { onDestroy, onMount } from 'svelte';
	import { ThemeObserver } from '../svelteUtils.ts';
	import type { ChartDataset } from 'chart.js/auto';

	interface Props {
		dataPoints: number[];
		dataPoints2?: number[] | undefined;
		labels: string[];
		showDatalabels?: boolean;
		showXLabels?: boolean;
		seriesName?: string;
		seriesName2?: string;
		aspectRatio?: number;
		logScale?: boolean;
		calculatePrecentages?: boolean;
		percent100?: number;
		stacked?: boolean;
		colors?: any;
	}

	let {
		dataPoints,
		dataPoints2 = undefined,
		labels,
		showDatalabels = false,
		showXLabels = true,
		seriesName = '',
		seriesName2 = '',
		aspectRatio = 1,
		logScale = false,
		calculatePrecentages = false,
		percent100 = 1,
		stacked = false,
		colors = [
			'rgba(255, 99, 132, 1)', // Red
			'rgba(54, 162, 235, 1)', // Blue
			'rgba(255, 205, 86, 1)', // Yellow
			'rgba(75, 192, 192, 1)', // Teal
			'rgba(255, 159, 64, 1)', // Orange
			'rgba(153, 102, 255, 1)', // Purple
			'rgba(255, 77, 166, 1)', // Pink
			'rgba(102, 204, 255, 1)', // Light Blue
			'rgba(255, 128, 0, 1)', // Orange
			'rgba(70, 191, 189, 1)', // Turquoise
			'rgba(128, 133, 233, 1)', // Lavender
			'rgba(177, 238, 147, 1)', // Lime Green
			'rgba(255, 184, 77, 1)', // Mustard
			'rgba(145, 232, 225, 1)', // Aqua
			'rgba(236, 112, 99, 1)', // Salmon
		],
	}: Props = $props();

	let downloadChartEl: HTMLCanvasElement | undefined = $state();

	let themeObserver: ThemeObserver;

	onMount(() => {
		themeObserver = new ThemeObserver();

		themeObserver.addChart(chartStyle => {
			Chart.defaults.color = chartStyle.text;
			Chart.defaults.borderColor = chartStyle.line;

			let datasets: ChartDataset[] = [
				{
					label: seriesName,
					data: dataPoints,
					backgroundColor: chartStyle.accent,
				},
				dataPoints2 !== undefined
					? {
							label: seriesName2,
							data: dataPoints2,
							backgroundColor: chartStyle.accent,
						}
					: undefined,
			].filter(x => x !== undefined);

			if (datasets.length > 1) {
				for (let i = 0; i < datasets.length; i++) {
					datasets[i].backgroundColor = colors[i % colors.length];
				}
			}

			// console.log(labels, dataPoints);

			return new Chart(downloadChartEl!, {
				type: 'bar',
				data: {
					labels,
					datasets: datasets,
				},
				options: {
					scales: {
						x: {
							grid: {
								display: false,
							},
							ticks: {
								display: showXLabels,
							},
							stacked,
						},
						y: {
							min: logScale ? 1 : 0,
							type: logScale ? 'logarithmic' : 'linear',
							grid: {
								color: chartStyle.line,
							},
							stacked,
						},
					},
					plugins: {
						// @ts-ignore
						datalabels: {
							display: showDatalabels,
							color: chartStyle.text,
							formatter: (value: number, context: unknown) => {
								return value;
							},
						},
						legend: {
							labels: {
								color: chartStyle.text,
							},
						},
						tooltip: {
							callbacks: {
								label: context => {
									const value = context.parsed.y;

									if (calculatePrecentages) {
										const percent = (value * 100) / percent100;
										return `${value} (${percent.toFixed(2)}%)`;
									}
									return `${value}`;
								},
							},
						},
					},
					aspectRatio: aspectRatio,
				},
			});
		});

		themeObserver.initObserver();
	});

	onDestroy(() => {
		themeObserver?.destroy();
	});
</script>

<div class="chart-wrapper" style="--aspect-ratio: {aspectRatio}">
	<canvas bind:this={downloadChartEl} id="release-download-chart"></canvas>
</div>

<style>
	.chart-wrapper {
		width: 100%;
		aspect-ratio: var(--aspect-ratio);
		position: relative;
	}
</style>
