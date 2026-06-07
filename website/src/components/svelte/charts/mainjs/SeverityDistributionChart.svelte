<script lang="ts">
	import { BarY, GridY, Plot } from 'svelteplot';
	import type { StackedNamedDataPoint } from '../../../../../../data-wasm/pkg/data_wasm';
	import ChartWrapper from '../../ChartWrapper.svelte';

	interface Props {
		dataPoints: StackedNamedDataPoint[];
	}

	const { dataPoints }: Props = $props();

	const severityOrder = ['Info', 'Notice', 'Warning', 'Critical'];
	const severityColors = ['var(--sl-color-text-accent)', 'var(--sl-color-green-high)', 'var(--sl-color-orange)', 'var(--sl-color-red)'];

	const categoryTotals = $derived.by(() => {
		const totals = new Map<string, number>();
		for (const point of dataPoints) {
			totals.set(point.name, (totals.get(point.name) ?? 0) + point.value);
		}
		return totals;
	});

	const mappedDataPoints = $derived.by(() =>
		dataPoints
			.map(point => ({
				category: point.name,
				value: point.value,
				severity: point.layer,
			}))
			.sort(
				(a, b) =>
					(categoryTotals.get(b.category) ?? 0) - (categoryTotals.get(a.category) ?? 0) ||
					a.category.localeCompare(b.category) ||
					severityOrder.indexOf(a.severity) - severityOrder.indexOf(b.severity),
			),
	);
</script>

<ChartWrapper>
	<Plot
		color={{ legend: true, domain: severityOrder, scheme: severityColors }}
		x={{ type: 'band', label: 'Category →', tickRotate: 45 }}
		y={{ label: '↑ Plugins' }}
		class="no-overflow-clip"
	>
		<GridY />
		<BarY data={mappedDataPoints} x="category" y="value" fill="severity" />
	</Plot>
</ChartWrapper>
