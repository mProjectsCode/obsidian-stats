<script lang="ts">
	import { Plot, Line, Frame, RectX, BrushX, RuleX, Dot, Pointer, Text, AxisX, AxisY } from 'svelteplot';
	import type { DownloadDataPoint, VersionDataPoint } from '../../../../../../data-wasm/pkg/data_wasm';
	import { smooth, toCompactString } from '../chartUtils';

	interface Props {
		dataPoints: DownloadDataPoint[];
		versions?: VersionDataPoint[]; // Optional, for version markers
	}

	const { dataPoints, versions }: Props = $props();

	const mappedData = dataPoints.map(point => {
		return {
			date: new Date(point.date),
			downloads: (point.downloads ?? null) as number,
			delta: (point.delta ?? null) as number,
		};
	});

	const smoothedDelta = smooth(mappedData, 'delta', 2);

	let brush = $state({
		enabled: false,
		x1: new Date(2024, 1, 1),
		x2: new Date(2025, 1, 1),
	});

	let zoomedToYear = $derived(brush.enabled && brush.x1 && brush.x2 ? Math.abs(brush.x2.getTime() - brush.x1.getTime()) < 1000 * 60 * 60 * 24 * 365 : false);

	const filteredData = $derived(brush.enabled ? mappedData.filter(d => d.date >= brush.x1 && d.date <= brush.x2) : mappedData);

	const filteredSmoothedDelta = $derived(brush.enabled ? smoothedDelta.filter(d => d.date >= brush.x1 && d.date <= brush.x2) : smoothedDelta);

	const mappedVersions = versions?.map(version => {
		return {
			date: new Date(version.date),
			version: version.version,
		};
	});

	const filteredVersions = $derived(brush.enabled ? mappedVersions?.filter(v => v.date >= brush.x1 && v.date <= brush.x2) : mappedVersions);

	const undefinedData = undefined as unknown as [];
</script>

<div style="touch-action: none">
	<Plot height={90} x={{ label: '', grid: true }} y={{ axis: false, label: '' }}>
		<Frame opacity={0.4} />
		<Line data={mappedData} x="date" y="downloads" opacity={0.3} />
		{#if brush.enabled}
			<RectX data={undefinedData} {...brush} fill="#33aaee" opacity={0.2} />
			<Line data={filteredData} x="date" y="downloads" />
		{/if}
		<BrushX bind:brush stroke={false} constrainToDomain />
	</Plot>
</div>

<Plot grid x={{ label: 'Date →' }} y={{ label: '↑ Downloads' }} class="no-overflow-clip">
	<AxisX />
	<AxisY />
	<Line data={filteredData} x={'date'} y={'downloads'} stroke={'var(--sl-color-text-accent)'}></Line>
	{#if filteredVersions}
		<RuleX data={filteredVersions} x={'date'} strokeOpacity={0.3} strokeDasharray={'5'} />
	{/if}
	<Pointer data={filteredData} x="date" y="downloads" maxDistance={30}>
		{#snippet children({ data })}
			<Text {data} fill="var(--sl-color-text-accent)" x="date" y="downloads" text={d => toCompactString(d.downloads)} lineAnchor="bottom" dy={-7} />
			<Dot {data} x="date" y="downloads" fill="var(--sl-color-text-accent)" />
		{/snippet}
	</Pointer>
</Plot>

<Plot grid x={{ label: 'Date →' }} y={{ label: '↑ Weekly Delta' }} class="no-overflow-clip">
	<AxisX />
	<AxisY />
	{#if zoomedToYear}
		<Dot data={filteredData} x={'date'} y={'delta'} opacity={0.5} stroke={'var(--sl-color-text-accent)'} />
	{/if}
	<Line data={filteredData} x={'date'} y={'delta'} strokeDasharray={'5'} opacity={0.5} stroke={'var(--sl-color-text-accent)'} />
	<Line data={filteredSmoothedDelta} x={'date'} y={'delta'} stroke={'var(--sl-color-text-accent)'} />
	{#if filteredVersions}
		<RuleX data={filteredVersions} x={'date'} strokeOpacity={0.3} strokeDasharray={'5'} />
	{/if}
	<Pointer data={filteredData} x="date" y="delta" maxDistance={30}>
		{#snippet children({ data })}
			<Text {data} fill="var(--sl-color-text-accent)" x="date" y="delta" text={d => toCompactString(d.delta)} lineAnchor="bottom" dy={-7} />
			<Dot {data} x="date" y="delta" fill="var(--sl-color-text-accent)" />
		{/snippet}
	</Pointer>
</Plot>
