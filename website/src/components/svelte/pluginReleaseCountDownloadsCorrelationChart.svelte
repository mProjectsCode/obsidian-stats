<script lang="ts">
	import Chart from 'chart.js/auto';
	import { onDestroy, onMount } from 'svelte';
	import { type DownloadReleaseCorrelationDataPoint } from '../../../../src/types.ts';
	import { ThemeObserver } from './svelteUtils.ts';

	interface Props {
		dataPoints: DownloadReleaseCorrelationDataPoint[];
	}

	let { dataPoints }: Props = $props();

	let downloadChartEl: HTMLCanvasElement = $state();

	let themeObserver: ThemeObserver;

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
								label: (item, _) => {
									return `${item.raw.label} (${item.raw.x}, ${item.raw.y})`;
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

<style>
	.chart-wrapper {
		width: 100%;
		aspect-ratio: 1;
		position: relative;
	}
</style>
