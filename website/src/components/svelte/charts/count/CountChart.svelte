<script lang="ts">
	import { Dot, Line, Plot, Pointer, Text } from 'svelteplot';
	import type { CountMonthlyDataPoint } from '../../../../../../data-wasm/pkg/data_wasm';
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

	const totalStroke = `Listed ${typeToString(type, true, true)}`;
	const totalWithRemovedStroke = `Including Removed ${typeToString(type, true, true)}`;
</script>

<Plot grid color={{ legend: true, scheme: 'tableau10' }} class="no-overflow-clip" y={{ label: `↑ ${typeToString(type, false, true)} Count` }}>
	<Line data={mappedDataPoints} x="date" y="total" stroke={totalStroke} marker="dot" />
	<Line data={mappedDataPoints} x="date" y="total_with_removed" stroke={totalWithRemovedStroke} marker="dot" />

	<Pointer data={mappedDataPoints} x="date" maxDistance={5}>
		{#snippet children({ data })}
			<Dot {data} x="date" y="total" fill={totalStroke} />
			<Dot {data} x="date" y="total_with_removed" fill={totalWithRemovedStroke} />
			<Text {data} fill={totalStroke} x="date" y="total" text={d => d.total.toFixed()} lineAnchor="bottom" dy={-7} />
			<Text {data} fill={totalWithRemovedStroke} x="date" y="total_with_removed" text={d => d.total_with_removed.toFixed()} lineAnchor="bottom" dy={-7} />
		{/snippet}
	</Pointer>
</Plot>
