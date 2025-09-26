<script lang="ts">
	import { Plot, Line, Frame, RectX, BrushX, RuleX, Dot, Pointer, Text, AxisX, AxisY } from 'svelteplot';
	import type { MultiDownloadDataPoint, VersionDataPoint } from '../../../../../../data-wasm/pkg/data_wasm';
	import { smooth, toCompactString } from '../chartUtils';

	interface Props {
		dataPoints: MultiDownloadDataPoint[];
	}

	const { dataPoints }: Props = $props();

	const mappedData = dataPoints.map(point => {
		return {
			date: new Date(point.date),
			category: point.category,
			downloads: (point.downloads ?? null) as number,
			delta: (point.delta ?? null) as number,
		};
	});

	let brush = $state({
		enabled: false,
		x1: new Date(2024, 1, 1),
		x2: new Date(2025, 1, 1),
	});

	let zoomedToYear = $derived(brush.enabled && brush.x1 && brush.x2 ? Math.abs(brush.x2.getTime() - brush.x1.getTime()) < 1000 * 60 * 60 * 24 * 365 : false);

	const filteredData = $derived(brush.enabled ? mappedData.filter(d => d.date >= brush.x1 && d.date <= brush.x2) : mappedData);

	const undefinedData = undefined as unknown as [];
</script>

<div style="touch-action: none">
	<Plot height={90} x={{ label: '', grid: true }} y={{ axis: false, label: '' }}>
		<Frame opacity={0.4} />
		<Line data={mappedData} x="date" y="downloads" stroke="category" opacity={0.3} />
		{#if brush.enabled}
			<RectX data={undefinedData} {...brush} fill="#33aaee" opacity={0.2} />
			<Line data={filteredData} x="date" y="downloads" stroke="category" />
		{/if}
		<BrushX bind:brush stroke={false} constrainToDomain />
	</Plot>
</div>

<Plot grid color={{ legend: true, scheme: 'tableau10' }} x={{ label: 'Date →' }} y={{ label: '↑ Downloads' }} class="no-overflow-clip">
	<AxisX />
	<AxisY />
	<Line data={filteredData} x={'date'} y={'downloads'} stroke={'category'}></Line>
	<Pointer data={filteredData} x="date" y="downloads" maxDistance={30}>
		{#snippet children({ data })}
			<Text {data} fill="category" x="date" y="downloads" text={d => toCompactString(d.downloads)} lineAnchor="bottom" dy={-7} />
			<Dot {data} x="date" y="downloads" fill="category" />
		{/snippet}
	</Pointer>
</Plot>

<Plot grid color={{ legend: true, scheme: 'tableau10' }} x={{ label: 'Date →' }} y={{ label: '↑ Weekly Delta' }} class="no-overflow-clip">
	<AxisX />
	<AxisY />
	{#if zoomedToYear}
		<Dot data={filteredData} x={'date'} y={'delta'} stroke={'category'} />
	{/if}
	<Line data={filteredData} x={'date'} y={'delta'} stroke={'category'} />
	<Pointer data={filteredData} x="date" y="delta" maxDistance={30}>
		{#snippet children({ data })}
			<Text {data} fill="category" x="date" y="delta" text={d => toCompactString(d.delta)} lineAnchor="bottom" dy={-7} />
			<Dot {data} x="date" y="delta" fill="category" />
		{/snippet}
	</Pointer>
</Plot>
