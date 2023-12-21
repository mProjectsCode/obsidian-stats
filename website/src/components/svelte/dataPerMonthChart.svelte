<script lang="ts">
	import Chart from 'chart.js/auto';
	import { onDestroy, onMount } from 'svelte';
	import { type PerMonthDataPoint } from '../../utils/utils';
	import { ThemeObserver } from './svelteUtils.ts';

	export let dataPoints: PerMonthDataPoint[];
	export let title: string;
	export let min: number = 0;
	export let max: number | undefined = undefined;
	export let type: 'bar' | 'line' = 'bar';

	let chartEl: HTMLCanvasElement;

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
					aspectRatio: 3 / 2,
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
		aspect-ratio: 3/2;
		position: relative;
	}
</style>
