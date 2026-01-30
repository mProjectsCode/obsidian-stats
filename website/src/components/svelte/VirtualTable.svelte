<script lang="ts" generics="T extends { id?: number | string }">
	interface Props {
		items: T[];
		colCount: number;
		colWidths?: string[];
		minWidth?: string;
		itemHeight?: number;
		overscan?: number;
		height?: number;
		header: () => any;
		row: (item: T, index: number) => any;
	}

	let { items, colCount, colWidths, minWidth, itemHeight = 40, overscan = 8, height = 600, header, row }: Props = $props();

	let scrollTop = $state(0);
	let viewportHeight = $state(0);
	let scrollbarWidth = $state(0);
	let scrollContainer: HTMLDivElement;
	let effectiveItemHeight = $derived(itemHeight);

	let gridTemplateColumns = $derived.by(() => {
		if (colWidths && colWidths.length === colCount) {
			return colWidths.join(' ');
		}
		return `repeat(${colCount}, minmax(0, 1fr))`;
	});

	let visibleRange = $derived.by(() => {
		const start = Math.max(0, Math.floor(scrollTop / effectiveItemHeight) - overscan);
		const visibleCount = Math.ceil(viewportHeight / effectiveItemHeight);
		const end = Math.min(items.length, start + visibleCount + overscan * 2);
		return { start, end };
	});

	let visibleItems = $derived(
		items.slice(visibleRange.start, visibleRange.end).map((item, i) => ({
			item,
			index: visibleRange.start + i,
		})),
	);

	let topSpacerHeight = $derived(visibleRange.start * effectiveItemHeight);
	let bottomSpacerHeight = $derived((items.length - visibleRange.end) * effectiveItemHeight);

	function handleScroll(e: Event): void {
		scrollTop = (e.target as HTMLDivElement).scrollTop;
	}

	function measureViewport(): void {
		if (scrollContainer) {
			viewportHeight = scrollContainer.clientHeight;
			scrollbarWidth = Math.max(0, scrollContainer.offsetWidth - scrollContainer.clientWidth);
		}
	}

	$effect(() => {
		measureViewport();
		window.addEventListener('resize', measureViewport);
		return () => window.removeEventListener('resize', measureViewport);
	});

	$effect(() => {
		// If filtering changes whether a vertical scrollbar is present,
		// keep the header padding in sync.
		items.length;
		measureViewport();
	});
</script>

<div class="vt-hscroll">
	<div class="vt-inner" style={minWidth ? `min-width: max(100%, ${minWidth});` : ''}>
		<div class="vt-row vt-header" style="grid-template-columns: {gridTemplateColumns}; padding-right: {scrollbarWidth}px;">
			{@render header()}
		</div>

		<div class="vt-vscroll" style="height: {height}px;" bind:this={scrollContainer} onscroll={handleScroll}>
			<div class="vt-spacer" style="height: {topSpacerHeight}px;"></div>
			{#each visibleItems as { item, index } (items[index]?.id || index)}
				<div class="vt-row" style="grid-template-columns: {gridTemplateColumns}; height: {effectiveItemHeight}px;">
					{@render row(item, index)}
				</div>
			{/each}
			<div class="vt-spacer" style="height: {bottomSpacerHeight}px;"></div>
		</div>
	</div>
</div>

<style>
	.vt-hscroll {
		width: 100%;
		overflow-x: auto;
		overflow-y: hidden;
		margin: 0;
	}

	.vt-inner {
		width: max-content;
		min-width: 100%;
	}

	.vt-vscroll {
		overflow-y: auto;
		overflow-x: hidden;
		margin: 0;
	}

	.vt-row {
		margin: 0;
		display: grid;
		align-items: center;
	}

	.vt-header {
		position: sticky;
		top: 0;
		z-index: 2;
		background: var(--sl-color-bg);
		border-bottom: 1px solid var(--sl-color-gray-5);
	}

	/* Apply "table-like" row separators without changing theme colors */
	.vt-row:not(.vt-header) {
		border-bottom: 1px solid var(--sl-color-gray-5);
	}

	/* Fixed-height, single-line cells with ellipsis */
	:global(.vt-cell) {
		margin: 0;
		display: flex;
		align-items: center;
		height: 100%;
		min-width: 0;
		white-space: nowrap;
		overflow: hidden;
		overflow-wrap: normal;
		word-break: normal;
		padding: 0.5rem 0.75rem;
	}

	:global(.vt-cell :is(a, span)) {
		display: block;
		min-width: 0;
		line-height: inherit;
		max-width: 100%;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.vt-spacer {
		pointer-events: none;
	}
</style>
