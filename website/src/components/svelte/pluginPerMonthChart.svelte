<script lang="ts">
	import Chart from 'chart.js/auto';
	import {onMount} from 'svelte';
	import {
		type PerMonthDataPoint,
	} from '../../utils/utils';

	export let dataPoints: PerMonthDataPoint[];
	export let title: string;
	export let min: number = 0;
	export let max: number | undefined = undefined;
	export let type: 'bar' | 'line' = 'bar';

	let chartEl: HTMLCanvasElement;

	onMount(() => {
		const style = getComputedStyle(document.body);
		const accentColor = style.getPropertyValue('--sl-color-accent-high');

		Chart.defaults.borderColor = style.getPropertyValue('--sl-color-hairline-light');
		Chart.defaults.color = style.getPropertyValue('--sl-color-text');

		new Chart(chartEl!, {
			type: type,
			data: {
				labels: dataPoints.map(d => `${d.year}-${d.month}`),
				datasets: [
					{
						label: title,
						data: dataPoints.map(d => d.value),
						backgroundColor: accentColor,
						borderColor: accentColor,
					},
				]
			},
			options: {
				scales: {
					y: {
						min: min,
                        max: max,
					}
				},
				aspectRatio: 3/2,
			}
		});


	});
</script>

<style>
    .chart-wrapper {
        width: 100%;
        aspect-ratio: 3/2;
        position: relative;
    }
</style>

<div class="chart-wrapper">
    <canvas bind:this={chartEl} id="percentage-per-month-chart"></canvas>
</div>