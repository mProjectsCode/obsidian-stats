<script lang="ts">
	import Chart from 'chart.js/auto';
	import {onMount} from 'svelte';
	import {dateToString, type DownloadReleaseCorrelationDataPoint} from '../../utils/utils';

	export let dataPoints: DownloadReleaseCorrelationDataPoint[];

	let downloadChartEl: HTMLCanvasElement;
	let downloadGrowthChartEl: HTMLCanvasElement;

	onMount(() => {
		const style = getComputedStyle(document.body);
		const accentColor = style.getPropertyValue('--sl-color-accent-high');

		Chart.defaults.borderColor = style.getPropertyValue('--sl-color-hairline-light');
		Chart.defaults.color = style.getPropertyValue('--sl-color-text');

		new Chart(downloadChartEl!, {
			type: 'scatter',
			data: {
				labels: dataPoints.map(d => d.id),
				datasets: [
					{
						label: 'Releases vs. Downloads',
						data: dataPoints.map(d => ({x: d.downloads, y: d.releases, label: d.id})),
						backgroundColor: accentColor,
						borderColor: accentColor,
					},
				]
			},
			options: {
				scales: {
					x: {
						type: 'logarithmic',
						position: 'bottom'
					},
					y: {
						type: 'linear',
						position: 'left',
                        min: 1,
					}
				},
				aspectRatio: 1,
                plugins: {
					tooltip: {
						callbacks: {
							label: (item, data) => {
								return `${item.raw.label} (${item.raw.x}, ${item.raw.y})`;
                            }
                        }
                    }
                }
			}
		});

		new Chart(downloadGrowthChartEl!, {
			type: 'scatter',
			data: {
				labels: dataPoints.map(d => d.id),
				datasets: [
					{
						label: 'Initial Release Time vs. Downloads',
						data: dataPoints.map(d => ({x: d.downloads, y: d.initialReleaseDate, label: d.id})),
						backgroundColor: accentColor,
						borderColor: accentColor,
					},
				]
			},
			options: {
				scales: {
					x: {
						type: 'logarithmic',
						position: 'bottom'
					},
					y: {
						type: 'linear',
						position: 'left',
                        max: Date.now(),
						ticks: {
							stepSize: 1000 * 60 * 60 * 24 * 90,
                            autoSkip: false,
                            includeBounds: false,
							callback: (value, index, values) => {
								const date = new Date(value);
								// if (date.getDate() !== 1) {
								// 	console.log(date);
								// 	return undefined;
                                // }
                                const dateString = dateToString(date);

								return dateString.substring(0, dateString.length - 3);
							}
						}
					}
				},
				aspectRatio: 1,
				plugins: {
					tooltip: {
						callbacks: {
							label: (item, data) => {
								return `${item.raw.label} (${item.raw.x}, ${dateToString(new Date(item.raw.y))})`;
							}
						}
					}
				}
			}
		});
	});
</script>

<style>
    .chart-wrapper {
        width: 100%;
        aspect-ratio: 1;
        position: relative;
    }
</style>

<div class="chart-wrapper">
    <canvas bind:this={downloadChartEl} id="plugin-download-chart"></canvas>
</div>

<div class="chart-wrapper">
    <canvas bind:this={downloadGrowthChartEl} id="plugin-download-growth-chart"></canvas>
</div>