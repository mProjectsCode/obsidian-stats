type Obj<K extends string | number | symbol> = {
	[key in K]: number | null | undefined;
};

export enum SmoothingMode {
	/** Centered simple moving average over a +/- window radius. */
	SimpleMovingAverage = 'simple',
	/** Centered weighted moving average with linearly decaying weights by distance. */
	WeightedMovingAverage = 'weighted',
	/** Exponential moving average (causal / left-looking) with an alpha derived from factor. */
	ExponentialMovingAverage = 'exponential',
}

export function smooth<T extends Obj<K>, K extends keyof T>(data: T[], key: K, factor: number, mode: SmoothingMode = SmoothingMode.WeightedMovingAverage): T[] {
	if (data.length === 0) {
		return data;
	}

	if (mode === SmoothingMode.ExponentialMovingAverage) {
		// If factor is in (0, 1), treat it as alpha directly.
		// Otherwise treat factor as a period-like value and derive alpha as 2/(factor+1).
		let alpha = factor > 0 && factor < 1 ? factor : factor > 0 ? 2 / (factor + 1) : 1;
		alpha = Math.max(0, Math.min(1, alpha));

		let prev: number | undefined;
		return data.map(point => {
			const value = point[key];
			if (value == null) {
				return point;
			}

			prev = prev == null ? value : alpha * value + (1 - alpha) * prev;
			return {
				...point,
				[key]: prev,
			} as T;
		});
	}

	const radius = Math.max(0, Math.floor(factor));

	if (mode === SmoothingMode.WeightedMovingAverage) {
		return data.map((point, index) => {
			if (point[key] == null) {
				return point;
			}

			let weightedSum = 0;
			let totalWeight = 0;
			for (let i = -radius; i <= radius; i++) {
				const j = index + i;
				if (j < 0 || j >= data.length) {
					continue;
				}

				const value = data[j][key];
				if (value == null) {
					continue;
				}

				const weight = radius + 1 - Math.abs(i);
				weightedSum += value * weight;
				totalWeight += weight;
			}

			return {
				...point,
				[key]: totalWeight > 0 ? weightedSum / totalWeight : point[key],
			} as T;
		});
	}

	// Default: SimpleMovingAverage (existing behavior)
	return data.map((point, index) => {
		if (point[key] == null) {
			return point;
		}

		let smoothedDelta = 0;
		let dataPoints = 0;
		for (let i = -radius; i <= radius; i++) {
			const j = index + i;
			if (j < 0 || j >= data.length) {
				continue;
			}

			const value = data[j][key];
			if (value != null) {
				smoothedDelta += value;
				dataPoints++;
			}
		}
		return {
			...point,
			[key]: smoothedDelta / dataPoints,
		} as T;
	});
}

export function toCompactString(datum: number | string | symbol | boolean | Date | object | null | undefined): string {
	if (datum == null) {
		return '';
	}
	if (typeof datum === 'number') {
		return datum.toLocaleString(undefined, { notation: 'compact' });
	}
	if (typeof datum === 'boolean') {
		return datum ? 'Yes' : 'No';
	}
	if (typeof datum === 'symbol') {
		return Symbol.keyFor(datum) || '';
	}
	if (datum instanceof Date) {
		return datum.toLocaleDateString();
	}
	if (typeof datum === 'object') {
		return JSON.stringify(datum);
	}
	return datum;
}

export function navigateToPlugin(id: string): void {
	window.open(`/obsidian-stats/plugins/${id}`, '_self');
}

const monthFormatOptions = {
	year: 'numeric',
	month: 'long',
} as const;

export function formatMonth(date: Date): string {
	return date.toLocaleDateString(undefined, monthFormatOptions);
}

const dateFormatOptions = {
	year: 'numeric',
	month: 'long',
	day: 'numeric',
} as const;

export function formatDate(date: Date): string {
	return date.toLocaleDateString(undefined, dateFormatOptions);
}
