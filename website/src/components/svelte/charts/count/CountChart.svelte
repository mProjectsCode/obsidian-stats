<script lang="ts">
	import { Dot, Line, Plot, Pointer, Text } from 'svelteplot';
	import type { CountMonthlyDataPoint } from '../../../../../../data-wasm/pkg/data_wasm';
	import { typeToString, type ItemType } from '../../../../utils/misc';
	import ChartWrapper from '../../ChartWrapper.svelte';
	import { formatMonth } from '../chartUtils';

	interface Props {
		dataPoints: CountMonthlyDataPoint[];
		type: ItemType;
	}

	const { dataPoints, type }: Props = $props();

	const mappedDataPoints = dataPoints
		.filter(x => x.total_with_removed > 0)
		.map(point => {
			return {
				date: new Date(point.date),
				total: point.total,
				total_with_removed: point.total_with_removed,
				new: point.new,
				new_removed: point.new_removed,
			};
		});
</script>

<ChartWrapper>
	<Plot grid class="no-overflow-clip" y={{ label: `↑ ${typeToString(type, false, true)} Count` }}>
		<Line data={mappedDataPoints} x="date" y="total" stroke="var(--sl-color-text-accent)" marker="dot" />

		<Pointer data={mappedDataPoints} x="date" maxDistance={5}>
			{#snippet children({ data })}
				<Dot {data} x="date" y="total" fill="var(--sl-color-text-accent)" />
				<Text
					{data}
					fill="var(--sl-color-text-accent)"
					x="date"
					y="total"
					text={d => `${formatMonth(d.date)}\n${d.total.toFixed()}`}
					lineAnchor="bottom"
					dy={-7}
				/>
			{/snippet}
		</Pointer>
	</Plot>
</ChartWrapper>
