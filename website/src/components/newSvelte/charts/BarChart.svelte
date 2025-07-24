<script lang="ts">
	import { BarY, Dot, Line, Plot } from 'svelteplot';
	import type { NamedDataPoint } from '../../../../../data-wasm/pkg/data_wasm';

	interface Props {
		dataPoints: NamedDataPoint[];
		xLabel: string;
		yLabel: string;
		yDomain?: [number, number];
	}

	const { dataPoints, xLabel, yLabel, yDomain }: Props = $props();

	const mappedDataPoints = dataPoints.map((point, index) => {
	    return {
	        index: index,
	        label: point.name,
	        value: point.value,
	    };
	});

    function sortData(a: any, b: any) {
        return b.value - a.value; // Sort in descending order
    }
</script>

<Plot grid x={{ type: 'band', label: `${xLabel} →` }} y={{ label: `↑ ${yLabel}`, domain: yDomain }} class="no-overflow">
	<BarY data={mappedDataPoints} x="label" y="value" fill="white" sort={sortData} />
</Plot>