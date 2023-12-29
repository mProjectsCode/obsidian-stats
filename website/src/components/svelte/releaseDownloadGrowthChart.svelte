<script lang="ts">
	import Chart from 'chart.js/auto';
	import { onDestroy, onMount } from 'svelte';
	import { ThemeObserver } from './svelteUtils.ts';
	import { ALL_OS, type WeeklyReleaseGrowthEntry } from '../../../../src/release/release.ts';

	export let dataPoints: Record<string, number[]>;
	export let labels: string[];

	let downloadChartEl: HTMLCanvasElement;

	let themeObserver: ThemeObserver;

	const osColors: { [os in (typeof ALL_OS)[number]]: string } = {
		windows: '#0078d7',
		macos: '#f65314',
		linux: '#008272',
	};

	onMount(() => {
		themeObserver = new ThemeObserver();

		themeObserver.addChart(chartStyle => {
			Chart.defaults.color = chartStyle.text;
			Chart.defaults.borderColor = chartStyle.line;

			return new Chart(downloadChartEl!, {
				type: 'bar',
				data: {
					labels,
					datasets: ALL_OS.map(os => ({
						label: os,
						data: dataPoints[os],
						backgroundColor: osColors[os],
						borderColor: osColors[os],
						fill: false,
					})),
				},
				options: {
					scales: {
						x: {
							stacked: true,
						},
						y: {
							stacked: true,
							beginAtZero: true,
						},
					},
					aspectRatio: 1,
				},
			});
		});

		console.log('added charts');

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
