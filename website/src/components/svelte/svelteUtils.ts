import Chart from 'chart.js/auto';

interface ChartStyle {
	accent: string;
	line: string;
	text: string;
}

type ChartCreateFunction = (chartStyle: ChartStyle) => Chart;
interface ChartStruct {
	create: ChartCreateFunction;
	chart: Chart | undefined;
}

export class ThemeObserver {
	charts: ChartStruct[] = [];
	observer: MutationObserver | undefined;

	initObserver(): void {
		const targetNode = document.documentElement;

		// Options for the observer (which mutations to observe)
		const config = { attributes: true };

		// Callback function to execute when mutations are observed
		const callback: MutationCallback = (mutationList, observer) => {
			for (const mutation of mutationList) {
				if (mutation.type === 'attributes' && mutation.attributeName === 'data-theme') {
					this.recreateCharts();
				}
			}
		};

		// Create an observer instance linked to the callback function
		this.observer = new MutationObserver(callback);

		// Start observing the target node for configured mutations
		this.observer.observe(targetNode, config);

		// Create the charts once on startup
		this.recreateCharts();
	}

	recreateCharts(): void {
		const style = getComputedStyle(document.body);
		const newStyles: ChartStyle = {
			accent: style.getPropertyValue('--sl-color-accent-high'),
			line: style.getPropertyValue('--sl-color-hairline-light'),
			text: style.getPropertyValue('--sl-color-text'),
		};

		for (const chart of this.charts) {
			chart.chart?.destroy();
			chart.chart = chart.create(newStyles);
		}
	}

	addChart(createChart: ChartCreateFunction): void {
		this.charts.push({
			create: createChart,
			chart: undefined,
		});
	}

	destroy() {
		for (const chart of this.charts) {
			chart.chart?.destroy();
		}
		this.observer?.disconnect();
	}
}
