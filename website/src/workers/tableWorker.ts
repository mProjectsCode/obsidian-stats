interface OverviewDataPoint {
	id: string;
	name: string;
	author: string;
	repo: string;
	added_commit: {
		date: string;
		hash: string;
	};
	removed_commit?: {
		date: string;
		hash: string;
	} | null;
}

interface InitMessage {
	type: 'init';
	data: OverviewDataPoint[];
}

interface ProcessMessage {
	type: 'process';
	sortBy: 'id' | 'name' | 'author' | 'repo' | 'added' | 'removed';
	ascending: boolean;
	searchQuery: string;
}

type WorkerMessage = InitMessage | ProcessMessage;

interface WorkerResponse {
	type: 'result';
	indices: number[];
}

let data: OverviewDataPoint[] = [];

type SearchField = 'id' | 'name' | 'author' | 'repo';

interface SearchToken {
	field?: SearchField;
	term: string;
}

function normalizeField(field: string): SearchField | undefined {
	switch (field) {
		case 'id':
			return 'id';
		case 'name':
		case 'title':
			return 'name';
		case 'author':
			return 'author';
		case 'repo':
			return 'repo';
		default:
			return undefined;
	}
}

function tokenizeSearchQuery(searchQuery: string): SearchToken[] {
	const rawTokens = searchQuery.toLowerCase().trim().split(/\s+/).filter(Boolean);

	return rawTokens
		.map(raw => {
			const colonIdx = raw.indexOf(':');
			if (colonIdx > 0) {
				const maybeField = raw.slice(0, colonIdx);
				const term = raw.slice(colonIdx + 1).trim();
				const field = normalizeField(maybeField);
				if (field && term) return { field, term };
			}

			return { term: raw };
		})
		.filter(t => t.term.length > 0);
}

function matchesSearchQuery(datum: OverviewDataPoint, tokens: SearchToken[]): boolean {
	if (tokens.length === 0) return true;

	const fields: Record<SearchField, string> = {
		id: datum.id.toLowerCase(),
		name: datum.name.toLowerCase(),
		author: datum.author.toLowerCase(),
		repo: datum.repo.toLowerCase(),
	};

	// AND across tokens.
	// - Unscoped token: OR across all fields
	// - Scoped token: match only that field
	return tokens.every(({ field, term }) => {
		if (field) return fields[field].includes(term);
		return (Object.values(fields) as string[]).some(h => h.includes(term));
	});
}

function getSortValue(datum: OverviewDataPoint, sortBy: string): string {
	switch (sortBy) {
		case 'id':
			return datum.id.toLowerCase();
		case 'name':
			return datum.name.toLowerCase();
		case 'author':
			return datum.author.toLowerCase();
		case 'repo':
			return datum.repo;
		case 'added':
			return datum.added_commit.date;
		case 'removed':
			return datum.removed_commit?.date ?? '';
		default:
			return '';
	}
}

self.onmessage = (e: MessageEvent<WorkerMessage>) => {
	const message = e.data;

	if (message.type === 'init') {
		data = message.data;
		return;
	}

	if (message.type !== 'process') return;

	const { sortBy, ascending, searchQuery } = message;

	// Create array of indices
	const indices = data.map((_, i) => i);

	// Sort the indices based on data values
	const sortModifier = ascending ? -1 : 1;
	indices.sort((aIdx, bIdx) => {
		const _a = getSortValue(data[aIdx], sortBy);
		const _b = getSortValue(data[bIdx], sortBy);

		if (_a < _b) {
			return -1 * sortModifier;
		} else if (_a > _b) {
			return 1 * sortModifier;
		} else {
			return 0;
		}
	});

	// Filter the sorted indices
	const tokens = tokenizeSearchQuery(searchQuery);
	const filteredIndices = tokens.length ? indices.filter(idx => matchesSearchQuery(data[idx], tokens)) : indices;

	const response: WorkerResponse = {
		type: 'result',
		indices: filteredIndices,
	};

	self.postMessage(response);
};
