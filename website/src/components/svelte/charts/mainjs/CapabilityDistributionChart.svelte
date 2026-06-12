<script lang="ts">
	import { BarX, GridX, Plot, Text } from 'svelteplot';
	import type { NamedDataPoint } from '../../../../../../data-wasm/pkg/data_wasm';
	import ChartWrapper from '../../ChartWrapper.svelte';
	import { toCompactString } from '../chartUtils';

	interface Props {
		dataPoints: NamedDataPoint[];
	}

	const { dataPoints }: Props = $props();

	const mappedDataPoints = $derived.by(() =>
		dataPoints
			.map(point => ({
				capability: point.name,
				value: point.value,
			}))
			.sort((a, b) => a.value - b.value || b.capability.localeCompare(a.capability)),
	);

	const capabilityOrder = $derived(mappedDataPoints.map(point => point.capability));
	const maxValue = $derived(Math.max(0, ...mappedDataPoints.map(point => point.value)));
</script>

<ChartWrapper>
	<Plot x={{ label: 'Plugins →', domain: [0, Math.max(1, maxValue * 1.12)] }} y={{ type: 'band', label: '', domain: capabilityOrder }} class="no-overflow-clip">
		<GridX />
		<BarX data={mappedDataPoints} x="value" y="capability" fill="var(--sl-color-text-accent)" />
		<Text
			data={mappedDataPoints}
			x="value"
			y="capability"
			text={point => toCompactString(point.value)}
			fill="var(--sl-color-text-foreground)"
			textAnchor="start"
			dx={6}
		/>
	</Plot>
</ChartWrapper>
