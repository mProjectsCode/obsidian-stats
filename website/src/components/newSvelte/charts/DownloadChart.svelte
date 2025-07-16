<script lang="ts">
    import { Plot, Line, windowY, Dot } from 'svelteplot';

    interface Props {
        dataPoints: {
            date: string; // e.g. "2023-10-01"
            downloads: number; // e.g. 100
            delta: number; // e.g. 10 (change from previous data point)
        }[];
    }

    const {
        dataPoints
    }: Props = $props();

    const mappedData = dataPoints.map(point => {
        return {
            date: new Date(point.date),
            downloads: point.downloads ?? null,
            delta: point.delta ?? null
        };
    });

    const smoothFactor = 7;
    const smoothedDelta = mappedData.map((point, index) => {
        let smoothedDelta = 0;
        let dataPoints = 0;
        for (let i = -smoothFactor; i <= smoothFactor; i++) {
            let j = index + i;

            if (j >= 0 && j < mappedData.length) {
                const value = mappedData[j].delta;
                if (value != null) {
                    smoothedDelta += value;
                    dataPoints++;
                }
            }
        }
        return {
            ...point,
            delta: smoothedDelta / dataPoints
        };
    })
</script>

<Plot grid height={600}>
    <Line data={mappedData} x={"date"} y={"downloads"}></Line>
</Plot>

<Plot grid height={600}>
    <Line data={mappedData} x={"date"} y={"delta"} strokeDasharray={"5"} opacity={0.5}></Line>
    <Line data={smoothedDelta} x={"date"} y={"delta"}></Line>
</Plot>
