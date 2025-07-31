<script lang="ts">
	import { Dot, Line, Plot, Pointer, Text } from 'svelteplot';
	import type { CountMonthlyDataPoint } from '../../../../../../data-wasm/pkg/data_wasm';
	import { smooth } from '../chartUtils';
	import { typeToString, type ItemType } from '../../../../utils/misc';

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

	const smoothedData = smooth(mappedDataPoints, 'new_removed', 3);
</script>

<Plot grid y={{ label: `↑ ${typeToString(type, true, true)} Removed per Month` }}>
	<Line data={smoothedData} x="date" y="new_removed" stroke="var(--sl-color-text-accent)" />
	<Line data={mappedDataPoints} x="date" y="new_removed" strokeDasharray={'5'} opacity={0.3} />
	<Dot data={mappedDataPoints} x="date" y="new_removed" opacity={0.3} />
	<Pointer data={mappedDataPoints} x="date" y="new_removed" maxDistance={30}>
		{#snippet children({ data })}
			<Text {data} fill="var(--sl-color-text-accent)" x="date" y="new_removed" text={d => d.new_removed.toFixed()} lineAnchor="bottom" dy={-7} />
			<Dot {data} x="date" y="new_removed" fill="var(--sl-color-text-accent)" />
		{/snippet}
	</Pointer>
</Plot>
