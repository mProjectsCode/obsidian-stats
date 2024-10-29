<script lang="ts">
	import Chart from 'chart.js/auto';
	import { onDestroy, onMount } from 'svelte';
	import { type PerMonthDataPoint } from '../../../../../src/types.ts';
	import { ThemeObserver } from '../svelteUtils.ts';

	interface Props {
		dataPoints: PerMonthDataPoint[];
		title: string;
		min?: number;
		max?: number | undefined;
		type?: 'bar' | 'line';
	}

	let {
		dataPoints,
		title,
		min = 0,
		max = undefined,
		type = 'bar'
	}: Props = $props();

	let chartEl: HTMLCanvasElement = $state();

	let themeObserver: ThemeObserver;

	onMount(() => {
		themeObserver = new ThemeObserver();

		themeObserver.addChart(chartStyle => {
			Chart.defaults.color = chartStyle.text;
			Chart.defaults.borderColor = chartStyle.line;

			return new Chart(chartEl!, {
				type: type,
				data: {
					labels: dataPoints.map(d => `${d.year}-${d.month}`),
					datasets: [
						{
							label: title,
							data: dataPoints.map(d => d.value),
							backgroundColor: chartStyle.accent,
							borderColor: chartStyle.accent,
						},
					],
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
