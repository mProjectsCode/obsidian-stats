<script lang="ts">
	import { GridY, Line, Plot, Pointer, Text, Dot } from 'svelteplot';
	import ChartWrapper from '../../ChartWrapper.svelte';
	import type { NamedDataPoint } from '../../../../../../data-wasm/pkg/data_wasm';
	import { formatDate, smooth } from '../chartUtils';

	interface Props {
		dataPoints: NamedDataPoint[];
	}

	const { dataPoints }: Props = $props();

	const mappedDataPoints = dataPoints.map(p => ({
		name: new Date(p.name),
		value: p.value,
	}));

	const smoothedDataPoints = smooth(mappedDataPoints, 'value', 7);
</script>

<ChartWrapper>
	<Plot x={{ label: 'Weeks →' }} y={{ label: '↑ Updates' }} class="no-overflow-clip">
		<GridY />
		<Line data={mappedDataPoints} x="name" y="value" strokeDasharray={'5'} opacity={0.3} />
		<Line data={smoothedDataPoints} x="name" y="value" stroke="var(--sl-color-text-accent)" />
		<Pointer data={mappedDataPoints} x="name" y="value" maxDistance={30}>
			{#snippet children({ data })}
				<Text
					{data}
					fill="var(--sl-color-text-accent)"
					x="name"
					y="value"
					text={d => `${formatDate(d.name)}\n${d.value.toFixed()}`}
					lineAnchor="bottom"
					dy={-7}
				/>
				<Dot {data} x="name" y="value" fill="var(--sl-color-text-accent)" />
			{/snippet}
		</Pointer>
	</Plot>
</ChartWrapper>
