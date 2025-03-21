<script lang="ts">
	import Chart from 'chart.js/auto';
	import { onDestroy, onMount } from 'svelte';
	import { ThemeObserver } from '../svelteUtils.ts';

	interface Props {
		dataPoints: (number | undefined)[][];
		setLabels: string[];
		labels: string[];
		min?: number;
		max?: number | undefined;
		type?: 'bar' | 'line';
		colors?: any;
	}

	let {
		dataPoints,
		setLabels,
		labels,
		min = 0,
		max = undefined,
		type = 'bar',
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

	let chartEl: HTMLCanvasElement | undefined = $state();

	let themeObserver: ThemeObserver;

	onMount(() => {
		themeObserver = new ThemeObserver();

		// console.log('dataPoints', dataPoints);

		const dataSets = dataPoints.map((points, i) => ({
			label: setLabels[i],
			data: points.map(d => d ?? null),
			backgroundColor: colors[i % colors.length],
			borderColor: colors[i % colors.length],
		}));

		// console.log('dataSets', dataSets);

		themeObserver.addChart(chartStyle => {
			Chart.defaults.color = chartStyle.text;
			Chart.defaults.borderColor = chartStyle.line;

			return new Chart(chartEl!, {
				type: type,
				data: {
					labels: labels,
					datasets: dataSets,
				},
				options: {
					scales: {
						y: {
							min: min,
							max: max,
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
	<canvas bind:this={chartEl} id="percentage-per-month-chart"></canvas>
</div>

<style>
	.chart-wrapper {
		width: 100%;
		aspect-ratio: 1;
		position: relative;
	}
</style>
