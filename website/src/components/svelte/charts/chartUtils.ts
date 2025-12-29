type Obj<K extends string | number | symbol> = {
	[key in K]: number;
};

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
			[key]: smoothedDelta / dataPoints,
		};
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
