<script lang="ts">
	import { Line, Plot } from 'svelteplot';

	interface Props {
		dataPoints: number[];
	}

	const { dataPoints }: Props = $props();

	const mappedDataPoints = dataPoints.map((inactivity, index) => ({
		inactivity,
		index,
	}));
</script>

<Plot grid x={{ label: 'Inactivity Rank →', type: 'linear' }} y={{ label: '↑ Days of Inactivity' }}>
	<Line data={mappedDataPoints} x="index" y="inactivity" stroke="var(--sl-color-text-accent)" />
</Plot>
