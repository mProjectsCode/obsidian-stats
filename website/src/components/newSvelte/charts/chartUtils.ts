type Obj<K extends string | number | symbol> = {
    [key in K]: number;
}

export function smooth<T extends Obj<K>, K extends keyof T>(data: T[], key: K, factor: number): T[] {
    return data.map((point, index) => {
        if (point[key] == null) {
            return point;
        }

        let smoothedDelta = 0;
        let dataPoints = 0;
        for (let i = -factor; i <= factor; i++) {
            let j = index + i;

            if (j >= 0 && j < data.length) {
                const value = data[j][key];
                if (value != null) {
                    smoothedDelta += value;
                    dataPoints++;
                }
            }
        }
        return {
            ...point,
            [key]: smoothedDelta / dataPoints
        };
    });
}