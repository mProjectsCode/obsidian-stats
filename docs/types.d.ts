declare module 'astro:content' {
	interface Render {
		'.mdx': Promise<{
			Content: import('astro').MarkdownInstance<{}>['Content'];
			headings: import('astro').MarkdownHeading[];
			remarkPluginFrontmatter: Record<string, any>;
		}>;
	}
}

declare module 'astro:content' {
	interface Render {
		'.md': Promise<{
			Content: import('astro').MarkdownInstance<{}>['Content'];
			headings: import('astro').MarkdownHeading[];
			remarkPluginFrontmatter: Record<string, any>;
		}>;
	}
}

declare module 'astro:content' {
	export { z } from 'astro/zod';

	type Flatten<T> = T extends { [K: string]: infer U } ? U : never;

	export type CollectionKey = keyof AnyEntryMap;
	export type CollectionEntry<C extends CollectionKey> = Flatten<AnyEntryMap[C]>;

	export type ContentCollectionKey = keyof ContentEntryMap;
	export type DataCollectionKey = keyof DataEntryMap;

	// This needs to be in sync with ImageMetadata
	export type ImageFunction = () => import('astro/zod').ZodObject<{
		src: import('astro/zod').ZodString;
		width: import('astro/zod').ZodNumber;
		height: import('astro/zod').ZodNumber;
		format: import('astro/zod').ZodUnion<
			[
				import('astro/zod').ZodLiteral<'png'>,
				import('astro/zod').ZodLiteral<'jpg'>,
				import('astro/zod').ZodLiteral<'jpeg'>,
				import('astro/zod').ZodLiteral<'tiff'>,
				import('astro/zod').ZodLiteral<'webp'>,
				import('astro/zod').ZodLiteral<'gif'>,
				import('astro/zod').ZodLiteral<'svg'>,
				import('astro/zod').ZodLiteral<'avif'>,
			]
		>;
	}>;

	type BaseSchemaWithoutEffects =
		| import('astro/zod').AnyZodObject
		| import('astro/zod').ZodUnion<[BaseSchemaWithoutEffects, ...BaseSchemaWithoutEffects[]]>
		| import('astro/zod').ZodDiscriminatedUnion<string, import('astro/zod').AnyZodObject[]>
		| import('astro/zod').ZodIntersection<BaseSchemaWithoutEffects, BaseSchemaWithoutEffects>;

	type BaseSchema =
		| BaseSchemaWithoutEffects
		| import('astro/zod').ZodEffects<BaseSchemaWithoutEffects>;

	export type SchemaContext = { image: ImageFunction };

	type DataCollectionConfig<S extends BaseSchema> = {
		type: 'data';
		schema?: S | ((context: SchemaContext) => S);
	};

	type ContentCollectionConfig<S extends BaseSchema> = {
		type?: 'content';
		schema?: S | ((context: SchemaContext) => S);
	};

	type CollectionConfig<S> = ContentCollectionConfig<S> | DataCollectionConfig<S>;

	export function defineCollection<S extends BaseSchema>(
		input: CollectionConfig<S>
	): CollectionConfig<S>;

	type AllValuesOf<T> = T extends any ? T[keyof T] : never;
	type ValidContentEntrySlug<C extends keyof ContentEntryMap> = AllValuesOf<
		ContentEntryMap[C]
	>['slug'];

	export function getEntryBySlug<
		C extends keyof ContentEntryMap,
		E extends ValidContentEntrySlug<C> | (string & {}),
	>(
		collection: C,
		// Note that this has to accept a regular string too, for SSR
		entrySlug: E
	): E extends ValidContentEntrySlug<C>
		? Promise<CollectionEntry<C>>
		: Promise<CollectionEntry<C> | undefined>;

	export function getDataEntryById<C extends keyof DataEntryMap, E extends keyof DataEntryMap[C]>(
		collection: C,
		entryId: E
	): Promise<CollectionEntry<C>>;

	export function getCollection<C extends keyof AnyEntryMap, E extends CollectionEntry<C>>(
		collection: C,
		filter?: (entry: CollectionEntry<C>) => entry is E
	): Promise<E[]>;
	export function getCollection<C extends keyof AnyEntryMap>(
		collection: C,
		filter?: (entry: CollectionEntry<C>) => unknown
	): Promise<CollectionEntry<C>[]>;

	export function getEntry<
		C extends keyof ContentEntryMap,
		E extends ValidContentEntrySlug<C> | (string & {}),
	>(entry: {
		collection: C;
		slug: E;
	}): E extends ValidContentEntrySlug<C>
		? Promise<CollectionEntry<C>>
		: Promise<CollectionEntry<C> | undefined>;
	export function getEntry<
		C extends keyof DataEntryMap,
		E extends keyof DataEntryMap[C] | (string & {}),
	>(entry: {
		collection: C;
		id: E;
	}): E extends keyof DataEntryMap[C]
		? Promise<DataEntryMap[C][E]>
		: Promise<CollectionEntry<C> | undefined>;
	export function getEntry<
		C extends keyof ContentEntryMap,
		E extends ValidContentEntrySlug<C> | (string & {}),
	>(
		collection: C,
		slug: E
	): E extends ValidContentEntrySlug<C>
		? Promise<CollectionEntry<C>>
		: Promise<CollectionEntry<C> | undefined>;
	export function getEntry<
		C extends keyof DataEntryMap,
		E extends keyof DataEntryMap[C] | (string & {}),
	>(
		collection: C,
		id: E
	): E extends keyof DataEntryMap[C]
		? Promise<DataEntryMap[C][E]>
		: Promise<CollectionEntry<C> | undefined>;

	/** Resolve an array of entry references from the same collection */
	export function getEntries<C extends keyof ContentEntryMap>(
		entries: {
			collection: C;
			slug: ValidContentEntrySlug<C>;
		}[]
	): Promise<CollectionEntry<C>[]>;
	export function getEntries<C extends keyof DataEntryMap>(
		entries: {
			collection: C;
			id: keyof DataEntryMap[C];
		}[]
	): Promise<CollectionEntry<C>[]>;

	export function reference<C extends keyof AnyEntryMap>(
		collection: C
	): import('astro/zod').ZodEffects<
		import('astro/zod').ZodString,
		C extends keyof ContentEntryMap
			? {
					collection: C;
					slug: ValidContentEntrySlug<C>;
			  }
			: {
					collection: C;
					id: keyof DataEntryMap[C];
			  }
	>;
	// Allow generic `string` to avoid excessive type errors in the config
	// if `dev` is not running to update as you edit.
	// Invalid collection names will be caught at build time.
	export function reference<C extends string>(
		collection: C
	): import('astro/zod').ZodEffects<import('astro/zod').ZodString, never>;

	type ReturnTypeOrOriginal<T> = T extends (...args: any[]) => infer R ? R : T;
	type InferEntrySchema<C extends keyof AnyEntryMap> = import('astro/zod').infer<
		ReturnTypeOrOriginal<Required<ContentConfig['collections'][C]>['schema']>
	>;

	type ContentEntryMap = {
		"docs": {
"home/about.md": {
	id: "home/about.md";
  slug: "home/about";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".md"] };
"index.mdx": {
	id: "index.mdx";
  slug: "index";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"pluginStats/downloads.mdx": {
	id: "pluginStats/downloads.mdx";
  slug: "pluginstats/downloads";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"pluginStats/releases.mdx": {
	id: "pluginStats/releases.mdx";
  slug: "pluginstats/releases";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"pluginStats/retiredPlugins.mdx": {
	id: "pluginStats/retiredPlugins.mdx";
  slug: "pluginstats/retiredplugins";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/13th-age-statblocks.mdx": {
	id: "plugins/13th-age-statblocks.mdx";
  slug: "plugins/13th-age-statblocks";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/2hop-links-plus.mdx": {
	id: "plugins/2hop-links-plus.mdx";
  slug: "plugins/2hop-links-plus";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/3d-graph-new.mdx": {
	id: "plugins/3d-graph-new.mdx";
  slug: "plugins/3d-graph-new";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/3d-graph.mdx": {
	id: "plugins/3d-graph.mdx";
  slug: "plugins/3d-graph";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/DEVONlink-obsidian.mdx": {
	id: "plugins/DEVONlink-obsidian.mdx";
  slug: "plugins/devonlink-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/OA-file-hider.mdx": {
	id: "plugins/OA-file-hider.mdx";
  slug: "plugins/oa-file-hider";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/ObsidianAnkiSync.mdx": {
	id: "plugins/ObsidianAnkiSync.mdx";
  slug: "plugins/obsidianankisync";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/abbreviations.mdx": {
	id: "plugins/abbreviations.mdx";
  slug: "plugins/abbreviations";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/actions-uri.mdx": {
	id: "plugins/actions-uri.mdx";
  slug: "plugins/actions-uri";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/adamantine-pick.mdx": {
	id: "plugins/adamantine-pick.mdx";
  slug: "plugins/adamantine-pick";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/adjacency-matrix-exporter.mdx": {
	id: "plugins/adjacency-matrix-exporter.mdx";
  slug: "plugins/adjacency-matrix-exporter";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/adjacency-matrix-maker.mdx": {
	id: "plugins/adjacency-matrix-maker.mdx";
  slug: "plugins/adjacency-matrix-maker";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/advanced-cursors.mdx": {
	id: "plugins/advanced-cursors.mdx";
  slug: "plugins/advanced-cursors";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/advanced-merger.mdx": {
	id: "plugins/advanced-merger.mdx";
  slug: "plugins/advanced-merger";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/advanced-paste.mdx": {
	id: "plugins/advanced-paste.mdx";
  slug: "plugins/advanced-paste";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/advanced-random-note.mdx": {
	id: "plugins/advanced-random-note.mdx";
  slug: "plugins/advanced-random-note";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/advanced-toolbar.mdx": {
	id: "plugins/advanced-toolbar.mdx";
  slug: "plugins/advanced-toolbar";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/ai-assistant.mdx": {
	id: "plugins/ai-assistant.mdx";
  slug: "plugins/ai-assistant";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/ai-commander.mdx": {
	id: "plugins/ai-commander.mdx";
  slug: "plugins/ai-commander";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/ai-editor.mdx": {
	id: "plugins/ai-editor.mdx";
  slug: "plugins/ai-editor";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/ai-mentor.mdx": {
	id: "plugins/ai-mentor.mdx";
  slug: "plugins/ai-mentor";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/ai-research-assistant.mdx": {
	id: "plugins/ai-research-assistant.mdx";
  slug: "plugins/ai-research-assistant";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/ai-summary.mdx": {
	id: "plugins/ai-summary.mdx";
  slug: "plugins/ai-summary";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/ai-tools.mdx": {
	id: "plugins/ai-tools.mdx";
  slug: "plugins/ai-tools";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/air-quotes.mdx": {
	id: "plugins/air-quotes.mdx";
  slug: "plugins/air-quotes";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/alx-folder-note-folderv.mdx": {
	id: "plugins/alx-folder-note-folderv.mdx";
  slug: "plugins/alx-folder-note-folderv";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/alx-folder-note.mdx": {
	id: "plugins/alx-folder-note.mdx";
  slug: "plugins/alx-folder-note";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/android-nomedia.mdx": {
	id: "plugins/android-nomedia.mdx";
  slug: "plugins/android-nomedia";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/anki-sync-plus.mdx": {
	id: "plugins/anki-sync-plus.mdx";
  slug: "plugins/anki-sync-plus";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/antidote-grammar-checker-integration.mdx": {
	id: "plugins/antidote-grammar-checker-integration.mdx";
  slug: "plugins/antidote-grammar-checker-integration";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/any-block.mdx": {
	id: "plugins/any-block.mdx";
  slug: "plugins/any-block";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/aosr.mdx": {
	id: "plugins/aosr.mdx";
  slug: "plugins/aosr";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/api-request.mdx": {
	id: "plugins/api-request.mdx";
  slug: "plugins/api-request";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/apl-render.mdx": {
	id: "plugins/apl-render.mdx";
  slug: "plugins/apl-render";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/apple-books-highlights.mdx": {
	id: "plugins/apple-books-highlights.mdx";
  slug: "plugins/apple-books-highlights";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/aprils-automatic-timelines.mdx": {
	id: "plugins/aprils-automatic-timelines.mdx";
  slug: "plugins/aprils-automatic-timelines";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/arcana.mdx": {
	id: "plugins/arcana.mdx";
  slug: "plugins/arcana";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/archive-to-single-note.mdx": {
	id: "plugins/archive-to-single-note.mdx";
  slug: "plugins/archive-to-single-note";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/archwiki-reader.mdx": {
	id: "plugins/archwiki-reader.mdx";
  slug: "plugins/archwiki-reader";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/arena.mdx": {
	id: "plugins/arena.mdx";
  slug: "plugins/arena";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/arrows.mdx": {
	id: "plugins/arrows.mdx";
  slug: "plugins/arrows";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/askify-obsidian-sync.mdx": {
	id: "plugins/askify-obsidian-sync.mdx";
  slug: "plugins/askify-obsidian-sync";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/at-people.mdx": {
	id: "plugins/at-people.mdx";
  slug: "plugins/at-people";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/at-symbol-linking.mdx": {
	id: "plugins/at-symbol-linking.mdx";
  slug: "plugins/at-symbol-linking";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/attachment-management.mdx": {
	id: "plugins/attachment-management.mdx";
  slug: "plugins/attachment-management";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/attachment-manager.mdx": {
	id: "plugins/attachment-manager.mdx";
  slug: "plugins/attachment-manager";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/attachment-uploader.mdx": {
	id: "plugins/attachment-uploader.mdx";
  slug: "plugins/attachment-uploader";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/auto-anki.mdx": {
	id: "plugins/auto-anki.mdx";
  slug: "plugins/auto-anki";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/auto-archive.mdx": {
	id: "plugins/auto-archive.mdx";
  slug: "plugins/auto-archive";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/auto-card-link.mdx": {
	id: "plugins/auto-card-link.mdx";
  slug: "plugins/auto-card-link";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/auto-class.mdx": {
	id: "plugins/auto-class.mdx";
  slug: "plugins/auto-class";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/auto-classifier.mdx": {
	id: "plugins/auto-classifier.mdx";
  slug: "plugins/auto-classifier";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/auto-displaystyle-inline-math.mdx": {
	id: "plugins/auto-displaystyle-inline-math.mdx";
  slug: "plugins/auto-displaystyle-inline-math";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/auto-filename.mdx": {
	id: "plugins/auto-filename.mdx";
  slug: "plugins/auto-filename";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/auto-front-matter.mdx": {
	id: "plugins/auto-front-matter.mdx";
  slug: "plugins/auto-front-matter";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/auto-glossary.mdx": {
	id: "plugins/auto-glossary.mdx";
  slug: "plugins/auto-glossary";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/auto-hide-cursor.mdx": {
	id: "plugins/auto-hide-cursor.mdx";
  slug: "plugins/auto-hide-cursor";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/auto-hyperlink.mdx": {
	id: "plugins/auto-hyperlink.mdx";
  slug: "plugins/auto-hyperlink";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/auto-journal.mdx": {
	id: "plugins/auto-journal.mdx";
  slug: "plugins/auto-journal";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/auto-literature.mdx": {
	id: "plugins/auto-literature.mdx";
  slug: "plugins/auto-literature";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/auto-moc.mdx": {
	id: "plugins/auto-moc.mdx";
  slug: "plugins/auto-moc";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/auto-note-mover.mdx": {
	id: "plugins/auto-note-mover.mdx";
  slug: "plugins/auto-note-mover";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/auto-reading-mode.mdx": {
	id: "plugins/auto-reading-mode.mdx";
  slug: "plugins/auto-reading-mode";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/auto-tag.mdx": {
	id: "plugins/auto-tag.mdx";
  slug: "plugins/auto-tag";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/auto-template-trigger.mdx": {
	id: "plugins/auto-template-trigger.mdx";
  slug: "plugins/auto-template-trigger";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/automatic-list-styles.mdx": {
	id: "plugins/automatic-list-styles.mdx";
  slug: "plugins/automatic-list-styles";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/automatic-table-of-contents.mdx": {
	id: "plugins/automatic-table-of-contents.mdx";
  slug: "plugins/automatic-table-of-contents";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/automatic-tags.mdx": {
	id: "plugins/automatic-tags.mdx";
  slug: "plugins/automatic-tags";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/ava.mdx": {
	id: "plugins/ava.mdx";
  slug: "plugins/ava";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/avatar.mdx": {
	id: "plugins/avatar.mdx";
  slug: "plugins/avatar";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/aw-watcher-obsidian.mdx": {
	id: "plugins/aw-watcher-obsidian.mdx";
  slug: "plugins/aw-watcher-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/awesome-brain-manager.mdx": {
	id: "plugins/awesome-brain-manager.mdx";
  slug: "plugins/awesome-brain-manager";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/awesome-image.mdx": {
	id: "plugins/awesome-image.mdx";
  slug: "plugins/awesome-image";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/awesome-reader.mdx": {
	id: "plugins/awesome-reader.mdx";
  slug: "plugins/awesome-reader";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/babashka.mdx": {
	id: "plugins/babashka.mdx";
  slug: "plugins/babashka";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/background-image.mdx": {
	id: "plugins/background-image.mdx";
  slug: "plugins/background-image";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/backlink-cache.mdx": {
	id: "plugins/backlink-cache.mdx";
  slug: "plugins/backlink-cache";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/badges.mdx": {
	id: "plugins/badges.mdx";
  slug: "plugins/badges";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/barcode-generator.mdx": {
	id: "plugins/barcode-generator.mdx";
  slug: "plugins/barcode-generator";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/battlesnake-viewer.mdx": {
	id: "plugins/battlesnake-viewer.mdx";
  slug: "plugins/battlesnake-viewer";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/bbawj-semantic-search.mdx": {
	id: "plugins/bbawj-semantic-search.mdx";
  slug: "plugins/bbawj-semantic-search";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/beeminder-word-count-plugin.mdx": {
	id: "plugins/beeminder-word-count-plugin.mdx";
  slug: "plugins/beeminder-word-count-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/better-canvas-lock.mdx": {
	id: "plugins/better-canvas-lock.mdx";
  slug: "plugins/better-canvas-lock";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/better-comment-toggle.mdx": {
	id: "plugins/better-comment-toggle.mdx";
  slug: "plugins/better-comment-toggle";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/better-fn.mdx": {
	id: "plugins/better-fn.mdx";
  slug: "plugins/better-fn";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/better-inline-fields.mdx": {
	id: "plugins/better-inline-fields.mdx";
  slug: "plugins/better-inline-fields";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/better-mathjax.mdx": {
	id: "plugins/better-mathjax.mdx";
  slug: "plugins/better-mathjax";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/better-pdf-plugin.mdx": {
	id: "plugins/better-pdf-plugin.mdx";
  slug: "plugins/better-pdf-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/better-reading-mode.mdx": {
	id: "plugins/better-reading-mode.mdx";
  slug: "plugins/better-reading-mode";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/better-search-views.mdx": {
	id: "plugins/better-search-views.mdx";
  slug: "plugins/better-search-views";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/better-word-count.mdx": {
	id: "plugins/better-word-count.mdx";
  slug: "plugins/better-word-count";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/big-calendar.mdx": {
	id: "plugins/big-calendar.mdx";
  slug: "plugins/big-calendar";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/birthday-tracker.mdx": {
	id: "plugins/birthday-tracker.mdx";
  slug: "plugins/birthday-tracker";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/blindfold-obsidian.mdx": {
	id: "plugins/blindfold-obsidian.mdx";
  slug: "plugins/blindfold-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/block-reference-count.mdx": {
	id: "plugins/block-reference-count.mdx";
  slug: "plugins/block-reference-count";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/blockier.mdx": {
	id: "plugins/blockier.mdx";
  slug: "plugins/blockier";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/blockquote-levels.mdx": {
	id: "plugins/blockquote-levels.mdx";
  slug: "plugins/blockquote-levels";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/blur.mdx": {
	id: "plugins/blur.mdx";
  slug: "plugins/blur";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/bmo-chatbot.mdx": {
	id: "plugins/bmo-chatbot.mdx";
  slug: "plugins/bmo-chatbot";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/booksidian-plugin.mdx": {
	id: "plugins/booksidian-plugin.mdx";
  slug: "plugins/booksidian-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/boost-link-suggestions.mdx": {
	id: "plugins/boost-link-suggestions.mdx";
  slug: "plugins/boost-link-suggestions";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/bpmn-plugin.mdx": {
	id: "plugins/bpmn-plugin.mdx";
  slug: "plugins/bpmn-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/brain.mdx": {
	id: "plugins/brain.mdx";
  slug: "plugins/brain";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/braincache.mdx": {
	id: "plugins/braincache.mdx";
  slug: "plugins/braincache";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/brainframe.mdx": {
	id: "plugins/brainframe.mdx";
  slug: "plugins/brainframe";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/breadcrumbs.mdx": {
	id: "plugins/breadcrumbs.mdx";
  slug: "plugins/breadcrumbs";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/bulk-exporter.mdx": {
	id: "plugins/bulk-exporter.mdx";
  slug: "plugins/bulk-exporter";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/bulkopen-selected-links.mdx": {
	id: "plugins/bulkopen-selected-links.mdx";
  slug: "plugins/bulkopen-selected-links";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/buttons.mdx": {
	id: "plugins/buttons.mdx";
  slug: "plugins/buttons";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/calc-craft.mdx": {
	id: "plugins/calc-craft.mdx";
  slug: "plugins/calc-craft";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/calctex.mdx": {
	id: "plugins/calctex.mdx";
  slug: "plugins/calctex";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/calendar.mdx": {
	id: "plugins/calendar.mdx";
  slug: "plugins/calendar";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/callout-integrator.mdx": {
	id: "plugins/callout-integrator.mdx";
  slug: "plugins/callout-integrator";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/callout-manager.mdx": {
	id: "plugins/callout-manager.mdx";
  slug: "plugins/callout-manager";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/cannoli.mdx": {
	id: "plugins/cannoli.mdx";
  slug: "plugins/cannoli";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/canvas-css-class.mdx": {
	id: "plugins/canvas-css-class.mdx";
  slug: "plugins/canvas-css-class";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/canvas-filter.mdx": {
	id: "plugins/canvas-filter.mdx";
  slug: "plugins/canvas-filter";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/canvas-links.mdx": {
	id: "plugins/canvas-links.mdx";
  slug: "plugins/canvas-links";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/canvas-llm-extender.mdx": {
	id: "plugins/canvas-llm-extender.mdx";
  slug: "plugins/canvas-llm-extender";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/canvas-mindmap.mdx": {
	id: "plugins/canvas-mindmap.mdx";
  slug: "plugins/canvas-mindmap";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/canvas-presentation.mdx": {
	id: "plugins/canvas-presentation.mdx";
  slug: "plugins/canvas-presentation";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/canvas-randomnote.mdx": {
	id: "plugins/canvas-randomnote.mdx";
  slug: "plugins/canvas-randomnote";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/canvas-send-to-back.mdx": {
	id: "plugins/canvas-send-to-back.mdx";
  slug: "plugins/canvas-send-to-back";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/card-board.mdx": {
	id: "plugins/card-board.mdx";
  slug: "plugins/card-board";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/cardify.mdx": {
	id: "plugins/cardify.mdx";
  slug: "plugins/cardify";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/change-case.mdx": {
	id: "plugins/change-case.mdx";
  slug: "plugins/change-case";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/character-insertion.mdx": {
	id: "plugins/character-insertion.mdx";
  slug: "plugins/character-insertion";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/chat-cbt.mdx": {
	id: "plugins/chat-cbt.mdx";
  slug: "plugins/chat-cbt";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/chat-stream.mdx": {
	id: "plugins/chat-stream.mdx";
  slug: "plugins/chat-stream";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/chat-with-bard.mdx": {
	id: "plugins/chat-with-bard.mdx";
  slug: "plugins/chat-with-bard";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/chatgpt-definitions.mdx": {
	id: "plugins/chatgpt-definitions.mdx";
  slug: "plugins/chatgpt-definitions";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/chatgpt-md.mdx": {
	id: "plugins/chatgpt-md.mdx";
  slug: "plugins/chatgpt-md";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/checkbox-reorder.mdx": {
	id: "plugins/checkbox-reorder.mdx";
  slug: "plugins/checkbox-reorder";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/chem.mdx": {
	id: "plugins/chem.mdx";
  slug: "plugins/chem";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/chemical-structure-renderer.mdx": {
	id: "plugins/chemical-structure-renderer.mdx";
  slug: "plugins/chemical-structure-renderer";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/chess-study.mdx": {
	id: "plugins/chess-study.mdx";
  slug: "plugins/chess-study";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/chesser-obsidian.mdx": {
	id: "plugins/chesser-obsidian.mdx";
  slug: "plugins/chesser-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/chord-lyrics.mdx": {
	id: "plugins/chord-lyrics.mdx";
  slug: "plugins/chord-lyrics";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/chronology.mdx": {
	id: "plugins/chronology.mdx";
  slug: "plugins/chronology";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/close-similar-tabs.mdx": {
	id: "plugins/close-similar-tabs.mdx";
  slug: "plugins/close-similar-tabs";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/cloudinary.mdx": {
	id: "plugins/cloudinary.mdx";
  slug: "plugins/cloudinary";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/cloze.mdx": {
	id: "plugins/cloze.mdx";
  slug: "plugins/cloze";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/cm-chs-patch.mdx": {
	id: "plugins/cm-chs-patch.mdx";
  slug: "plugins/cm-chs-patch";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/cm-editor-syntax-highlight-obsidian.mdx": {
	id: "plugins/cm-editor-syntax-highlight-obsidian.mdx";
  slug: "plugins/cm-editor-syntax-highlight-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/cm-show-whitespace-obsidian.mdx": {
	id: "plugins/cm-show-whitespace-obsidian.mdx";
  slug: "plugins/cm-show-whitespace-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/cm-typewriter-scroll-obsidian.mdx": {
	id: "plugins/cm-typewriter-scroll-obsidian.mdx";
  slug: "plugins/cm-typewriter-scroll-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/cmdr.mdx": {
	id: "plugins/cmdr.mdx";
  slug: "plugins/cmdr";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/cmenu-plugin.mdx": {
	id: "plugins/cmenu-plugin.mdx";
  slug: "plugins/cmenu-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/code-block-copy.mdx": {
	id: "plugins/code-block-copy.mdx";
  slug: "plugins/code-block-copy";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/code-block-from-selection.mdx": {
	id: "plugins/code-block-from-selection.mdx";
  slug: "plugins/code-block-from-selection";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/code-block-plugin.mdx": {
	id: "plugins/code-block-plugin.mdx";
  slug: "plugins/code-block-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/code-emitter.mdx": {
	id: "plugins/code-emitter.mdx";
  slug: "plugins/code-emitter";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/code-files.mdx": {
	id: "plugins/code-files.mdx";
  slug: "plugins/code-files";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/code-styler.mdx": {
	id: "plugins/code-styler.mdx";
  slug: "plugins/code-styler";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/codeblock-customizer.mdx": {
	id: "plugins/codeblock-customizer.mdx";
  slug: "plugins/codeblock-customizer";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/codeblock-tabs.mdx": {
	id: "plugins/codeblock-tabs.mdx";
  slug: "plugins/codeblock-tabs";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/codeblock-template.mdx": {
	id: "plugins/codeblock-template.mdx";
  slug: "plugins/codeblock-template";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/codename.mdx": {
	id: "plugins/codename.mdx";
  slug: "plugins/codename";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/codestats.mdx": {
	id: "plugins/codestats.mdx";
  slug: "plugins/codestats";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/collapse-node.mdx": {
	id: "plugins/collapse-node.mdx";
  slug: "plugins/collapse-node";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/color-palette.mdx": {
	id: "plugins/color-palette.mdx";
  slug: "plugins/color-palette";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/colored-tags-wrangler.mdx": {
	id: "plugins/colored-tags-wrangler.mdx";
  slug: "plugins/colored-tags-wrangler";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/colored-tags.mdx": {
	id: "plugins/colored-tags.mdx";
  slug: "plugins/colored-tags";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/colored-text.mdx": {
	id: "plugins/colored-text.mdx";
  slug: "plugins/colored-text";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/colorful-note-borders.mdx": {
	id: "plugins/colorful-note-borders.mdx";
  slug: "plugins/colorful-note-borders";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/commando-command-repeater.mdx": {
	id: "plugins/commando-command-repeater.mdx";
  slug: "plugins/commando-command-repeater";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/companion.mdx": {
	id: "plugins/companion.mdx";
  slug: "plugins/companion";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/completed-area.mdx": {
	id: "plugins/completed-area.mdx";
  slug: "plugins/completed-area";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/completed-task-display.mdx": {
	id: "plugins/completed-task-display.mdx";
  slug: "plugins/completed-task-display";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/confluence-integration.mdx": {
	id: "plugins/confluence-integration.mdx";
  slug: "plugins/confluence-integration";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/confluence-to-obsidian.mdx": {
	id: "plugins/confluence-to-obsidian.mdx";
  slug: "plugins/confluence-to-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/consistent-attachments-and-links.mdx": {
	id: "plugins/consistent-attachments-and-links.mdx";
  slug: "plugins/consistent-attachments-and-links";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/console.mdx": {
	id: "plugins/console.mdx";
  slug: "plugins/console";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/content-linker.mdx": {
	id: "plugins/content-linker.mdx";
  slug: "plugins/content-linker";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/contentful-publisher.mdx": {
	id: "plugins/contentful-publisher.mdx";
  slug: "plugins/contentful-publisher";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/contextual-comments.mdx": {
	id: "plugins/contextual-comments.mdx";
  slug: "plugins/contextual-comments";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/control-characters.mdx": {
	id: "plugins/control-characters.mdx";
  slug: "plugins/control-characters";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/convert-url-to-iframe.mdx": {
	id: "plugins/convert-url-to-iframe.mdx";
  slug: "plugins/convert-url-to-iframe";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/cooklang-obsidian.mdx": {
	id: "plugins/cooklang-obsidian.mdx";
  slug: "plugins/cooklang-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/copilot-auto-completion.mdx": {
	id: "plugins/copilot-auto-completion.mdx";
  slug: "plugins/copilot-auto-completion";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/copilot.mdx": {
	id: "plugins/copilot.mdx";
  slug: "plugins/copilot";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/copy-as-html.mdx": {
	id: "plugins/copy-as-html.mdx";
  slug: "plugins/copy-as-html";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/copy-as-latex.mdx": {
	id: "plugins/copy-as-latex.mdx";
  slug: "plugins/copy-as-latex";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/copy-document-as-html.mdx": {
	id: "plugins/copy-document-as-html.mdx";
  slug: "plugins/copy-document-as-html";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/copy-inline-code.mdx": {
	id: "plugins/copy-inline-code.mdx";
  slug: "plugins/copy-inline-code";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/copy-metadata.mdx": {
	id: "plugins/copy-metadata.mdx";
  slug: "plugins/copy-metadata";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/copy-note.mdx": {
	id: "plugins/copy-note.mdx";
  slug: "plugins/copy-note";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/copy-publish-url.mdx": {
	id: "plugins/copy-publish-url.mdx";
  slug: "plugins/copy-publish-url";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/copy-url-in-preview.mdx": {
	id: "plugins/copy-url-in-preview.mdx";
  slug: "plugins/copy-url-in-preview";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/creases.mdx": {
	id: "plugins/creases.mdx";
  slug: "plugins/creases";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/create-note-in-folder.mdx": {
	id: "plugins/create-note-in-folder.mdx";
  slug: "plugins/create-note-in-folder";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/cron.mdx": {
	id: "plugins/cron.mdx";
  slug: "plugins/cron";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/crossbow.mdx": {
	id: "plugins/crossbow.mdx";
  slug: "plugins/crossbow";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/crumbs-obsidian.mdx": {
	id: "plugins/crumbs-obsidian.mdx";
  slug: "plugins/crumbs-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/cryptsidian.mdx": {
	id: "plugins/cryptsidian.mdx";
  slug: "plugins/cryptsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/css-editor.mdx": {
	id: "plugins/css-editor.mdx";
  slug: "plugins/css-editor";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/css-snippets.mdx": {
	id: "plugins/css-snippets.mdx";
  slug: "plugins/css-snippets";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/csv-codeblock.mdx": {
	id: "plugins/csv-codeblock.mdx";
  slug: "plugins/csv-codeblock";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/csv-obsidian.mdx": {
	id: "plugins/csv-obsidian.mdx";
  slug: "plugins/csv-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/custom-classes.mdx": {
	id: "plugins/custom-classes.mdx";
  slug: "plugins/custom-classes";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/custom-font-loader.mdx": {
	id: "plugins/custom-font-loader.mdx";
  slug: "plugins/custom-font-loader";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/custom-list-character.mdx": {
	id: "plugins/custom-list-character.mdx";
  slug: "plugins/custom-list-character";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/custom-note-width.mdx": {
	id: "plugins/custom-note-width.mdx";
  slug: "plugins/custom-note-width";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/custom-sort.mdx": {
	id: "plugins/custom-sort.mdx";
  slug: "plugins/custom-sort";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/custom-state-for-task-list.mdx": {
	id: "plugins/custom-state-for-task-list.mdx";
  slug: "plugins/custom-state-for-task-list";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/customizable-menu.mdx": {
	id: "plugins/customizable-menu.mdx";
  slug: "plugins/customizable-menu";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/customizable-page-header-buttons.mdx": {
	id: "plugins/customizable-page-header-buttons.mdx";
  slug: "plugins/customizable-page-header-buttons";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/customizable-sidebar.mdx": {
	id: "plugins/customizable-sidebar.mdx";
  slug: "plugins/customizable-sidebar";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/customjs.mdx": {
	id: "plugins/customjs.mdx";
  slug: "plugins/customjs";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/cycle-in-sidebar.mdx": {
	id: "plugins/cycle-in-sidebar.mdx";
  slug: "plugins/cycle-in-sidebar";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/cycle-through-panes.mdx": {
	id: "plugins/cycle-through-panes.mdx";
  slug: "plugins/cycle-through-panes";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/d2-obsidian.mdx": {
	id: "plugins/d2-obsidian.mdx";
  slug: "plugins/d2-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/daily-activity.mdx": {
	id: "plugins/daily-activity.mdx";
  slug: "plugins/daily-activity";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/daily-icalendar.mdx": {
	id: "plugins/daily-icalendar.mdx";
  slug: "plugins/daily-icalendar";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/daily-note-pinner.mdx": {
	id: "plugins/daily-note-pinner.mdx";
  slug: "plugins/daily-note-pinner";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/daily-notes-editor.mdx": {
	id: "plugins/daily-notes-editor.mdx";
  slug: "plugins/daily-notes-editor";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/dangerzone-writing-plugin.mdx": {
	id: "plugins/dangerzone-writing-plugin.mdx";
  slug: "plugins/dangerzone-writing-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/darlal-switcher-plus.mdx": {
	id: "plugins/darlal-switcher-plus.mdx";
  slug: "plugins/darlal-switcher-plus";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/data-entry.mdx": {
	id: "plugins/data-entry.mdx";
  slug: "plugins/data-entry";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/dataview.mdx": {
	id: "plugins/dataview.mdx";
  slug: "plugins/dataview";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/datetime-language-changer.mdx": {
	id: "plugins/datetime-language-changer.mdx";
  slug: "plugins/datetime-language-changer";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/day-planner-og.mdx": {
	id: "plugins/day-planner-og.mdx";
  slug: "plugins/day-planner-og";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/dbfolder.mdx": {
	id: "plugins/dbfolder.mdx";
  slug: "plugins/dbfolder";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/deepl.mdx": {
	id: "plugins/deepl.mdx";
  slug: "plugins/deepl";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/dendron-tree.mdx": {
	id: "plugins/dendron-tree.mdx";
  slug: "plugins/dendron-tree";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/desci.mdx": {
	id: "plugins/desci.mdx";
  slug: "plugins/desci";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/desk.mdx": {
	id: "plugins/desk.mdx";
  slug: "plugins/desk";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/dida-sync.mdx": {
	id: "plugins/dida-sync.mdx";
  slug: "plugins/dida-sync";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/digital-paper.mdx": {
	id: "plugins/digital-paper.mdx";
  slug: "plugins/digital-paper";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/digitalgarden.mdx": {
	id: "plugins/digitalgarden.mdx";
  slug: "plugins/digitalgarden";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/discord-message-formatter.mdx": {
	id: "plugins/discord-message-formatter.mdx";
  slug: "plugins/discord-message-formatter";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/discordian-plugin.mdx": {
	id: "plugins/discordian-plugin.mdx";
  slug: "plugins/discordian-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/disk-usage.mdx": {
	id: "plugins/disk-usage.mdx";
  slug: "plugins/disk-usage";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/dmn-eval.mdx": {
	id: "plugins/dmn-eval.mdx";
  slug: "plugins/dmn-eval";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/dmn-plugin.mdx": {
	id: "plugins/dmn-plugin.mdx";
  slug: "plugins/dmn-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/double-click-tab.mdx": {
	id: "plugins/double-click-tab.mdx";
  slug: "plugins/double-click-tab";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/double-colon-conceal.mdx": {
	id: "plugins/double-colon-conceal.mdx";
  slug: "plugins/double-colon-conceal";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/draw-harada-method.mdx": {
	id: "plugins/draw-harada-method.mdx";
  slug: "plugins/draw-harada-method";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/drawio-obsidian.mdx": {
	id: "plugins/drawio-obsidian.mdx";
  slug: "plugins/drawio-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/due-when.mdx": {
	id: "plugins/due-when.mdx";
  slug: "plugins/due-when";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/duplicate-line.mdx": {
	id: "plugins/duplicate-line.mdx";
  slug: "plugins/duplicate-line";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/dynamic-rtl.mdx": {
	id: "plugins/dynamic-rtl.mdx";
  slug: "plugins/dynamic-rtl";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/dynamic-timetable.mdx": {
	id: "plugins/dynamic-timetable.mdx";
  slug: "plugins/dynamic-timetable";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/easy-bake.mdx": {
	id: "plugins/easy-bake.mdx";
  slug: "plugins/easy-bake";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/easy-toggle-sidebars.mdx": {
	id: "plugins/easy-toggle-sidebars.mdx";
  slug: "plugins/easy-toggle-sidebars";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/easy-typing-obsidian.mdx": {
	id: "plugins/easy-typing-obsidian.mdx";
  slug: "plugins/easy-typing-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/edit-gemini.mdx": {
	id: "plugins/edit-gemini.mdx";
  slug: "plugins/edit-gemini";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/edit-history.mdx": {
	id: "plugins/edit-history.mdx";
  slug: "plugins/edit-history";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/edit-mdx.mdx": {
	id: "plugins/edit-mdx.mdx";
  slug: "plugins/edit-mdx";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/editing-toolbar.mdx": {
	id: "plugins/editing-toolbar.mdx";
  slug: "plugins/editing-toolbar";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/editor-autofocus.mdx": {
	id: "plugins/editor-autofocus.mdx";
  slug: "plugins/editor-autofocus";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/editor-commands-remap.mdx": {
	id: "plugins/editor-commands-remap.mdx";
  slug: "plugins/editor-commands-remap";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/editor-width-slider.mdx": {
	id: "plugins/editor-width-slider.mdx";
  slug: "plugins/editor-width-slider";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/eleven-labs.mdx": {
	id: "plugins/eleven-labs.mdx";
  slug: "plugins/eleven-labs";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/emacs-text-editor.mdx": {
	id: "plugins/emacs-text-editor.mdx";
  slug: "plugins/emacs-text-editor";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/email-block-plugin.mdx": {
	id: "plugins/email-block-plugin.mdx";
  slug: "plugins/email-block-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/embed-code-file.mdx": {
	id: "plugins/embed-code-file.mdx";
  slug: "plugins/embed-code-file";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/eml-reader.mdx": {
	id: "plugins/eml-reader.mdx";
  slug: "plugins/eml-reader";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/emo-uploader.mdx": {
	id: "plugins/emo-uploader.mdx";
  slug: "plugins/emo-uploader";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/emoji-magic.mdx": {
	id: "plugins/emoji-magic.mdx";
  slug: "plugins/emoji-magic";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/emoji-shortcodes.mdx": {
	id: "plugins/emoji-shortcodes.mdx";
  slug: "plugins/emoji-shortcodes";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/emoji-tags-titler.mdx": {
	id: "plugins/emoji-tags-titler.mdx";
  slug: "plugins/emoji-tags-titler";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/emoji-titler.mdx": {
	id: "plugins/emoji-titler.mdx";
  slug: "plugins/emoji-titler";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/enhance-youtube-links.mdx": {
	id: "plugins/enhance-youtube-links.mdx";
  slug: "plugins/enhance-youtube-links";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/enlightenment-obsidian.mdx": {
	id: "plugins/enlightenment-obsidian.mdx";
  slug: "plugins/enlightenment-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/epub-importer.mdx": {
	id: "plugins/epub-importer.mdx";
  slug: "plugins/epub-importer";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/excalibrain.mdx": {
	id: "plugins/excalibrain.mdx";
  slug: "plugins/excalibrain";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/excalidraw-cn.mdx": {
	id: "plugins/excalidraw-cn.mdx";
  slug: "plugins/excalidraw-cn";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/excel.mdx": {
	id: "plugins/excel.mdx";
  slug: "plugins/excel";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/execute-code.mdx": {
	id: "plugins/execute-code.mdx";
  slug: "plugins/execute-code";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/execute-python.mdx": {
	id: "plugins/execute-python.mdx";
  slug: "plugins/execute-python";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/exercises.mdx": {
	id: "plugins/exercises.mdx";
  slug: "plugins/exercises";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/expiration-date-tracker.mdx": {
	id: "plugins/expiration-date-tracker.mdx";
  slug: "plugins/expiration-date-tracker";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/extended-context-menu.mdx": {
	id: "plugins/extended-context-menu.mdx";
  slug: "plugins/extended-context-menu";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/extract-highlights-plugin.mdx": {
	id: "plugins/extract-highlights-plugin.mdx";
  slug: "plugins/extract-highlights-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/extract-url.mdx": {
	id: "plugins/extract-url.mdx";
  slug: "plugins/extract-url";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/fantasy-calendar.mdx": {
	id: "plugins/fantasy-calendar.mdx";
  slug: "plugins/fantasy-calendar";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/fantasy-content-generator.mdx": {
	id: "plugins/fantasy-content-generator.mdx";
  slug: "plugins/fantasy-content-generator";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/fantasy-name.mdx": {
	id: "plugins/fantasy-name.mdx";
  slug: "plugins/fantasy-name";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/favorite-note.mdx": {
	id: "plugins/favorite-note.mdx";
  slug: "plugins/favorite-note";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/feeds.mdx": {
	id: "plugins/feeds.mdx";
  slug: "plugins/feeds";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/file-chucker.mdx": {
	id: "plugins/file-chucker.mdx";
  slug: "plugins/file-chucker";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/file-cleaner-redux.mdx": {
	id: "plugins/file-cleaner-redux.mdx";
  slug: "plugins/file-cleaner-redux";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/file-diff.mdx": {
	id: "plugins/file-diff.mdx";
  slug: "plugins/file-diff";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/file-explorer-markdown-titles.mdx": {
	id: "plugins/file-explorer-markdown-titles.mdx";
  slug: "plugins/file-explorer-markdown-titles";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/file-explorer-note-count.mdx": {
	id: "plugins/file-explorer-note-count.mdx";
  slug: "plugins/file-explorer-note-count";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/file-explorer-plus.mdx": {
	id: "plugins/file-explorer-plus.mdx";
  slug: "plugins/file-explorer-plus";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/file-forgetting-curve-obsidian.mdx": {
	id: "plugins/file-forgetting-curve-obsidian.mdx";
  slug: "plugins/file-forgetting-curve-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/file-include.mdx": {
	id: "plugins/file-include.mdx";
  slug: "plugins/file-include";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/file-index.mdx": {
	id: "plugins/file-index.mdx";
  slug: "plugins/file-index";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/file-indicators.mdx": {
	id: "plugins/file-indicators.mdx";
  slug: "plugins/file-indicators";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/file-order.mdx": {
	id: "plugins/file-order.mdx";
  slug: "plugins/file-order";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/file-property-enhancer.mdx": {
	id: "plugins/file-property-enhancer.mdx";
  slug: "plugins/file-property-enhancer";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/file-publisher.mdx": {
	id: "plugins/file-publisher.mdx";
  slug: "plugins/file-publisher";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/file-tree-alternative.mdx": {
	id: "plugins/file-tree-alternative.mdx";
  slug: "plugins/file-tree-alternative";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/file-tree-generator.mdx": {
	id: "plugins/file-tree-generator.mdx";
  slug: "plugins/file-tree-generator";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/fill-in-the-blank.mdx": {
	id: "plugins/fill-in-the-blank.mdx";
  slug: "plugins/fill-in-the-blank";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/find-and-replace-in-selection.mdx": {
	id: "plugins/find-and-replace-in-selection.mdx";
  slug: "plugins/find-and-replace-in-selection";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/find-unlinked-files.mdx": {
	id: "plugins/find-unlinked-files.mdx";
  slug: "plugins/find-unlinked-files";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/findoc.mdx": {
	id: "plugins/findoc.mdx";
  slug: "plugins/findoc";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/finnish-spellcheck.mdx": {
	id: "plugins/finnish-spellcheck.mdx";
  slug: "plugins/finnish-spellcheck";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/flashcard-gen.mdx": {
	id: "plugins/flashcard-gen.mdx";
  slug: "plugins/flashcard-gen";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/flashcard-learning.mdx": {
	id: "plugins/flashcard-learning.mdx";
  slug: "plugins/flashcard-learning";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/flashcards-llm.mdx": {
	id: "plugins/flashcards-llm.mdx";
  slug: "plugins/flashcards-llm";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/flashcards-obsidian.mdx": {
	id: "plugins/flashcards-obsidian.mdx";
  slug: "plugins/flashcards-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/fleeting-notes-obsidian.mdx": {
	id: "plugins/fleeting-notes-obsidian.mdx";
  slug: "plugins/fleeting-notes-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/float-search.mdx": {
	id: "plugins/float-search.mdx";
  slug: "plugins/float-search";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/floating-highlights.mdx": {
	id: "plugins/floating-highlights.mdx";
  slug: "plugins/floating-highlights";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/floating-toc.mdx": {
	id: "plugins/floating-toc.mdx";
  slug: "plugins/floating-toc";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/floccus-bookmarks-to-markdown.mdx": {
	id: "plugins/floccus-bookmarks-to-markdown.mdx";
  slug: "plugins/floccus-bookmarks-to-markdown";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/flomo-importer.mdx": {
	id: "plugins/flomo-importer.mdx";
  slug: "plugins/flomo-importer";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/flowershow.mdx": {
	id: "plugins/flowershow.mdx";
  slug: "plugins/flowershow";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/focus-active-sentence.mdx": {
	id: "plugins/focus-active-sentence.mdx";
  slug: "plugins/focus-active-sentence";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/fold-anywhere.mdx": {
	id: "plugins/fold-anywhere.mdx";
  slug: "plugins/fold-anywhere";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/folder-note-core.mdx": {
	id: "plugins/folder-note-core.mdx";
  slug: "plugins/folder-note-core";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/folder-note-plugin.mdx": {
	id: "plugins/folder-note-plugin.mdx";
  slug: "plugins/folder-note-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/folder-notes.mdx": {
	id: "plugins/folder-notes.mdx";
  slug: "plugins/folder-notes";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/footlinks.mdx": {
	id: "plugins/footlinks.mdx";
  slug: "plugins/footlinks";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/format-hotkeys-obsidian.mdx": {
	id: "plugins/format-hotkeys-obsidian.mdx";
  slug: "plugins/format-hotkeys-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/formatto-format.mdx": {
	id: "plugins/formatto-format.mdx";
  slug: "plugins/formatto-format";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/fountain-editor.mdx": {
	id: "plugins/fountain-editor.mdx";
  slug: "plugins/fountain-editor";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/frontmatter-alias-display.mdx": {
	id: "plugins/frontmatter-alias-display.mdx";
  slug: "plugins/frontmatter-alias-display";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/frontmatter-generator.mdx": {
	id: "plugins/frontmatter-generator.mdx";
  slug: "plugins/frontmatter-generator";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/frontmatter-links.mdx": {
	id: "plugins/frontmatter-links.mdx";
  slug: "plugins/frontmatter-links";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/frontmatter-modified-date.mdx": {
	id: "plugins/frontmatter-modified-date.mdx";
  slug: "plugins/frontmatter-modified-date";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/fuzzy-chinese-pinyin.mdx": {
	id: "plugins/fuzzy-chinese-pinyin.mdx";
  slug: "plugins/fuzzy-chinese-pinyin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/game-search.mdx": {
	id: "plugins/game-search.mdx";
  slug: "plugins/game-search";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/garble-text.mdx": {
	id: "plugins/garble-text.mdx";
  slug: "plugins/garble-text";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/gemmy.mdx": {
	id: "plugins/gemmy.mdx";
  slug: "plugins/gemmy";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/gene-ai.mdx": {
	id: "plugins/gene-ai.mdx";
  slug: "plugins/gene-ai";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/generic-initiative-tracker.mdx": {
	id: "plugins/generic-initiative-tracker.mdx";
  slug: "plugins/generic-initiative-tracker";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/geocoding-properties.mdx": {
	id: "plugins/geocoding-properties.mdx";
  slug: "plugins/geocoding-properties";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/get-info-plugin.mdx": {
	id: "plugins/get-info-plugin.mdx";
  slug: "plugins/get-info-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/get-stock-information.mdx": {
	id: "plugins/get-stock-information.mdx";
  slug: "plugins/get-stock-information";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/ghcat-reminder.mdx": {
	id: "plugins/ghcat-reminder.mdx";
  slug: "plugins/ghcat-reminder";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/ghost-fade-focus.mdx": {
	id: "plugins/ghost-fade-focus.mdx";
  slug: "plugins/ghost-fade-focus";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/git-url.mdx": {
	id: "plugins/git-url.mdx";
  slug: "plugins/git-url";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/github-embeds.mdx": {
	id: "plugins/github-embeds.mdx";
  slug: "plugins/github-embeds";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/github-issue-augmentation.mdx": {
	id: "plugins/github-issue-augmentation.mdx";
  slug: "plugins/github-issue-augmentation";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/gladdis.mdx": {
	id: "plugins/gladdis.mdx";
  slug: "plugins/gladdis";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/global-markdown-encrypt.mdx": {
	id: "plugins/global-markdown-encrypt.mdx";
  slug: "plugins/global-markdown-encrypt";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/global-search-and-replace.mdx": {
	id: "plugins/global-search-and-replace.mdx";
  slug: "plugins/global-search-and-replace";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/gnome-terminal-loader.mdx": {
	id: "plugins/gnome-terminal-loader.mdx";
  slug: "plugins/gnome-terminal-loader";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/google-bard-assistant.mdx": {
	id: "plugins/google-bard-assistant.mdx";
  slug: "plugins/google-bard-assistant";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/google-calendar.mdx": {
	id: "plugins/google-calendar.mdx";
  slug: "plugins/google-calendar";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/google-keep-import.mdx": {
	id: "plugins/google-keep-import.mdx";
  slug: "plugins/google-keep-import";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/google-photos.mdx": {
	id: "plugins/google-photos.mdx";
  slug: "plugins/google-photos";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/gpg-crypt.mdx": {
	id: "plugins/gpg-crypt.mdx";
  slug: "plugins/gpg-crypt";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/gpt-assistant.mdx": {
	id: "plugins/gpt-assistant.mdx";
  slug: "plugins/gpt-assistant";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/gpt-liteinquirer.mdx": {
	id: "plugins/gpt-liteinquirer.mdx";
  slug: "plugins/gpt-liteinquirer";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/gpt3-notes.mdx": {
	id: "plugins/gpt3-notes.mdx";
  slug: "plugins/gpt3-notes";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/graph-analysis.mdx": {
	id: "plugins/graph-analysis.mdx";
  slug: "plugins/graph-analysis";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/graph-nested-tags.mdx": {
	id: "plugins/graph-nested-tags.mdx";
  slug: "plugins/graph-nested-tags";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/grappling-hook.mdx": {
	id: "plugins/grappling-hook.mdx";
  slug: "plugins/grappling-hook";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/gtd-no-next-step.mdx": {
	id: "plugins/gtd-no-next-step.mdx";
  slug: "plugins/gtd-no-next-step";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/guid-front-matter.mdx": {
	id: "plugins/guid-front-matter.mdx";
  slug: "plugins/guid-front-matter";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/habit-calendar.mdx": {
	id: "plugins/habit-calendar.mdx";
  slug: "plugins/habit-calendar";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/habit-tracker.mdx": {
	id: "plugins/habit-tracker.mdx";
  slug: "plugins/habit-tracker";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/hackerone.mdx": {
	id: "plugins/hackerone.mdx";
  slug: "plugins/hackerone";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/halo.mdx": {
	id: "plugins/halo.mdx";
  slug: "plugins/halo";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/hamsterbase.mdx": {
	id: "plugins/hamsterbase.mdx";
  slug: "plugins/hamsterbase";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/handwritten-notes.mdx": {
	id: "plugins/handwritten-notes.mdx";
  slug: "plugins/handwritten-notes";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/hard-breaks.mdx": {
	id: "plugins/hard-breaks.mdx";
  slug: "plugins/hard-breaks";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/harpoon.mdx": {
	id: "plugins/harpoon.mdx";
  slug: "plugins/harpoon";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/header-enhancer.mdx": {
	id: "plugins/header-enhancer.mdx";
  slug: "plugins/header-enhancer";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/heading-level-indent.mdx": {
	id: "plugins/heading-level-indent.mdx";
  slug: "plugins/heading-level-indent";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/heatmap-calendar.mdx": {
	id: "plugins/heatmap-calendar.mdx";
  slug: "plugins/heatmap-calendar";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/helpmate.mdx": {
	id: "plugins/helpmate.mdx";
  slug: "plugins/helpmate";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/hexo-auto-updater.mdx": {
	id: "plugins/hexo-auto-updater.mdx";
  slug: "plugins/hexo-auto-updater";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/heycalmdown-navigate-cursor-history.mdx": {
	id: "plugins/heycalmdown-navigate-cursor-history.mdx";
  slug: "plugins/heycalmdown-navigate-cursor-history";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/hidden-folder-obsidian.mdx": {
	id: "plugins/hidden-folder-obsidian.mdx";
  slug: "plugins/hidden-folder-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/hide-folders.mdx": {
	id: "plugins/hide-folders.mdx";
  slug: "plugins/hide-folders";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/highlightr-plugin.mdx": {
	id: "plugins/highlightr-plugin.mdx";
  slug: "plugins/highlightr-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/hints-plugin.mdx": {
	id: "plugins/hints-plugin.mdx";
  slug: "plugins/hints-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/hk-code-block.mdx": {
	id: "plugins/hk-code-block.mdx";
  slug: "plugins/hk-code-block";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/home-tab.mdx": {
	id: "plugins/home-tab.mdx";
  slug: "plugins/home-tab";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/homepage.mdx": {
	id: "plugins/homepage.mdx";
  slug: "plugins/homepage";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/homework-manager.mdx": {
	id: "plugins/homework-manager.mdx";
  slug: "plugins/homework-manager";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/hotkey-helper.mdx": {
	id: "plugins/hotkey-helper.mdx";
  slug: "plugins/hotkey-helper";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/hotkeysplus-obsidian.mdx": {
	id: "plugins/hotkeysplus-obsidian.mdx";
  slug: "plugins/hotkeysplus-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/hover-external-link.mdx": {
	id: "plugins/hover-external-link.mdx";
  slug: "plugins/hover-external-link";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/html-server.mdx": {
	id: "plugins/html-server.mdx";
  slug: "plugins/html-server";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/html-tabs.mdx": {
	id: "plugins/html-tabs.mdx";
  slug: "plugins/html-tabs";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/hugo-preview-obsidian.mdx": {
	id: "plugins/hugo-preview-obsidian.mdx";
  slug: "plugins/hugo-preview-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/hunchly.mdx": {
	id: "plugins/hunchly.mdx";
  slug: "plugins/hunchly";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/ibook.mdx": {
	id: "plugins/ibook.mdx";
  slug: "plugins/ibook";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/ical.mdx": {
	id: "plugins/ical.mdx";
  slug: "plugins/ical";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/iconoir-icons.mdx": {
	id: "plugins/iconoir-icons.mdx";
  slug: "plugins/iconoir-icons";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/ics.mdx": {
	id: "plugins/ics.mdx";
  slug: "plugins/ics";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/idorecall.mdx": {
	id: "plugins/idorecall.mdx";
  slug: "plugins/idorecall";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/image-captions.mdx": {
	id: "plugins/image-captions.mdx";
  slug: "plugins/image-captions";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/image-classify-paste.mdx": {
	id: "plugins/image-classify-paste.mdx";
  slug: "plugins/image-classify-paste";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/image-converter.mdx": {
	id: "plugins/image-converter.mdx";
  slug: "plugins/image-converter";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/image-ocr.mdx": {
	id: "plugins/image-ocr.mdx";
  slug: "plugins/image-ocr";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/image-upload-toolkit.mdx": {
	id: "plugins/image-upload-toolkit.mdx";
  slug: "plugins/image-upload-toolkit";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/image-window.mdx": {
	id: "plugins/image-window.mdx";
  slug: "plugins/image-window";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/image2latex.mdx": {
	id: "plugins/image2latex.mdx";
  slug: "plugins/image2latex";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/imdone-obsidian-plugin.mdx": {
	id: "plugins/imdone-obsidian-plugin.mdx";
  slug: "plugins/imdone-obsidian-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/import-foundry.mdx": {
	id: "plugins/import-foundry.mdx";
  slug: "plugins/import-foundry";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/improved-random-note.mdx": {
	id: "plugins/improved-random-note.mdx";
  slug: "plugins/improved-random-note";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/improved-vimcursor.mdx": {
	id: "plugins/improved-vimcursor.mdx";
  slug: "plugins/improved-vimcursor";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/inbox.mdx": {
	id: "plugins/inbox.mdx";
  slug: "plugins/inbox";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/incomplete-files.mdx": {
	id: "plugins/incomplete-files.mdx";
  slug: "plugins/incomplete-files";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/incremental-id.mdx": {
	id: "plugins/incremental-id.mdx";
  slug: "plugins/incremental-id";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/index-checker.mdx": {
	id: "plugins/index-checker.mdx";
  slug: "plugins/index-checker";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/influx.mdx": {
	id: "plugins/influx.mdx";
  slug: "plugins/influx";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/ini-obsidian.mdx": {
	id: "plugins/ini-obsidian.mdx";
  slug: "plugins/ini-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/initiative-tracker.mdx": {
	id: "plugins/initiative-tracker.mdx";
  slug: "plugins/initiative-tracker";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/inline-code-highlight.mdx": {
	id: "plugins/inline-code-highlight.mdx";
  slug: "plugins/inline-code-highlight";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/inline-encrypter.mdx": {
	id: "plugins/inline-encrypter.mdx";
  slug: "plugins/inline-encrypter";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/inline-math.mdx": {
	id: "plugins/inline-math.mdx";
  slug: "plugins/inline-math";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/insert-heading-link.mdx": {
	id: "plugins/insert-heading-link.mdx";
  slug: "plugins/insert-heading-link";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/insert-unsplash-image.mdx": {
	id: "plugins/insert-unsplash-image.mdx";
  slug: "plugins/insert-unsplash-image";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/invio.mdx": {
	id: "plugins/invio.mdx";
  slug: "plugins/invio";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/janitor.mdx": {
	id: "plugins/janitor.mdx";
  slug: "plugins/janitor";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/japanese-word-splitter.mdx": {
	id: "plugins/japanese-word-splitter.mdx";
  slug: "plugins/japanese-word-splitter";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/jelly-snippets.mdx": {
	id: "plugins/jelly-snippets.mdx";
  slug: "plugins/jelly-snippets";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/jira-cloud.mdx": {
	id: "plugins/jira-cloud.mdx";
  slug: "plugins/jira-cloud";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/jira-linker.mdx": {
	id: "plugins/jira-linker.mdx";
  slug: "plugins/jira-linker";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/journal-review.mdx": {
	id: "plugins/journal-review.mdx";
  slug: "plugins/journal-review";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/js-engine.mdx": {
	id: "plugins/js-engine.mdx";
  slug: "plugins/js-engine";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/json-table.mdx": {
	id: "plugins/json-table.mdx";
  slug: "plugins/json-table";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/juggl.mdx": {
	id: "plugins/juggl.mdx";
  slug: "plugins/juggl";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/just-share-please.mdx": {
	id: "plugins/just-share-please.mdx";
  slug: "plugins/just-share-please";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/ketcher.mdx": {
	id: "plugins/ketcher.mdx";
  slug: "plugins/ketcher";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/key-promoter.mdx": {
	id: "plugins/key-promoter.mdx";
  slug: "plugins/key-promoter";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/key-value-list.mdx": {
	id: "plugins/key-value-list.mdx";
  slug: "plugins/key-value-list";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/keyboard-analyzer.mdx": {
	id: "plugins/keyboard-analyzer.mdx";
  slug: "plugins/keyboard-analyzer";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/keyshots.mdx": {
	id: "plugins/keyshots.mdx";
  slug: "plugins/keyshots";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/keyword-highlighter.mdx": {
	id: "plugins/keyword-highlighter.mdx";
  slug: "plugins/keyword-highlighter";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/khoj.mdx": {
	id: "plugins/khoj.mdx";
  slug: "plugins/khoj";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/kill-and-yank.mdx": {
	id: "plugins/kill-and-yank.mdx";
  slug: "plugins/kill-and-yank";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/kindle-csv-converter.mdx": {
	id: "plugins/kindle-csv-converter.mdx";
  slug: "plugins/kindle-csv-converter";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/koncham-workspace.mdx": {
	id: "plugins/koncham-workspace.mdx";
  slug: "plugins/koncham-workspace";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/kr-book-info-plugin.mdx": {
	id: "plugins/kr-book-info-plugin.mdx";
  slug: "plugins/kr-book-info-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/language-translator.mdx": {
	id: "plugins/language-translator.mdx";
  slug: "plugins/language-translator";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/lapel.mdx": {
	id: "plugins/lapel.mdx";
  slug: "plugins/lapel";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/last-modified-timestamp-in-status-bar.mdx": {
	id: "plugins/last-modified-timestamp-in-status-bar.mdx";
  slug: "plugins/last-modified-timestamp-in-status-bar";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/latex-algorithms.mdx": {
	id: "plugins/latex-algorithms.mdx";
  slug: "plugins/latex-algorithms";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/latex-matrices.mdx": {
	id: "plugins/latex-matrices.mdx";
  slug: "plugins/latex-matrices";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/latex-to-unicode.mdx": {
	id: "plugins/latex-to-unicode.mdx";
  slug: "plugins/latex-to-unicode";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/laws-of-form.mdx": {
	id: "plugins/laws-of-form.mdx";
  slug: "plugins/laws-of-form";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/lds-scriptures-reference.mdx": {
	id: "plugins/lds-scriptures-reference.mdx";
  slug: "plugins/lds-scriptures-reference";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/leader-hotkeys-obsidian.mdx": {
	id: "plugins/leader-hotkeys-obsidian.mdx";
  slug: "plugins/leader-hotkeys-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/ledger-obsidian.mdx": {
	id: "plugins/ledger-obsidian.mdx";
  slug: "plugins/ledger-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/lilypond.mdx": {
	id: "plugins/lilypond.mdx";
  slug: "plugins/lilypond";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/ling-gloss.mdx": {
	id: "plugins/ling-gloss.mdx";
  slug: "plugins/ling-gloss";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/link-exploder.mdx": {
	id: "plugins/link-exploder.mdx";
  slug: "plugins/link-exploder";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/link-favicon.mdx": {
	id: "plugins/link-favicon.mdx";
  slug: "plugins/link-favicon";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/link-headers-directly.mdx": {
	id: "plugins/link-headers-directly.mdx";
  slug: "plugins/link-headers-directly";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/link-info-server.mdx": {
	id: "plugins/link-info-server.mdx";
  slug: "plugins/link-info-server";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/link-nodes-in-canvas.mdx": {
	id: "plugins/link-nodes-in-canvas.mdx";
  slug: "plugins/link-nodes-in-canvas";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/link-range.mdx": {
	id: "plugins/link-range.mdx";
  slug: "plugins/link-range";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/link-tree.mdx": {
	id: "plugins/link-tree.mdx";
  slug: "plugins/link-tree";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/link-with-alias.mdx": {
	id: "plugins/link-with-alias.mdx";
  slug: "plugins/link-with-alias";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/linkding-importer.mdx": {
	id: "plugins/linkding-importer.mdx";
  slug: "plugins/linkding-importer";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/linked-data-helper.mdx": {
	id: "plugins/linked-data-helper.mdx";
  slug: "plugins/linked-data-helper";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/linked-data-vocabularies.mdx": {
	id: "plugins/linked-data-vocabularies.mdx";
  slug: "plugins/linked-data-vocabularies";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/linkify.mdx": {
	id: "plugins/linkify.mdx";
  slug: "plugins/linkify";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/links.mdx": {
	id: "plugins/links.mdx";
  slug: "plugins/links";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/linkshelf.mdx": {
	id: "plugins/linkshelf.mdx";
  slug: "plugins/linkshelf";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/liquid-templates.mdx": {
	id: "plugins/liquid-templates.mdx";
  slug: "plugins/liquid-templates";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/list-style.mdx": {
	id: "plugins/list-style.mdx";
  slug: "plugins/list-style";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/literate-haskell.mdx": {
	id: "plugins/literate-haskell.mdx";
  slug: "plugins/literate-haskell";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/local-backup.mdx": {
	id: "plugins/local-backup.mdx";
  slug: "plugins/local-backup";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/local-graphql.mdx": {
	id: "plugins/local-graphql.mdx";
  slug: "plugins/local-graphql";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/local-quotes.mdx": {
	id: "plugins/local-quotes.mdx";
  slug: "plugins/local-quotes";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/logstravaganza.mdx": {
	id: "plugins/logstravaganza.mdx";
  slug: "plugins/logstravaganza";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/longform.mdx": {
	id: "plugins/longform.mdx";
  slug: "plugins/longform";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/loom.mdx": {
	id: "plugins/loom.mdx";
  slug: "plugins/loom";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/lovely-mindmap.mdx": {
	id: "plugins/lovely-mindmap.mdx";
  slug: "plugins/lovely-mindmap";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/lskypro-auto-upload.mdx": {
	id: "plugins/lskypro-auto-upload.mdx";
  slug: "plugins/lskypro-auto-upload";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/luhman.mdx": {
	id: "plugins/luhman.mdx";
  slug: "plugins/luhman";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/lumberjack-obsidian.mdx": {
	id: "plugins/lumberjack-obsidian.mdx";
  slug: "plugins/lumberjack-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/lunar-calendar.mdx": {
	id: "plugins/lunar-calendar.mdx";
  slug: "plugins/lunar-calendar";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/lyrics.mdx": {
	id: "plugins/lyrics.mdx";
  slug: "plugins/lyrics";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/macOS-keyboard-nav-obsidian.mdx": {
	id: "plugins/macOS-keyboard-nav-obsidian.mdx";
  slug: "plugins/macos-keyboard-nav-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/macro-plugin.mdx": {
	id: "plugins/macro-plugin.mdx";
  slug: "plugins/macro-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/magic-calendar.mdx": {
	id: "plugins/magic-calendar.mdx";
  slug: "plugins/magic-calendar";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/make-md.mdx": {
	id: "plugins/make-md.mdx";
  slug: "plugins/make-md";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/manictime.mdx": {
	id: "plugins/manictime.mdx";
  slug: "plugins/manictime";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/map-of-content.mdx": {
	id: "plugins/map-of-content.mdx";
  slug: "plugins/map-of-content";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/marginnote-companion.mdx": {
	id: "plugins/marginnote-companion.mdx";
  slug: "plugins/marginnote-companion";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/markdoc.mdx": {
	id: "plugins/markdoc.mdx";
  slug: "plugins/markdoc";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/markdown-attributes.mdx": {
	id: "plugins/markdown-attributes.mdx";
  slug: "plugins/markdown-attributes";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/markdown-blogger.mdx": {
	id: "plugins/markdown-blogger.mdx";
  slug: "plugins/markdown-blogger";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/markdown-chords.mdx": {
	id: "plugins/markdown-chords.mdx";
  slug: "plugins/markdown-chords";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/markdown-link-space-encoder.mdx": {
	id: "plugins/markdown-link-space-encoder.mdx";
  slug: "plugins/markdown-link-space-encoder";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/markdown-prettifier.mdx": {
	id: "plugins/markdown-prettifier.mdx";
  slug: "plugins/markdown-prettifier";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/markdown-shortcuts.mdx": {
	id: "plugins/markdown-shortcuts.mdx";
  slug: "plugins/markdown-shortcuts";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/markdown-sync-scroll.mdx": {
	id: "plugins/markdown-sync-scroll.mdx";
  slug: "plugins/markdown-sync-scroll";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/markdown-table-editor.mdx": {
	id: "plugins/markdown-table-editor.mdx";
  slug: "plugins/markdown-table-editor";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/markdown-to-slack-message.mdx": {
	id: "plugins/markdown-to-slack-message.mdx";
  slug: "plugins/markdown-to-slack-message";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/markdown-tree.mdx": {
	id: "plugins/markdown-tree.mdx";
  slug: "plugins/markdown-tree";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/marp-slides.mdx": {
	id: "plugins/marp-slides.mdx";
  slug: "plugins/marp-slides";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/marp.mdx": {
	id: "plugins/marp.mdx";
  slug: "plugins/marp";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/material-symbols.mdx": {
	id: "plugins/material-symbols.mdx";
  slug: "plugins/material-symbols";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/math-booster.mdx": {
	id: "plugins/math-booster.mdx";
  slug: "plugins/math-booster";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/mathlinks.mdx": {
	id: "plugins/mathlinks.mdx";
  slug: "plugins/mathlinks";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/mathlive.mdx": {
	id: "plugins/mathlive.mdx";
  slug: "plugins/mathlive";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/mathpad.mdx": {
	id: "plugins/mathpad.mdx";
  slug: "plugins/mathpad";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/matter.mdx": {
	id: "plugins/matter.mdx";
  slug: "plugins/matter";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/maximise-active-pane-obsidian.mdx": {
	id: "plugins/maximise-active-pane-obsidian.mdx";
  slug: "plugins/maximise-active-pane-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/md-image-caption.mdx": {
	id: "plugins/md-image-caption.mdx";
  slug: "plugins/md-image-caption";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/mdx-as-md-obsidian.mdx": {
	id: "plugins/mdx-as-md-obsidian.mdx";
  slug: "plugins/mdx-as-md-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/mdx.mdx": {
	id: "plugins/mdx.mdx";
  slug: "plugins/mdx";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/media-extended.mdx": {
	id: "plugins/media-extended.mdx";
  slug: "plugins/media-extended";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/media-sync.mdx": {
	id: "plugins/media-sync.mdx";
  slug: "plugins/media-sync";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/meld-build.mdx": {
	id: "plugins/meld-build.mdx";
  slug: "plugins/meld-build";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/meld-calc.mdx": {
	id: "plugins/meld-calc.mdx";
  slug: "plugins/meld-calc";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/meld-encrypt.mdx": {
	id: "plugins/meld-encrypt.mdx";
  slug: "plugins/meld-encrypt";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/memorization.mdx": {
	id: "plugins/memorization.mdx";
  slug: "plugins/memorization";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/memos-sync.mdx": {
	id: "plugins/memos-sync.mdx";
  slug: "plugins/memos-sync";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/merge-notes.mdx": {
	id: "plugins/merge-notes.mdx";
  slug: "plugins/merge-notes";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/mermaid-helper.mdx": {
	id: "plugins/mermaid-helper.mdx";
  slug: "plugins/mermaid-helper";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/mermaid-themes.mdx": {
	id: "plugins/mermaid-themes.mdx";
  slug: "plugins/mermaid-themes";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/mermaid-tools.mdx": {
	id: "plugins/mermaid-tools.mdx";
  slug: "plugins/mermaid-tools";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/metadata-extractor.mdx": {
	id: "plugins/metadata-extractor.mdx";
  slug: "plugins/metadata-extractor";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/metadata-menu.mdx": {
	id: "plugins/metadata-menu.mdx";
  slug: "plugins/metadata-menu";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/metaedit.mdx": {
	id: "plugins/metaedit.mdx";
  slug: "plugins/metaedit";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/metal-archives.mdx": {
	id: "plugins/metal-archives.mdx";
  slug: "plugins/metal-archives";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/micro-templates.mdx": {
	id: "plugins/micro-templates.mdx";
  slug: "plugins/micro-templates";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/microblog-publish-plugin.mdx": {
	id: "plugins/microblog-publish-plugin.mdx";
  slug: "plugins/microblog-publish-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/mini-toolbar.mdx": {
	id: "plugins/mini-toolbar.mdx";
  slug: "plugins/mini-toolbar";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/mini-vimrc.mdx": {
	id: "plugins/mini-vimrc.mdx";
  slug: "plugins/mini-vimrc";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/minio-uploader.mdx": {
	id: "plugins/minio-uploader.mdx";
  slug: "plugins/minio-uploader";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/minitabs.mdx": {
	id: "plugins/minitabs.mdx";
  slug: "plugins/minitabs";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/mixa.mdx": {
	id: "plugins/mixa.mdx";
  slug: "plugins/mixa";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/mlir-syntax-highlight.mdx": {
	id: "plugins/mlir-syntax-highlight.mdx";
  slug: "plugins/mlir-syntax-highlight";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/mochi-cards-exporter.mdx": {
	id: "plugins/mochi-cards-exporter.mdx";
  slug: "plugins/mochi-cards-exporter";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/mochi-cards-pro.mdx": {
	id: "plugins/mochi-cards-pro.mdx";
  slug: "plugins/mochi-cards-pro";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/modalforms.mdx": {
	id: "plugins/modalforms.mdx";
  slug: "plugins/modalforms";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/modules.mdx": {
	id: "plugins/modules.mdx";
  slug: "plugins/modules";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/mononote.mdx": {
	id: "plugins/mononote.mdx";
  slug: "plugins/mononote";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/mood-tracker.mdx": {
	id: "plugins/mood-tracker.mdx";
  slug: "plugins/mood-tracker";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/moon-server-publisher.mdx": {
	id: "plugins/moon-server-publisher.mdx";
  slug: "plugins/moon-server-publisher";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/mousewheel-image-zoom.mdx": {
	id: "plugins/mousewheel-image-zoom.mdx";
  slug: "plugins/mousewheel-image-zoom";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/movie-obsidian.mdx": {
	id: "plugins/movie-obsidian.mdx";
  slug: "plugins/movie-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/moviegrabber.mdx": {
	id: "plugins/moviegrabber.mdx";
  slug: "plugins/moviegrabber";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/mrj-add-codemirror-matchbrackets.mdx": {
	id: "plugins/mrj-add-codemirror-matchbrackets.mdx";
  slug: "plugins/mrj-add-codemirror-matchbrackets";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/mrj-backlinks-in-document.mdx": {
	id: "plugins/mrj-backlinks-in-document.mdx";
  slug: "plugins/mrj-backlinks-in-document";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/mrj-crosslink-between-notes.mdx": {
	id: "plugins/mrj-crosslink-between-notes.mdx";
  slug: "plugins/mrj-crosslink-between-notes";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/mrj-jump-to-link.mdx": {
	id: "plugins/mrj-jump-to-link.mdx";
  slug: "plugins/mrj-jump-to-link";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/mrj-text-expand.mdx": {
	id: "plugins/mrj-text-expand.mdx";
  slug: "plugins/mrj-text-expand";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/msg-handler.mdx": {
	id: "plugins/msg-handler.mdx";
  slug: "plugins/msg-handler";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/mtg-card-links.mdx": {
	id: "plugins/mtg-card-links.mdx";
  slug: "plugins/mtg-card-links";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/multi-column-markdown.mdx": {
	id: "plugins/multi-column-markdown.mdx";
  slug: "plugins/multi-column-markdown";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/multi-line-formatting.mdx": {
	id: "plugins/multi-line-formatting.mdx";
  slug: "plugins/multi-line-formatting";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/multi-properties.mdx": {
	id: "plugins/multi-properties.mdx";
  slug: "plugins/multi-properties";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/multi-tag.mdx": {
	id: "plugins/multi-tag.mdx";
  slug: "plugins/multi-tag";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/multiple-notes-outline.mdx": {
	id: "plugins/multiple-notes-outline.mdx";
  slug: "plugins/multiple-notes-outline";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/music-code-blocks.mdx": {
	id: "plugins/music-code-blocks.mdx";
  slug: "plugins/music-code-blocks";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/mx-bili-plugin.mdx": {
	id: "plugins/mx-bili-plugin.mdx";
  slug: "plugins/mx-bili-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/mysnippets-plugin.mdx": {
	id: "plugins/mysnippets-plugin.mdx";
  slug: "plugins/mysnippets-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/nai4obsidian.mdx": {
	id: "plugins/nai4obsidian.mdx";
  slug: "plugins/nai4obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/neo4j-graph-view.mdx": {
	id: "plugins/neo4j-graph-view.mdx";
  slug: "plugins/neo4j-graph-view";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/netwik.mdx": {
	id: "plugins/netwik.mdx";
  slug: "plugins/netwik";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/new-tab-default-page.mdx": {
	id: "plugins/new-tab-default-page.mdx";
  slug: "plugins/new-tab-default-page";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/next-link.mdx": {
	id: "plugins/next-link.mdx";
  slug: "plugins/next-link";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/nifty-links.mdx": {
	id: "plugins/nifty-links.mdx";
  slug: "plugins/nifty-links";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/ninja-cursor.mdx": {
	id: "plugins/ninja-cursor.mdx";
  slug: "plugins/ninja-cursor";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/nl-fast-image-cleaner.mdx": {
	id: "plugins/nl-fast-image-cleaner.mdx";
  slug: "plugins/nl-fast-image-cleaner";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/nl-syntax-highlighting.mdx": {
	id: "plugins/nl-syntax-highlighting.mdx";
  slug: "plugins/nl-syntax-highlighting";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/nldates-obsidian.mdx": {
	id: "plugins/nldates-obsidian.mdx";
  slug: "plugins/nldates-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/no-dupe-leaves.mdx": {
	id: "plugins/no-dupe-leaves.mdx";
  slug: "plugins/no-dupe-leaves";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/no-empty-windows.mdx": {
	id: "plugins/no-empty-windows.mdx";
  slug: "plugins/no-empty-windows";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/no-tabs.mdx": {
	id: "plugins/no-tabs.mdx";
  slug: "plugins/no-tabs";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/nostr-writer.mdx": {
	id: "plugins/nostr-writer.mdx";
  slug: "plugins/nostr-writer";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/note-aliases.mdx": {
	id: "plugins/note-aliases.mdx";
  slug: "plugins/note-aliases";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/note-archiver.mdx": {
	id: "plugins/note-archiver.mdx";
  slug: "plugins/note-archiver";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/note-batcher.mdx": {
	id: "plugins/note-batcher.mdx";
  slug: "plugins/note-batcher";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/note-folder-autorename.mdx": {
	id: "plugins/note-folder-autorename.mdx";
  slug: "plugins/note-folder-autorename";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/note-refactor-obsidian.mdx": {
	id: "plugins/note-refactor-obsidian.mdx";
  slug: "plugins/note-refactor-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/note-synchronizer.mdx": {
	id: "plugins/note-synchronizer.mdx";
  slug: "plugins/note-synchronizer";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/notes-dater.mdx": {
	id: "plugins/notes-dater.mdx";
  slug: "plugins/notes-dater";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/notes-merger.mdx": {
	id: "plugins/notes-merger.mdx";
  slug: "plugins/notes-merger";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/notes-sync-share.mdx": {
	id: "plugins/notes-sync-share.mdx";
  slug: "plugins/notes-sync-share";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/notetweet.mdx": {
	id: "plugins/notetweet.mdx";
  slug: "plugins/notetweet";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/nothing.mdx": {
	id: "plugins/nothing.mdx";
  slug: "plugins/nothing";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/notion-like-tables.mdx": {
	id: "plugins/notion-like-tables.mdx";
  slug: "plugins/notion-like-tables";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/novel-word-count.mdx": {
	id: "plugins/novel-word-count.mdx";
  slug: "plugins/novel-word-count";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/nuke-orphans.mdx": {
	id: "plugins/nuke-orphans.mdx";
  slug: "plugins/nuke-orphans";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/number-headings-obsidian.mdx": {
	id: "plugins/number-headings-obsidian.mdx";
  slug: "plugins/number-headings-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/numerals.mdx": {
	id: "plugins/numerals.mdx";
  slug: "plugins/numerals";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/o2.mdx": {
	id: "plugins/o2.mdx";
  slug: "plugins/o2";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obisidian-note-linker.mdx": {
	id: "plugins/obisidian-note-linker.mdx";
  slug: "plugins/obisidian-note-linker";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obligator.mdx": {
	id: "plugins/obligator.mdx";
  slug: "plugins/obligator";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/oblogger.mdx": {
	id: "plugins/oblogger.mdx";
  slug: "plugins/oblogger";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obs-text-wrapper.mdx": {
	id: "plugins/obs-text-wrapper.mdx";
  slug: "plugins/obs-text-wrapper";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-2hop-links-plugin.mdx": {
	id: "plugins/obsidian-2hop-links-plugin.mdx";
  slug: "plugins/obsidian-2hop-links-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-5e-statblocks.mdx": {
	id: "plugins/obsidian-5e-statblocks.mdx";
  slug: "plugins/obsidian-5e-statblocks";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-account-linker.mdx": {
	id: "plugins/obsidian-account-linker.mdx";
  slug: "plugins/obsidian-account-linker";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-achievements.mdx": {
	id: "plugins/obsidian-achievements.mdx";
  slug: "plugins/obsidian-achievements";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-activity-history.mdx": {
	id: "plugins/obsidian-activity-history.mdx";
  slug: "plugins/obsidian-activity-history";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-activity-logger.mdx": {
	id: "plugins/obsidian-activity-logger.mdx";
  slug: "plugins/obsidian-activity-logger";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-admonition.mdx": {
	id: "plugins/obsidian-admonition.mdx";
  slug: "plugins/obsidian-admonition";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-advanced-appearance.mdx": {
	id: "plugins/obsidian-advanced-appearance.mdx";
  slug: "plugins/obsidian-advanced-appearance";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-advanced-codeblock.mdx": {
	id: "plugins/obsidian-advanced-codeblock.mdx";
  slug: "plugins/obsidian-advanced-codeblock";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-advanced-new-file.mdx": {
	id: "plugins/obsidian-advanced-new-file.mdx";
  slug: "plugins/obsidian-advanced-new-file";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-advanced-new-folder.mdx": {
	id: "plugins/obsidian-advanced-new-folder.mdx";
  slug: "plugins/obsidian-advanced-new-folder";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-advanced-slides.mdx": {
	id: "plugins/obsidian-advanced-slides.mdx";
  slug: "plugins/obsidian-advanced-slides";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-advanced-uri.mdx": {
	id: "plugins/obsidian-advanced-uri.mdx";
  slug: "plugins/obsidian-advanced-uri";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-aggregator.mdx": {
	id: "plugins/obsidian-aggregator.mdx";
  slug: "plugins/obsidian-aggregator";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-agile-task-notes.mdx": {
	id: "plugins/obsidian-agile-task-notes.mdx";
  slug: "plugins/obsidian-agile-task-notes";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-alias-from-heading.mdx": {
	id: "plugins/obsidian-alias-from-heading.mdx";
  slug: "plugins/obsidian-alias-from-heading";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-amazingmarvin-plugin.mdx": {
	id: "plugins/obsidian-amazingmarvin-plugin.mdx";
  slug: "plugins/obsidian-amazingmarvin-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-ankibridge.mdx": {
	id: "plugins/obsidian-ankibridge.mdx";
  slug: "plugins/obsidian-ankibridge";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-annotator.mdx": {
	id: "plugins/obsidian-annotator.mdx";
  slug: "plugins/obsidian-annotator";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-another-quick-switcher.mdx": {
	id: "plugins/obsidian-another-quick-switcher.mdx";
  slug: "plugins/obsidian-another-quick-switcher";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-apple-reminders-plugin.mdx": {
	id: "plugins/obsidian-apple-reminders-plugin.mdx";
  slug: "plugins/obsidian-apple-reminders-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-apply-patterns.mdx": {
	id: "plugins/obsidian-apply-patterns.mdx";
  slug: "plugins/obsidian-apply-patterns";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-archivebox-plugin.mdx": {
	id: "plugins/obsidian-archivebox-plugin.mdx";
  slug: "plugins/obsidian-archivebox-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-argdown-plugin.mdx": {
	id: "plugins/obsidian-argdown-plugin.mdx";
  slug: "plugins/obsidian-argdown-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-asciidoc-blocks.mdx": {
	id: "plugins/obsidian-asciidoc-blocks.mdx";
  slug: "plugins/obsidian-asciidoc-blocks";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-asciimath.mdx": {
	id: "plugins/obsidian-asciimath.mdx";
  slug: "plugins/obsidian-asciimath";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-attachment-name-formatting.mdx": {
	id: "plugins/obsidian-attachment-name-formatting.mdx";
  slug: "plugins/obsidian-attachment-name-formatting";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-attendance.mdx": {
	id: "plugins/obsidian-attendance.mdx";
  slug: "plugins/obsidian-attendance";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-audio-notes.mdx": {
	id: "plugins/obsidian-audio-notes.mdx";
  slug: "plugins/obsidian-audio-notes";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-audio-player.mdx": {
	id: "plugins/obsidian-audio-player.mdx";
  slug: "plugins/obsidian-audio-player";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-audio-speed-plugin.mdx": {
	id: "plugins/obsidian-audio-speed-plugin.mdx";
  slug: "plugins/obsidian-audio-speed-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-auto-hide.mdx": {
	id: "plugins/obsidian-auto-hide.mdx";
  slug: "plugins/obsidian-auto-hide";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-auto-link-title.mdx": {
	id: "plugins/obsidian-auto-link-title.mdx";
  slug: "plugins/obsidian-auto-link-title";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-auto-pair-chinese-symbol.mdx": {
	id: "plugins/obsidian-auto-pair-chinese-symbol.mdx";
  slug: "plugins/obsidian-auto-pair-chinese-symbol";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-auto-split.mdx": {
	id: "plugins/obsidian-auto-split.mdx";
  slug: "plugins/obsidian-auto-split";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-autocomplete-plugin.mdx": {
	id: "plugins/obsidian-autocomplete-plugin.mdx";
  slug: "plugins/obsidian-autocomplete-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-autoscroll.mdx": {
	id: "plugins/obsidian-autoscroll.mdx";
  slug: "plugins/obsidian-autoscroll";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-awesome-flashcard.mdx": {
	id: "plugins/obsidian-awesome-flashcard.mdx";
  slug: "plugins/obsidian-awesome-flashcard";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-badge.mdx": {
	id: "plugins/obsidian-badge.mdx";
  slug: "plugins/obsidian-badge";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-banners.mdx": {
	id: "plugins/obsidian-banners.mdx";
  slug: "plugins/obsidian-banners";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-basetag.mdx": {
	id: "plugins/obsidian-basetag.mdx";
  slug: "plugins/obsidian-basetag";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-bbcode.mdx": {
	id: "plugins/obsidian-bbcode.mdx";
  slug: "plugins/obsidian-bbcode";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-bellboy.mdx": {
	id: "plugins/obsidian-bellboy.mdx";
  slug: "plugins/obsidian-bellboy";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-better-codeblock.mdx": {
	id: "plugins/obsidian-better-codeblock.mdx";
  slug: "plugins/obsidian-better-codeblock";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-better-command-palette.mdx": {
	id: "plugins/obsidian-better-command-palette.mdx";
  slug: "plugins/obsidian-better-command-palette";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-better-internal-link-inserter.mdx": {
	id: "plugins/obsidian-better-internal-link-inserter.mdx";
  slug: "plugins/obsidian-better-internal-link-inserter";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-bible-linker.mdx": {
	id: "plugins/obsidian-bible-linker.mdx";
  slug: "plugins/obsidian-bible-linker";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-bible-reference.mdx": {
	id: "plugins/obsidian-bible-reference.mdx";
  slug: "plugins/obsidian-bible-reference";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-binary-file-manager-plugin.mdx": {
	id: "plugins/obsidian-binary-file-manager-plugin.mdx";
  slug: "plugins/obsidian-binary-file-manager-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-book-search-plugin.mdx": {
	id: "plugins/obsidian-book-search-plugin.mdx";
  slug: "plugins/obsidian-book-search-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-budget-wysiwyg.mdx": {
	id: "plugins/obsidian-budget-wysiwyg.mdx";
  slug: "plugins/obsidian-budget-wysiwyg";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-bulk-rename-plugin.mdx": {
	id: "plugins/obsidian-bulk-rename-plugin.mdx";
  slug: "plugins/obsidian-bulk-rename-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-buttondown-plugin.mdx": {
	id: "plugins/obsidian-buttondown-plugin.mdx";
  slug: "plugins/obsidian-buttondown-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-calibre-plugin.mdx": {
	id: "plugins/obsidian-calibre-plugin.mdx";
  slug: "plugins/obsidian-calibre-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-california-coast-settings.mdx": {
	id: "plugins/obsidian-california-coast-settings.mdx";
  slug: "plugins/obsidian-california-coast-settings";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-camera.mdx": {
	id: "plugins/obsidian-camera.mdx";
  slug: "plugins/obsidian-camera";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-canvas-conversation.mdx": {
	id: "plugins/obsidian-canvas-conversation.mdx";
  slug: "plugins/obsidian-canvas-conversation";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-card-view-mode.mdx": {
	id: "plugins/obsidian-card-view-mode.mdx";
  slug: "plugins/obsidian-card-view-mode";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-card-view-switcher-plugin.mdx": {
	id: "plugins/obsidian-card-view-switcher-plugin.mdx";
  slug: "plugins/obsidian-card-view-switcher-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-carry-forward.mdx": {
	id: "plugins/obsidian-carry-forward.mdx";
  slug: "plugins/obsidian-carry-forward";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-charts.mdx": {
	id: "plugins/obsidian-charts.mdx";
  slug: "plugins/obsidian-charts";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-chartsview-plugin.mdx": {
	id: "plugins/obsidian-chartsview-plugin.mdx";
  slug: "plugins/obsidian-chartsview-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-chat-view.mdx": {
	id: "plugins/obsidian-chat-view.mdx";
  slug: "plugins/obsidian-chat-view";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-checkbox3states-plugin.mdx": {
	id: "plugins/obsidian-checkbox3states-plugin.mdx";
  slug: "plugins/obsidian-checkbox3states-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-checklist-plugin.mdx": {
	id: "plugins/obsidian-checklist-plugin.mdx";
  slug: "plugins/obsidian-checklist-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-checklist-reset.mdx": {
	id: "plugins/obsidian-checklist-reset.mdx";
  slug: "plugins/obsidian-checklist-reset";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-chess.mdx": {
	id: "plugins/obsidian-chess.mdx";
  slug: "plugins/obsidian-chess";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-chessboard.mdx": {
	id: "plugins/obsidian-chessboard.mdx";
  slug: "plugins/obsidian-chessboard";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-chevereto-image-uploader.mdx": {
	id: "plugins/obsidian-chevereto-image-uploader.mdx";
  slug: "plugins/obsidian-chevereto-image-uploader";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-chorded-hotkeys.mdx": {
	id: "plugins/obsidian-chorded-hotkeys.mdx";
  slug: "plugins/obsidian-chorded-hotkeys";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-circuitjs.mdx": {
	id: "plugins/obsidian-circuitjs.mdx";
  slug: "plugins/obsidian-circuitjs";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-citation-plugin.mdx": {
	id: "plugins/obsidian-citation-plugin.mdx";
  slug: "plugins/obsidian-citation-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-clipper.mdx": {
	id: "plugins/obsidian-clipper.mdx";
  slug: "plugins/obsidian-clipper";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-cloudinary-uploader.mdx": {
	id: "plugins/obsidian-cloudinary-uploader.mdx";
  slug: "plugins/obsidian-cloudinary-uploader";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-code-block-enhancer.mdx": {
	id: "plugins/obsidian-code-block-enhancer.mdx";
  slug: "plugins/obsidian-code-block-enhancer";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-code-copy.mdx": {
	id: "plugins/obsidian-code-copy.mdx";
  slug: "plugins/obsidian-code-copy";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-code-preview.mdx": {
	id: "plugins/obsidian-code-preview.mdx";
  slug: "plugins/obsidian-code-preview";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-codeblock-labels.mdx": {
	id: "plugins/obsidian-codeblock-labels.mdx";
  slug: "plugins/obsidian-codeblock-labels";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-codemirror-options.mdx": {
	id: "plugins/obsidian-codemirror-options.mdx";
  slug: "plugins/obsidian-codemirror-options";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-collapse-all-plugin.mdx": {
	id: "plugins/obsidian-collapse-all-plugin.mdx";
  slug: "plugins/obsidian-collapse-all-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-colorful-tag.mdx": {
	id: "plugins/obsidian-colorful-tag.mdx";
  slug: "plugins/obsidian-colorful-tag";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-columns.mdx": {
	id: "plugins/obsidian-columns.mdx";
  slug: "plugins/obsidian-columns";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-command-alias-plugin.mdx": {
	id: "plugins/obsidian-command-alias-plugin.mdx";
  slug: "plugins/obsidian-command-alias-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-command-palette-minus-plugin.mdx": {
	id: "plugins/obsidian-command-palette-minus-plugin.mdx";
  slug: "plugins/obsidian-command-palette-minus-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-comments.mdx": {
	id: "plugins/obsidian-comments.mdx";
  slug: "plugins/obsidian-comments";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-commits.mdx": {
	id: "plugins/obsidian-commits.mdx";
  slug: "plugins/obsidian-commits";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-completr.mdx": {
	id: "plugins/obsidian-completr.mdx";
  slug: "plugins/obsidian-completr";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-contacts.mdx": {
	id: "plugins/obsidian-contacts.mdx";
  slug: "plugins/obsidian-contacts";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-contextual-typography.mdx": {
	id: "plugins/obsidian-contextual-typography.mdx";
  slug: "plugins/obsidian-contextual-typography";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-copy-block-link.mdx": {
	id: "plugins/obsidian-copy-block-link.mdx";
  slug: "plugins/obsidian-copy-block-link";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-copy-search-url.mdx": {
	id: "plugins/obsidian-copy-search-url.mdx";
  slug: "plugins/obsidian-copy-search-url";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-core-search-assistant-plugin.mdx": {
	id: "plugins/obsidian-core-search-assistant-plugin.mdx";
  slug: "plugins/obsidian-core-search-assistant-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-crypto-lookup.mdx": {
	id: "plugins/obsidian-crypto-lookup.mdx";
  slug: "plugins/obsidian-crypto-lookup";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-csv-table.mdx": {
	id: "plugins/obsidian-csv-table.mdx";
  slug: "plugins/obsidian-csv-table";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-cursor-location-plugin.mdx": {
	id: "plugins/obsidian-cursor-location-plugin.mdx";
  slug: "plugins/obsidian-cursor-location-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-custom-attachment-location.mdx": {
	id: "plugins/obsidian-custom-attachment-location.mdx";
  slug: "plugins/obsidian-custom-attachment-location";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-custom-file-extensions-plugin.mdx": {
	id: "plugins/obsidian-custom-file-extensions-plugin.mdx";
  slug: "plugins/obsidian-custom-file-extensions-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-custom-frames.mdx": {
	id: "plugins/obsidian-custom-frames.mdx";
  slug: "plugins/obsidian-custom-frames";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-daf-yomi.mdx": {
	id: "plugins/obsidian-daf-yomi.mdx";
  slug: "plugins/obsidian-daf-yomi";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-daily-named-folder.mdx": {
	id: "plugins/obsidian-daily-named-folder.mdx";
  slug: "plugins/obsidian-daily-named-folder";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-daily-note-outline.mdx": {
	id: "plugins/obsidian-daily-note-outline.mdx";
  slug: "plugins/obsidian-daily-note-outline";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-daily-notes-new-tab.mdx": {
	id: "plugins/obsidian-daily-notes-new-tab.mdx";
  slug: "plugins/obsidian-daily-notes-new-tab";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-daily-notes-opener.mdx": {
	id: "plugins/obsidian-daily-notes-opener.mdx";
  slug: "plugins/obsidian-daily-notes-opener";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-daily-notes-viewer.mdx": {
	id: "plugins/obsidian-daily-notes-viewer.mdx";
  slug: "plugins/obsidian-daily-notes-viewer";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-daily-stats.mdx": {
	id: "plugins/obsidian-daily-stats.mdx";
  slug: "plugins/obsidian-daily-stats";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-dangling-links.mdx": {
	id: "plugins/obsidian-dangling-links.mdx";
  slug: "plugins/obsidian-dangling-links";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-dashing-cursor.mdx": {
	id: "plugins/obsidian-dashing-cursor.mdx";
  slug: "plugins/obsidian-dashing-cursor";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-dataset-aid.mdx": {
	id: "plugins/obsidian-dataset-aid.mdx";
  slug: "plugins/obsidian-dataset-aid";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-day-and-night.mdx": {
	id: "plugins/obsidian-day-and-night.mdx";
  slug: "plugins/obsidian-day-and-night";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-day-planner.mdx": {
	id: "plugins/obsidian-day-planner.mdx";
  slug: "plugins/obsidian-day-planner";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-desmos.mdx": {
	id: "plugins/obsidian-desmos.mdx";
  slug: "plugins/obsidian-desmos";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-diagrams-net.mdx": {
	id: "plugins/obsidian-diagrams-net.mdx";
  slug: "plugins/obsidian-diagrams-net";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-dialogue-plugin.mdx": {
	id: "plugins/obsidian-dialogue-plugin.mdx";
  slug: "plugins/obsidian-dialogue-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-dice-roller.mdx": {
	id: "plugins/obsidian-dice-roller.mdx";
  slug: "plugins/obsidian-dice-roller";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-dictionary-plugin.mdx": {
	id: "plugins/obsidian-dictionary-plugin.mdx";
  slug: "plugins/obsidian-dictionary-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-dirtreeist.mdx": {
	id: "plugins/obsidian-dirtreeist.mdx";
  slug: "plugins/obsidian-dirtreeist";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-discordrpc.mdx": {
	id: "plugins/obsidian-discordrpc.mdx";
  slug: "plugins/obsidian-discordrpc";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-divide-and-conquer.mdx": {
	id: "plugins/obsidian-divide-and-conquer.mdx";
  slug: "plugins/obsidian-divide-and-conquer";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-douban-plugin.mdx": {
	id: "plugins/obsidian-douban-plugin.mdx";
  slug: "plugins/obsidian-douban-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-doubleshift.mdx": {
	id: "plugins/obsidian-doubleshift.mdx";
  slug: "plugins/obsidian-doubleshift";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-drag-n-drop-plugin.mdx": {
	id: "plugins/obsidian-drag-n-drop-plugin.mdx";
  slug: "plugins/obsidian-drag-n-drop-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-dropbox-backups.mdx": {
	id: "plugins/obsidian-dropbox-backups.mdx";
  slug: "plugins/obsidian-dropbox-backups";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-dynamic-background.mdx": {
	id: "plugins/obsidian-dynamic-background.mdx";
  slug: "plugins/obsidian-dynamic-background";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-dynamic-embed.mdx": {
	id: "plugins/obsidian-dynamic-embed.mdx";
  slug: "plugins/obsidian-dynamic-embed";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-dynamic-highlights.mdx": {
	id: "plugins/obsidian-dynamic-highlights.mdx";
  slug: "plugins/obsidian-dynamic-highlights";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-dynamic-toc.mdx": {
	id: "plugins/obsidian-dynamic-toc.mdx";
  slug: "plugins/obsidian-dynamic-toc";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-dynbedded.mdx": {
	id: "plugins/obsidian-dynbedded.mdx";
  slug: "plugins/obsidian-dynbedded";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-echarts.mdx": {
	id: "plugins/obsidian-echarts.mdx";
  slug: "plugins/obsidian-echarts";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-editor-shortcuts.mdx": {
	id: "plugins/obsidian-editor-shortcuts.mdx";
  slug: "plugins/obsidian-editor-shortcuts";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-electron-window-tweaker.mdx": {
	id: "plugins/obsidian-electron-window-tweaker.mdx";
  slug: "plugins/obsidian-electron-window-tweaker";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-embedded-code-title.mdx": {
	id: "plugins/obsidian-embedded-code-title.mdx";
  slug: "plugins/obsidian-embedded-code-title";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-embedded-note-paths.mdx": {
	id: "plugins/obsidian-embedded-note-paths.mdx";
  slug: "plugins/obsidian-embedded-note-paths";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-embedded-note-titles.mdx": {
	id: "plugins/obsidian-embedded-note-titles.mdx";
  slug: "plugins/obsidian-embedded-note-titles";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-emoji-toolbar.mdx": {
	id: "plugins/obsidian-emoji-toolbar.mdx";
  slug: "plugins/obsidian-emoji-toolbar";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-emotion-picker.mdx": {
	id: "plugins/obsidian-emotion-picker.mdx";
  slug: "plugins/obsidian-emotion-picker";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-enhancing-export.mdx": {
	id: "plugins/obsidian-enhancing-export.mdx";
  slug: "plugins/obsidian-enhancing-export";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-enhancing-mindmap.mdx": {
	id: "plugins/obsidian-enhancing-mindmap.mdx";
  slug: "plugins/obsidian-enhancing-mindmap";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-epub-plugin.mdx": {
	id: "plugins/obsidian-epub-plugin.mdx";
  slug: "plugins/obsidian-epub-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-etherpad-plugin.mdx": {
	id: "plugins/obsidian-etherpad-plugin.mdx";
  slug: "plugins/obsidian-etherpad-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-excalidraw-plugin.mdx": {
	id: "plugins/obsidian-excalidraw-plugin.mdx";
  slug: "plugins/obsidian-excalidraw-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-excel-to-markdown-table.mdx": {
	id: "plugins/obsidian-excel-to-markdown-table.mdx";
  slug: "plugins/obsidian-excel-to-markdown-table";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-expand-bullet.mdx": {
	id: "plugins/obsidian-expand-bullet.mdx";
  slug: "plugins/obsidian-expand-bullet";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-export-image.mdx": {
	id: "plugins/obsidian-export-image.mdx";
  slug: "plugins/obsidian-export-image";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-export-to-tex.mdx": {
	id: "plugins/obsidian-export-to-tex.mdx";
  slug: "plugins/obsidian-export-to-tex";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-extlnkhelper-plugin.mdx": {
	id: "plugins/obsidian-extlnkhelper-plugin.mdx";
  slug: "plugins/obsidian-extlnkhelper-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-extra-md-commands.mdx": {
	id: "plugins/obsidian-extra-md-commands.mdx";
  slug: "plugins/obsidian-extra-md-commands";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-extract-pdf-annotations.mdx": {
	id: "plugins/obsidian-extract-pdf-annotations.mdx";
  slug: "plugins/obsidian-extract-pdf-annotations";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-extract-pdf-highlights.mdx": {
	id: "plugins/obsidian-extract-pdf-highlights.mdx";
  slug: "plugins/obsidian-extract-pdf-highlights";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-file-cleaner.mdx": {
	id: "plugins/obsidian-file-cleaner.mdx";
  slug: "plugins/obsidian-file-cleaner";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-file-color.mdx": {
	id: "plugins/obsidian-file-color.mdx";
  slug: "plugins/obsidian-file-color";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-file-cooker.mdx": {
	id: "plugins/obsidian-file-cooker.mdx";
  slug: "plugins/obsidian-file-cooker";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-file-info-plugin.mdx": {
	id: "plugins/obsidian-file-info-plugin.mdx";
  slug: "plugins/obsidian-file-info-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-file-link.mdx": {
	id: "plugins/obsidian-file-link.mdx";
  slug: "plugins/obsidian-file-link";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-file-path-to-uri.mdx": {
	id: "plugins/obsidian-file-path-to-uri.mdx";
  slug: "plugins/obsidian-file-path-to-uri";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-filename-emoji-remover.mdx": {
	id: "plugins/obsidian-filename-emoji-remover.mdx";
  slug: "plugins/obsidian-filename-emoji-remover";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-filename-heading-sync.mdx": {
	id: "plugins/obsidian-filename-heading-sync.mdx";
  slug: "plugins/obsidian-filename-heading-sync";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-find-and-replace-in-selection.mdx": {
	id: "plugins/obsidian-find-and-replace-in-selection.mdx";
  slug: "plugins/obsidian-find-and-replace-in-selection";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-flexible-pomo.mdx": {
	id: "plugins/obsidian-flexible-pomo.mdx";
  slug: "plugins/obsidian-flexible-pomo";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-focus-mode.mdx": {
	id: "plugins/obsidian-focus-mode.mdx";
  slug: "plugins/obsidian-focus-mode";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-focus-plugin.mdx": {
	id: "plugins/obsidian-focus-plugin.mdx";
  slug: "plugins/obsidian-focus-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-folder-focus-mode.mdx": {
	id: "plugins/obsidian-folder-focus-mode.mdx";
  slug: "plugins/obsidian-folder-focus-mode";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-folder-index.mdx": {
	id: "plugins/obsidian-folder-index.mdx";
  slug: "plugins/obsidian-folder-index";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-footnote-indicator.mdx": {
	id: "plugins/obsidian-footnote-indicator.mdx";
  slug: "plugins/obsidian-footnote-indicator";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-footnotes.mdx": {
	id: "plugins/obsidian-footnotes.mdx";
  slug: "plugins/obsidian-footnotes";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-format-code.mdx": {
	id: "plugins/obsidian-format-code.mdx";
  slug: "plugins/obsidian-format-code";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-fountain.mdx": {
	id: "plugins/obsidian-fountain.mdx";
  slug: "plugins/obsidian-fountain";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-front-matter-title-plugin.mdx": {
	id: "plugins/obsidian-front-matter-title-plugin.mdx";
  slug: "plugins/obsidian-front-matter-title-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-frontmatter-tag-suggest.mdx": {
	id: "plugins/obsidian-frontmatter-tag-suggest.mdx";
  slug: "plugins/obsidian-frontmatter-tag-suggest";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-full-calendar.mdx": {
	id: "plugins/obsidian-full-calendar.mdx";
  slug: "plugins/obsidian-full-calendar";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-fullscreen-plugin.mdx": {
	id: "plugins/obsidian-fullscreen-plugin.mdx";
  slug: "plugins/obsidian-fullscreen-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-functionplot.mdx": {
	id: "plugins/obsidian-functionplot.mdx";
  slug: "plugins/obsidian-functionplot";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-furigana.mdx": {
	id: "plugins/obsidian-furigana.mdx";
  slug: "plugins/obsidian-furigana";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-fuzzytag.mdx": {
	id: "plugins/obsidian-fuzzytag.mdx";
  slug: "plugins/obsidian-fuzzytag";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-gallery.mdx": {
	id: "plugins/obsidian-gallery.mdx";
  slug: "plugins/obsidian-gallery";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-ghost-publish.mdx": {
	id: "plugins/obsidian-ghost-publish.mdx";
  slug: "plugins/obsidian-ghost-publish";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-gist.mdx": {
	id: "plugins/obsidian-gist.mdx";
  slug: "plugins/obsidian-gist";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-git.mdx": {
	id: "plugins/obsidian-git.mdx";
  slug: "plugins/obsidian-git";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-gitlab-issues.mdx": {
	id: "plugins/obsidian-gitlab-issues.mdx";
  slug: "plugins/obsidian-gitlab-issues";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-global-hotkeys.mdx": {
	id: "plugins/obsidian-global-hotkeys.mdx";
  slug: "plugins/obsidian-global-hotkeys";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-go-to-line.mdx": {
	id: "plugins/obsidian-go-to-line.mdx";
  slug: "plugins/obsidian-go-to-line";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-golinks.mdx": {
	id: "plugins/obsidian-golinks.mdx";
  slug: "plugins/obsidian-golinks";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-google-lookup.mdx": {
	id: "plugins/obsidian-google-lookup.mdx";
  slug: "plugins/obsidian-google-lookup";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-google-tasks.mdx": {
	id: "plugins/obsidian-google-tasks.mdx";
  slug: "plugins/obsidian-google-tasks";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-gpt.mdx": {
	id: "plugins/obsidian-gpt.mdx";
  slug: "plugins/obsidian-gpt";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-grandfather.mdx": {
	id: "plugins/obsidian-grandfather.mdx";
  slug: "plugins/obsidian-grandfather";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-graphviz.mdx": {
	id: "plugins/obsidian-graphviz.mdx";
  slug: "plugins/obsidian-graphviz";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-group-snippets.mdx": {
	id: "plugins/obsidian-group-snippets.mdx";
  slug: "plugins/obsidian-group-snippets";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-habit-tracker.mdx": {
	id: "plugins/obsidian-habit-tracker.mdx";
  slug: "plugins/obsidian-habit-tracker";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-habitica-integration.mdx": {
	id: "plugins/obsidian-habitica-integration.mdx";
  slug: "plugins/obsidian-habitica-integration";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-hackernews.mdx": {
	id: "plugins/obsidian-hackernews.mdx";
  slug: "plugins/obsidian-hackernews";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-handlebars.mdx": {
	id: "plugins/obsidian-handlebars.mdx";
  slug: "plugins/obsidian-handlebars";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-heading-shifter.mdx": {
	id: "plugins/obsidian-heading-shifter.mdx";
  slug: "plugins/obsidian-heading-shifter";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-hide-sidebars-when-narrow.mdx": {
	id: "plugins/obsidian-hide-sidebars-when-narrow.mdx";
  slug: "plugins/obsidian-hide-sidebars-when-narrow";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-hider.mdx": {
	id: "plugins/obsidian-hider.mdx";
  slug: "plugins/obsidian-hider";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-highlightpublicnotes-plugin.mdx": {
	id: "plugins/obsidian-highlightpublicnotes-plugin.mdx";
  slug: "plugins/obsidian-highlightpublicnotes-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-hotkeys-chords.mdx": {
	id: "plugins/obsidian-hotkeys-chords.mdx";
  slug: "plugins/obsidian-hotkeys-chords";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-hotkeys-for-specific-files.mdx": {
	id: "plugins/obsidian-hotkeys-for-specific-files.mdx";
  slug: "plugins/obsidian-hotkeys-for-specific-files";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-hotkeys-for-templates.mdx": {
	id: "plugins/obsidian-hotkeys-for-templates.mdx";
  slug: "plugins/obsidian-hotkeys-for-templates";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-hover-editor.mdx": {
	id: "plugins/obsidian-hover-editor.mdx";
  slug: "plugins/obsidian-hover-editor";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-html-plugin.mdx": {
	id: "plugins/obsidian-html-plugin.mdx";
  slug: "plugins/obsidian-html-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-html-tags-autocomplete.mdx": {
	id: "plugins/obsidian-html-tags-autocomplete.mdx";
  slug: "plugins/obsidian-html-tags-autocomplete";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-hyphenation.mdx": {
	id: "plugins/obsidian-hyphenation.mdx";
  slug: "plugins/obsidian-hyphenation";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-hypothesis-plugin.mdx": {
	id: "plugins/obsidian-hypothesis-plugin.mdx";
  slug: "plugins/obsidian-hypothesis-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-icon-folder.mdx": {
	id: "plugins/obsidian-icon-folder.mdx";
  slug: "plugins/obsidian-icon-folder";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-icon-shortcodes.mdx": {
	id: "plugins/obsidian-icon-shortcodes.mdx";
  slug: "plugins/obsidian-icon-shortcodes";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-icon-swapper.mdx": {
	id: "plugins/obsidian-icon-swapper.mdx";
  slug: "plugins/obsidian-icon-swapper";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-icons-plugin.mdx": {
	id: "plugins/obsidian-icons-plugin.mdx";
  slug: "plugins/obsidian-icons-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-image-auto-upload-plugin.mdx": {
	id: "plugins/obsidian-image-auto-upload-plugin.mdx";
  slug: "plugins/obsidian-image-auto-upload-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-image-caption.mdx": {
	id: "plugins/obsidian-image-caption.mdx";
  slug: "plugins/obsidian-image-caption";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-image-gallery.mdx": {
	id: "plugins/obsidian-image-gallery.mdx";
  slug: "plugins/obsidian-image-gallery";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-image-layouts.mdx": {
	id: "plugins/obsidian-image-layouts.mdx";
  slug: "plugins/obsidian-image-layouts";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-image-toolkit.mdx": {
	id: "plugins/obsidian-image-toolkit.mdx";
  slug: "plugins/obsidian-image-toolkit";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-image-uploader.mdx": {
	id: "plugins/obsidian-image-uploader.mdx";
  slug: "plugins/obsidian-image-uploader";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-imgur-plugin.mdx": {
	id: "plugins/obsidian-imgur-plugin.mdx";
  slug: "plugins/obsidian-imgur-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-import-json.mdx": {
	id: "plugins/obsidian-import-json.mdx";
  slug: "plugins/obsidian-import-json";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-importer.mdx": {
	id: "plugins/obsidian-importer.mdx";
  slug: "plugins/obsidian-importer";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-incremental-writing.mdx": {
	id: "plugins/obsidian-incremental-writing.mdx";
  slug: "plugins/obsidian-incremental-writing";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-indent-lines.mdx": {
	id: "plugins/obsidian-indent-lines.mdx";
  slug: "plugins/obsidian-indent-lines";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-indentation-guides.mdx": {
	id: "plugins/obsidian-indentation-guides.mdx";
  slug: "plugins/obsidian-indentation-guides";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-initiative-tracker.mdx": {
	id: "plugins/obsidian-initiative-tracker.mdx";
  slug: "plugins/obsidian-initiative-tracker";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-itinerary.mdx": {
	id: "plugins/obsidian-itinerary.mdx";
  slug: "plugins/obsidian-itinerary";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-ivre-plugin.mdx": {
	id: "plugins/obsidian-ivre-plugin.mdx";
  slug: "plugins/obsidian-ivre-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-javascript-init.mdx": {
	id: "plugins/obsidian-javascript-init.mdx";
  slug: "plugins/obsidian-javascript-init";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-jira-issue.mdx": {
	id: "plugins/obsidian-jira-issue.mdx";
  slug: "plugins/obsidian-jira-issue";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-journey-plugin.mdx": {
	id: "plugins/obsidian-journey-plugin.mdx";
  slug: "plugins/obsidian-journey-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-jsonifier.mdx": {
	id: "plugins/obsidian-jsonifier.mdx";
  slug: "plugins/obsidian-jsonifier";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-jtab.mdx": {
	id: "plugins/obsidian-jtab.mdx";
  slug: "plugins/obsidian-jtab";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-juliandate.mdx": {
	id: "plugins/obsidian-juliandate.mdx";
  slug: "plugins/obsidian-juliandate";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-jump-to-date-plugin.mdx": {
	id: "plugins/obsidian-jump-to-date-plugin.mdx";
  slug: "plugins/obsidian-jump-to-date-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-jupyter.mdx": {
	id: "plugins/obsidian-jupyter.mdx";
  slug: "plugins/obsidian-jupyter";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-kanban.mdx": {
	id: "plugins/obsidian-kanban.mdx";
  slug: "plugins/obsidian-kanban";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-key-sequence-shortcut.mdx": {
	id: "plugins/obsidian-key-sequence-shortcut.mdx";
  slug: "plugins/obsidian-key-sequence-shortcut";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-kindle-export.mdx": {
	id: "plugins/obsidian-kindle-export.mdx";
  slug: "plugins/obsidian-kindle-export";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-kindle-plugin.mdx": {
	id: "plugins/obsidian-kindle-plugin.mdx";
  slug: "plugins/obsidian-kindle-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-kobo-highlights-importer-plugin.mdx": {
	id: "plugins/obsidian-kobo-highlights-importer-plugin.mdx";
  slug: "plugins/obsidian-kobo-highlights-importer-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-koreader-plugin.mdx": {
	id: "plugins/obsidian-koreader-plugin.mdx";
  slug: "plugins/obsidian-koreader-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-kroki.mdx": {
	id: "plugins/obsidian-kroki.mdx";
  slug: "plugins/obsidian-kroki";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-languagetool-plugin.mdx": {
	id: "plugins/obsidian-languagetool-plugin.mdx";
  slug: "plugins/obsidian-languagetool-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-latex-environments.mdx": {
	id: "plugins/obsidian-latex-environments.mdx";
  slug: "plugins/obsidian-latex-environments";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-latex-preamble-plugin.mdx": {
	id: "plugins/obsidian-latex-preamble-plugin.mdx";
  slug: "plugins/obsidian-latex-preamble-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-latex-suite.mdx": {
	id: "plugins/obsidian-latex-suite.mdx";
  slug: "plugins/obsidian-latex-suite";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-latex.mdx": {
	id: "plugins/obsidian-latex.mdx";
  slug: "plugins/obsidian-latex";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-leaflet-plugin.mdx": {
	id: "plugins/obsidian-leaflet-plugin.mdx";
  slug: "plugins/obsidian-leaflet-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-limelight.mdx": {
	id: "plugins/obsidian-limelight.mdx";
  slug: "plugins/obsidian-limelight";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-lineup-builder.mdx": {
	id: "plugins/obsidian-lineup-builder.mdx";
  slug: "plugins/obsidian-lineup-builder";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-link-archive.mdx": {
	id: "plugins/obsidian-link-archive.mdx";
  slug: "plugins/obsidian-link-archive";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-link-converter.mdx": {
	id: "plugins/obsidian-link-converter.mdx";
  slug: "plugins/obsidian-link-converter";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-link-embed.mdx": {
	id: "plugins/obsidian-link-embed.mdx";
  slug: "plugins/obsidian-link-embed";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-link-indexer.mdx": {
	id: "plugins/obsidian-link-indexer.mdx";
  slug: "plugins/obsidian-link-indexer";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-link-opener.mdx": {
	id: "plugins/obsidian-link-opener.mdx";
  slug: "plugins/obsidian-link-opener";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-linter.mdx": {
	id: "plugins/obsidian-linter.mdx";
  slug: "plugins/obsidian-linter";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-list-callouts.mdx": {
	id: "plugins/obsidian-list-callouts.mdx";
  slug: "plugins/obsidian-list-callouts";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-list-modified.mdx": {
	id: "plugins/obsidian-list-modified.mdx";
  slug: "plugins/obsidian-list-modified";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-livesync.mdx": {
	id: "plugins/obsidian-livesync.mdx";
  slug: "plugins/obsidian-livesync";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-living-graph.mdx": {
	id: "plugins/obsidian-living-graph.mdx";
  slug: "plugins/obsidian-living-graph";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-local-file-interface-plugin.mdx": {
	id: "plugins/obsidian-local-file-interface-plugin.mdx";
  slug: "plugins/obsidian-local-file-interface-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-local-images-plus.mdx": {
	id: "plugins/obsidian-local-images-plus.mdx";
  slug: "plugins/obsidian-local-images-plus";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-local-images.mdx": {
	id: "plugins/obsidian-local-images.mdx";
  slug: "plugins/obsidian-local-images";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-local-rest-api.mdx": {
	id: "plugins/obsidian-local-rest-api.mdx";
  slug: "plugins/obsidian-local-rest-api";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-lock-screen-plugin.mdx": {
	id: "plugins/obsidian-lock-screen-plugin.mdx";
  slug: "plugins/obsidian-lock-screen-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-map-view.mdx": {
	id: "plugins/obsidian-map-view.mdx";
  slug: "plugins/obsidian-map-view";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-mark-and-select.mdx": {
	id: "plugins/obsidian-mark-and-select.mdx";
  slug: "plugins/obsidian-mark-and-select";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-markbase.mdx": {
	id: "plugins/obsidian-markbase.mdx";
  slug: "plugins/obsidian-markbase";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-markdown-export-plugin.mdx": {
	id: "plugins/obsidian-markdown-export-plugin.mdx";
  slug: "plugins/obsidian-markdown-export-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-markdown-file-suffix.mdx": {
	id: "plugins/obsidian-markdown-file-suffix.mdx";
  slug: "plugins/obsidian-markdown-file-suffix";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-markdown-formatting-assistant-plugin.mdx": {
	id: "plugins/obsidian-markdown-formatting-assistant-plugin.mdx";
  slug: "plugins/obsidian-markdown-formatting-assistant-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-markdown-furigana.mdx": {
	id: "plugins/obsidian-markdown-furigana.mdx";
  slug: "plugins/obsidian-markdown-furigana";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-markmind.mdx": {
	id: "plugins/obsidian-markmind.mdx";
  slug: "plugins/obsidian-markmind";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-math-plus.mdx": {
	id: "plugins/obsidian-math-plus.mdx";
  slug: "plugins/obsidian-math-plus";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-mathjax-wikilinks.mdx": {
	id: "plugins/obsidian-mathjax-wikilinks.mdx";
  slug: "plugins/obsidian-mathjax-wikilinks";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-matrix.mdx": {
	id: "plugins/obsidian-matrix.mdx";
  slug: "plugins/obsidian-matrix";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-md-to-jira.mdx": {
	id: "plugins/obsidian-md-to-jira.mdx";
  slug: "plugins/obsidian-md-to-jira";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-media-db-plugin.mdx": {
	id: "plugins/obsidian-media-db-plugin.mdx";
  slug: "plugins/obsidian-media-db-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-meeting-notes.mdx": {
	id: "plugins/obsidian-meeting-notes.mdx";
  slug: "plugins/obsidian-meeting-notes";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-memos.mdx": {
	id: "plugins/obsidian-memos.mdx";
  slug: "plugins/obsidian-memos";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-meta-bind-plugin.mdx": {
	id: "plugins/obsidian-meta-bind-plugin.mdx";
  slug: "plugins/obsidian-meta-bind-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-metacopy.mdx": {
	id: "plugins/obsidian-metacopy.mdx";
  slug: "plugins/obsidian-metacopy";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-metatable.mdx": {
	id: "plugins/obsidian-metatable.mdx";
  slug: "plugins/obsidian-metatable";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-metatemplates.mdx": {
	id: "plugins/obsidian-metatemplates.mdx";
  slug: "plugins/obsidian-metatemplates";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-metronome-plugin.mdx": {
	id: "plugins/obsidian-metronome-plugin.mdx";
  slug: "plugins/obsidian-metronome-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-min-width.mdx": {
	id: "plugins/obsidian-min-width.mdx";
  slug: "plugins/obsidian-min-width";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-min3ditorhotkeys-plugin.mdx": {
	id: "plugins/obsidian-min3ditorhotkeys-plugin.mdx";
  slug: "plugins/obsidian-min3ditorhotkeys-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-mind-map.mdx": {
	id: "plugins/obsidian-mind-map.mdx";
  slug: "plugins/obsidian-mind-map";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-mindmap-nextgen.mdx": {
	id: "plugins/obsidian-mindmap-nextgen.mdx";
  slug: "plugins/obsidian-mindmap-nextgen";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-minimal-settings.mdx": {
	id: "plugins/obsidian-minimal-settings.mdx";
  slug: "plugins/obsidian-minimal-settings";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-mkdocs-publisher.mdx": {
	id: "plugins/obsidian-mkdocs-publisher.mdx";
  slug: "plugins/obsidian-mkdocs-publisher";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-mtg.mdx": {
	id: "plugins/obsidian-mtg.mdx";
  slug: "plugins/obsidian-mtg";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-native-scrollbars.mdx": {
	id: "plugins/obsidian-native-scrollbars.mdx";
  slug: "plugins/obsidian-native-scrollbars";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-new-bullet-with-time.mdx": {
	id: "plugins/obsidian-new-bullet-with-time.mdx";
  slug: "plugins/obsidian-new-bullet-with-time";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-new-note-new-window.mdx": {
	id: "plugins/obsidian-new-note-new-window.mdx";
  slug: "plugins/obsidian-new-note-new-window";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-nomnoml-diagram.mdx": {
	id: "plugins/obsidian-nomnoml-diagram.mdx";
  slug: "plugins/obsidian-nomnoml-diagram";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-note-autocreator.mdx": {
	id: "plugins/obsidian-note-autocreator.mdx";
  slug: "plugins/obsidian-note-autocreator";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-note-content-pusher.mdx": {
	id: "plugins/obsidian-note-content-pusher.mdx";
  slug: "plugins/obsidian-note-content-pusher";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-notes-from-template.mdx": {
	id: "plugins/obsidian-notes-from-template.mdx";
  slug: "plugins/obsidian-notes-from-template";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-notion-video.mdx": {
	id: "plugins/obsidian-notion-video.mdx";
  slug: "plugins/obsidian-notion-video";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-ocr.mdx": {
	id: "plugins/obsidian-ocr.mdx";
  slug: "plugins/obsidian-ocr";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-old-note-admonitor.mdx": {
	id: "plugins/obsidian-old-note-admonitor.mdx";
  slug: "plugins/obsidian-old-note-admonitor";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-omnivore.mdx": {
	id: "plugins/obsidian-omnivore.mdx";
  slug: "plugins/obsidian-omnivore";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-open-file-by-magic-date.mdx": {
	id: "plugins/obsidian-open-file-by-magic-date.mdx";
  slug: "plugins/obsidian-open-file-by-magic-date";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-open-in-other-editor.mdx": {
	id: "plugins/obsidian-open-in-other-editor.mdx";
  slug: "plugins/obsidian-open-in-other-editor";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-open-link-with.mdx": {
	id: "plugins/obsidian-open-link-with.mdx";
  slug: "plugins/obsidian-open-link-with";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-org-mode.mdx": {
	id: "plugins/obsidian-org-mode.mdx";
  slug: "plugins/obsidian-org-mode";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-orthography.mdx": {
	id: "plugins/obsidian-orthography.mdx";
  slug: "plugins/obsidian-orthography";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-oura-plugin.mdx": {
	id: "plugins/obsidian-oura-plugin.mdx";
  slug: "plugins/obsidian-oura-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-outliner.mdx": {
	id: "plugins/obsidian-outliner.mdx";
  slug: "plugins/obsidian-outliner";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-overdue.mdx": {
	id: "plugins/obsidian-overdue.mdx";
  slug: "plugins/obsidian-overdue";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-pandoc-reference-list.mdx": {
	id: "plugins/obsidian-pandoc-reference-list.mdx";
  slug: "plugins/obsidian-pandoc-reference-list";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-pandoc.mdx": {
	id: "plugins/obsidian-pandoc.mdx";
  slug: "plugins/obsidian-pandoc";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-pangu.mdx": {
	id: "plugins/obsidian-pangu.mdx";
  slug: "plugins/obsidian-pangu";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-paper-cut.mdx": {
	id: "plugins/obsidian-paper-cut.mdx";
  slug: "plugins/obsidian-paper-cut";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-party.mdx": {
	id: "plugins/obsidian-party.mdx";
  slug: "plugins/obsidian-party";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-paste-as-html.mdx": {
	id: "plugins/obsidian-paste-as-html.mdx";
  slug: "plugins/obsidian-paste-as-html";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-paste-image-rename.mdx": {
	id: "plugins/obsidian-paste-image-rename.mdx";
  slug: "plugins/obsidian-paste-image-rename";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-paste-png-to-jpeg.mdx": {
	id: "plugins/obsidian-paste-png-to-jpeg.mdx";
  slug: "plugins/obsidian-paste-png-to-jpeg";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-paste-to-current-indentation.mdx": {
	id: "plugins/obsidian-paste-to-current-indentation.mdx";
  slug: "plugins/obsidian-paste-to-current-indentation";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-path-finder.mdx": {
	id: "plugins/obsidian-path-finder.mdx";
  slug: "plugins/obsidian-path-finder";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-path-title.mdx": {
	id: "plugins/obsidian-path-title.mdx";
  slug: "plugins/obsidian-path-title";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-pending-notes.mdx": {
	id: "plugins/obsidian-pending-notes.mdx";
  slug: "plugins/obsidian-pending-notes";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-pikt.mdx": {
	id: "plugins/obsidian-pikt.mdx";
  slug: "plugins/obsidian-pikt";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-pipe-tricks.mdx": {
	id: "plugins/obsidian-pipe-tricks.mdx";
  slug: "plugins/obsidian-pipe-tricks";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-plaintext.mdx": {
	id: "plugins/obsidian-plaintext.mdx";
  slug: "plugins/obsidian-plaintext";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-plantuml.mdx": {
	id: "plugins/obsidian-plantuml.mdx";
  slug: "plugins/obsidian-plantuml";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-plotly.mdx": {
	id: "plugins/obsidian-plotly.mdx";
  slug: "plugins/obsidian-plotly";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-pluck.mdx": {
	id: "plugins/obsidian-pluck.mdx";
  slug: "plugins/obsidian-pluck";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-plugin-dynamodb.mdx": {
	id: "plugins/obsidian-plugin-dynamodb.mdx";
  slug: "plugins/obsidian-plugin-dynamodb";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-plugin-groups.mdx": {
	id: "plugins/obsidian-plugin-groups.mdx";
  slug: "plugins/obsidian-plugin-groups";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-plugin-prettier.mdx": {
	id: "plugins/obsidian-plugin-prettier.mdx";
  slug: "plugins/obsidian-plugin-prettier";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-plugin-tagged-documents-viewer.mdx": {
	id: "plugins/obsidian-plugin-tagged-documents-viewer.mdx";
  slug: "plugins/obsidian-plugin-tagged-documents-viewer";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-plugin-time-diff.mdx": {
	id: "plugins/obsidian-plugin-time-diff.mdx";
  slug: "plugins/obsidian-plugin-time-diff";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-plugin-toc.mdx": {
	id: "plugins/obsidian-plugin-toc.mdx";
  slug: "plugins/obsidian-plugin-toc";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-plugin-todo.mdx": {
	id: "plugins/obsidian-plugin-todo.mdx";
  slug: "plugins/obsidian-plugin-todo";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-plugin-update-tracker.mdx": {
	id: "plugins/obsidian-plugin-update-tracker.mdx";
  slug: "plugins/obsidian-plugin-update-tracker";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-pocket.mdx": {
	id: "plugins/obsidian-pocket.mdx";
  slug: "plugins/obsidian-pocket";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-pomodoro-plugin.mdx": {
	id: "plugins/obsidian-pomodoro-plugin.mdx";
  slug: "plugins/obsidian-pomodoro-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-power-search.mdx": {
	id: "plugins/obsidian-power-search.mdx";
  slug: "plugins/obsidian-power-search";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-pretty-bibtex.mdx": {
	id: "plugins/obsidian-pretty-bibtex.mdx";
  slug: "plugins/obsidian-pretty-bibtex";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-project-garden.mdx": {
	id: "plugins/obsidian-project-garden.mdx";
  slug: "plugins/obsidian-project-garden";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-projects.mdx": {
	id: "plugins/obsidian-projects.mdx";
  slug: "plugins/obsidian-projects";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-prominent-starred-files.mdx": {
	id: "plugins/obsidian-prominent-starred-files.mdx";
  slug: "plugins/obsidian-prominent-starred-files";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-prozen.mdx": {
	id: "plugins/obsidian-prozen.mdx";
  slug: "plugins/obsidian-prozen";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-qrcode-plugin.mdx": {
	id: "plugins/obsidian-qrcode-plugin.mdx";
  slug: "plugins/obsidian-qrcode-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-query-language.mdx": {
	id: "plugins/obsidian-query-language.mdx";
  slug: "plugins/obsidian-query-language";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-query2table.mdx": {
	id: "plugins/obsidian-query2table.mdx";
  slug: "plugins/obsidian-query2table";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-quickshare.mdx": {
	id: "plugins/obsidian-quickshare.mdx";
  slug: "plugins/obsidian-quickshare";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-quiet-outline.mdx": {
	id: "plugins/obsidian-quiet-outline.mdx";
  slug: "plugins/obsidian-quiet-outline";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-quran-lookup.mdx": {
	id: "plugins/obsidian-quran-lookup.mdx";
  slug: "plugins/obsidian-quran-lookup";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-raindrop-highlights.mdx": {
	id: "plugins/obsidian-raindrop-highlights.mdx";
  slug: "plugins/obsidian-raindrop-highlights";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-random-todo.mdx": {
	id: "plugins/obsidian-random-todo.mdx";
  slug: "plugins/obsidian-random-todo";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-rant.mdx": {
	id: "plugins/obsidian-rant.mdx";
  slug: "plugins/obsidian-rant";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-rapid-notes.mdx": {
	id: "plugins/obsidian-rapid-notes.mdx";
  slug: "plugins/obsidian-rapid-notes";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-react-components.mdx": {
	id: "plugins/obsidian-react-components.mdx";
  slug: "plugins/obsidian-react-components";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-read-it-later.mdx": {
	id: "plugins/obsidian-read-it-later.mdx";
  slug: "plugins/obsidian-read-it-later";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-reading-time.mdx": {
	id: "plugins/obsidian-reading-time.mdx";
  slug: "plugins/obsidian-reading-time";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-readlater.mdx": {
	id: "plugins/obsidian-readlater.mdx";
  slug: "plugins/obsidian-readlater";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-readwise.mdx": {
	id: "plugins/obsidian-readwise.mdx";
  slug: "plugins/obsidian-readwise";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-recall.mdx": {
	id: "plugins/obsidian-recall.mdx";
  slug: "plugins/obsidian-recall";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-redirect.mdx": {
	id: "plugins/obsidian-redirect.mdx";
  slug: "plugins/obsidian-redirect";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-regex-pipeline.mdx": {
	id: "plugins/obsidian-regex-pipeline.mdx";
  slug: "plugins/obsidian-regex-pipeline";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-regex-replace.mdx": {
	id: "plugins/obsidian-regex-replace.mdx";
  slug: "plugins/obsidian-regex-replace";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-related-notes-finder.mdx": {
	id: "plugins/obsidian-related-notes-finder.mdx";
  slug: "plugins/obsidian-related-notes-finder";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-relation-pane.mdx": {
	id: "plugins/obsidian-relation-pane.mdx";
  slug: "plugins/obsidian-relation-pane";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-relative-find.mdx": {
	id: "plugins/obsidian-relative-find.mdx";
  slug: "plugins/obsidian-relative-find";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-relative-line-numbers.mdx": {
	id: "plugins/obsidian-relative-line-numbers.mdx";
  slug: "plugins/obsidian-relative-line-numbers";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-relativenumber.mdx": {
	id: "plugins/obsidian-relativenumber.mdx";
  slug: "plugins/obsidian-relativenumber";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-release-timeline.mdx": {
	id: "plugins/obsidian-release-timeline.mdx";
  slug: "plugins/obsidian-release-timeline";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-remember-file-state.mdx": {
	id: "plugins/obsidian-remember-file-state.mdx";
  slug: "plugins/obsidian-remember-file-state";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-reminder-plugin.mdx": {
	id: "plugins/obsidian-reminder-plugin.mdx";
  slug: "plugins/obsidian-reminder-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-reset-font-size.mdx": {
	id: "plugins/obsidian-reset-font-size.mdx";
  slug: "plugins/obsidian-reset-font-size";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-reveal-active-file.mdx": {
	id: "plugins/obsidian-reveal-active-file.mdx";
  slug: "plugins/obsidian-reveal-active-file";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-review-notes-plugin.mdx": {
	id: "plugins/obsidian-review-notes-plugin.mdx";
  slug: "plugins/obsidian-review-notes-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-rewarder.mdx": {
	id: "plugins/obsidian-rewarder.mdx";
  slug: "plugins/obsidian-rewarder";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-rich-links.mdx": {
	id: "plugins/obsidian-rich-links.mdx";
  slug: "plugins/obsidian-rich-links";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-rollover-daily-todos.mdx": {
	id: "plugins/obsidian-rollover-daily-todos.mdx";
  slug: "plugins/obsidian-rollover-daily-todos";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-rtl.mdx": {
	id: "plugins/obsidian-rtl.mdx";
  slug: "plugins/obsidian-rtl";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-sakana-widget.mdx": {
	id: "plugins/obsidian-sakana-widget.mdx";
  slug: "plugins/obsidian-sakana-widget";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-sample-plugin.mdx": {
	id: "plugins/obsidian-sample-plugin.mdx";
  slug: "plugins/obsidian-sample-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-save-as-gist.mdx": {
	id: "plugins/obsidian-save-as-gist.mdx";
  slug: "plugins/obsidian-save-as-gist";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-screwdriver.mdx": {
	id: "plugins/obsidian-screwdriver.mdx";
  slug: "plugins/obsidian-screwdriver";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-scroll-offset.mdx": {
	id: "plugins/obsidian-scroll-offset.mdx";
  slug: "plugins/obsidian-scroll-offset";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-scroll-to-top-plugin.mdx": {
	id: "plugins/obsidian-scroll-to-top-plugin.mdx";
  slug: "plugins/obsidian-scroll-to-top-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-search-everywhere-plugin.mdx": {
	id: "plugins/obsidian-search-everywhere-plugin.mdx";
  slug: "plugins/obsidian-search-everywhere-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-sentence-navigator.mdx": {
	id: "plugins/obsidian-sentence-navigator.mdx";
  slug: "plugins/obsidian-sentence-navigator";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-sequence-hotkeys.mdx": {
	id: "plugins/obsidian-sequence-hotkeys.mdx";
  slug: "plugins/obsidian-sequence-hotkeys";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-set-mobile-theme.mdx": {
	id: "plugins/obsidian-set-mobile-theme.mdx";
  slug: "plugins/obsidian-set-mobile-theme";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-share-as-gist.mdx": {
	id: "plugins/obsidian-share-as-gist.mdx";
  slug: "plugins/obsidian-share-as-gist";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-shellcommands.mdx": {
	id: "plugins/obsidian-shellcommands.mdx";
  slug: "plugins/obsidian-shellcommands";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-shortcut-launcher.mdx": {
	id: "plugins/obsidian-shortcut-launcher.mdx";
  slug: "plugins/obsidian-shortcut-launcher";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-shortcuts-for-starred-files.mdx": {
	id: "plugins/obsidian-shortcuts-for-starred-files.mdx";
  slug: "plugins/obsidian-shortcuts-for-starred-files";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-show-file-path.mdx": {
	id: "plugins/obsidian-show-file-path.mdx";
  slug: "plugins/obsidian-show-file-path";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-sidebar-expand-on-hover.mdx": {
	id: "plugins/obsidian-sidebar-expand-on-hover.mdx";
  slug: "plugins/obsidian-sidebar-expand-on-hover";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-sidebar-toggler.mdx": {
	id: "plugins/obsidian-sidebar-toggler.mdx";
  slug: "plugins/obsidian-sidebar-toggler";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-simple-mention.mdx": {
	id: "plugins/obsidian-simple-mention.mdx";
  slug: "plugins/obsidian-simple-mention";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-siteswap.mdx": {
	id: "plugins/obsidian-siteswap.mdx";
  slug: "plugins/obsidian-siteswap";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-smart-links.mdx": {
	id: "plugins/obsidian-smart-links.mdx";
  slug: "plugins/obsidian-smart-links";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-smart-typography.mdx": {
	id: "plugins/obsidian-smart-typography.mdx";
  slug: "plugins/obsidian-smart-typography";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-smarter-md-hotkeys.mdx": {
	id: "plugins/obsidian-smarter-md-hotkeys.mdx";
  slug: "plugins/obsidian-smarter-md-hotkeys";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-snippet-downloader.mdx": {
	id: "plugins/obsidian-snippet-downloader.mdx";
  slug: "plugins/obsidian-snippet-downloader";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-snippetor.mdx": {
	id: "plugins/obsidian-snippetor.mdx";
  slug: "plugins/obsidian-snippetor";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-soomda.mdx": {
	id: "plugins/obsidian-soomda.mdx";
  slug: "plugins/obsidian-soomda";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-sort-and-permute-lines.mdx": {
	id: "plugins/obsidian-sort-and-permute-lines.mdx";
  slug: "plugins/obsidian-sort-and-permute-lines";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-sortable.mdx": {
	id: "plugins/obsidian-sortable.mdx";
  slug: "plugins/obsidian-sortable";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-spaced-repetition.mdx": {
	id: "plugins/obsidian-spaced-repetition.mdx";
  slug: "plugins/obsidian-spaced-repetition";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-spotlight.mdx": {
	id: "plugins/obsidian-spotlight.mdx";
  slug: "plugins/obsidian-spotlight";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-stack-overflow.mdx": {
	id: "plugins/obsidian-stack-overflow.mdx";
  slug: "plugins/obsidian-stack-overflow";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-state-switcher.mdx": {
	id: "plugins/obsidian-state-switcher.mdx";
  slug: "plugins/obsidian-state-switcher";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-static-file-server.mdx": {
	id: "plugins/obsidian-static-file-server.mdx";
  slug: "plugins/obsidian-static-file-server";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-statusbar-pomo.mdx": {
	id: "plugins/obsidian-statusbar-pomo.mdx";
  slug: "plugins/obsidian-statusbar-pomo";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-steemit.mdx": {
	id: "plugins/obsidian-steemit.mdx";
  slug: "plugins/obsidian-steemit";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-stille.mdx": {
	id: "plugins/obsidian-stille.mdx";
  slug: "plugins/obsidian-stille";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-stopwatch-plugin.mdx": {
	id: "plugins/obsidian-stopwatch-plugin.mdx";
  slug: "plugins/obsidian-stopwatch-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-structured-plugin.mdx": {
	id: "plugins/obsidian-structured-plugin.mdx";
  slug: "plugins/obsidian-structured-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-style-settings.mdx": {
	id: "plugins/obsidian-style-settings.mdx";
  slug: "plugins/obsidian-style-settings";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-stylist.mdx": {
	id: "plugins/obsidian-stylist.mdx";
  slug: "plugins/obsidian-stylist";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-system-dark-mode.mdx": {
	id: "plugins/obsidian-system-dark-mode.mdx";
  slug: "plugins/obsidian-system-dark-mode";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-system-theme.mdx": {
	id: "plugins/obsidian-system-theme.mdx";
  slug: "plugins/obsidian-system-theme";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-table-generator.mdx": {
	id: "plugins/obsidian-table-generator.mdx";
  slug: "plugins/obsidian-table-generator";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-table-to-csv-exporter.mdx": {
	id: "plugins/obsidian-table-to-csv-exporter.mdx";
  slug: "plugins/obsidian-table-to-csv-exporter";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-tabs.mdx": {
	id: "plugins/obsidian-tabs.mdx";
  slug: "plugins/obsidian-tabs";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-tagfolder.mdx": {
	id: "plugins/obsidian-tagfolder.mdx";
  slug: "plugins/obsidian-tagfolder";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-task-archiver.mdx": {
	id: "plugins/obsidian-task-archiver.mdx";
  slug: "plugins/obsidian-task-archiver";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-task-collector.mdx": {
	id: "plugins/obsidian-task-collector.mdx";
  slug: "plugins/obsidian-task-collector";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-task-marker.mdx": {
	id: "plugins/obsidian-task-marker.mdx";
  slug: "plugins/obsidian-task-marker";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-task-progress-bar.mdx": {
	id: "plugins/obsidian-task-progress-bar.mdx";
  slug: "plugins/obsidian-task-progress-bar";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-tasks-plugin.mdx": {
	id: "plugins/obsidian-tasks-plugin.mdx";
  slug: "plugins/obsidian-tasks-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-telegraph-publish.mdx": {
	id: "plugins/obsidian-telegraph-publish.mdx";
  slug: "plugins/obsidian-telegraph-publish";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-temple.mdx": {
	id: "plugins/obsidian-temple.mdx";
  slug: "plugins/obsidian-temple";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-text-expander-js.mdx": {
	id: "plugins/obsidian-text-expander-js.mdx";
  slug: "plugins/obsidian-text-expander-js";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-text-expander.mdx": {
	id: "plugins/obsidian-text-expander.mdx";
  slug: "plugins/obsidian-text-expander";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-text-format.mdx": {
	id: "plugins/obsidian-text-format.mdx";
  slug: "plugins/obsidian-text-format";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-textgenerator-plugin.mdx": {
	id: "plugins/obsidian-textgenerator-plugin.mdx";
  slug: "plugins/obsidian-textgenerator-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-theme-design-utilities.mdx": {
	id: "plugins/obsidian-theme-design-utilities.mdx";
  slug: "plugins/obsidian-theme-design-utilities";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-theme-toggler.mdx": {
	id: "plugins/obsidian-theme-toggler.mdx";
  slug: "plugins/obsidian-theme-toggler";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-things-link.mdx": {
	id: "plugins/obsidian-things-link.mdx";
  slug: "plugins/obsidian-things-link";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-things3-sync.mdx": {
	id: "plugins/obsidian-things3-sync.mdx";
  slug: "plugins/obsidian-things3-sync";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-thumbnails.mdx": {
	id: "plugins/obsidian-thumbnails.mdx";
  slug: "plugins/obsidian-thumbnails";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-tidy-footnotes.mdx": {
	id: "plugins/obsidian-tidy-footnotes.mdx";
  slug: "plugins/obsidian-tidy-footnotes";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-tikzjax.mdx": {
	id: "plugins/obsidian-tikzjax.mdx";
  slug: "plugins/obsidian-tikzjax";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-timeline.mdx": {
	id: "plugins/obsidian-timeline.mdx";
  slug: "plugins/obsidian-timeline";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-timelines.mdx": {
	id: "plugins/obsidian-timelines.mdx";
  slug: "plugins/obsidian-timelines";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-timestamp-notes.mdx": {
	id: "plugins/obsidian-timestamp-notes.mdx";
  slug: "plugins/obsidian-timestamp-notes";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-timestamper.mdx": {
	id: "plugins/obsidian-timestamper.mdx";
  slug: "plugins/obsidian-timestamper";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-title-index.mdx": {
	id: "plugins/obsidian-title-index.mdx";
  slug: "plugins/obsidian-title-index";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-title-serial-number-plugin.mdx": {
	id: "plugins/obsidian-title-serial-number-plugin.mdx";
  slug: "plugins/obsidian-title-serial-number-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-to-anki-plugin.mdx": {
	id: "plugins/obsidian-to-anki-plugin.mdx";
  slug: "plugins/obsidian-to-anki-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-to-flomo.mdx": {
	id: "plugins/obsidian-to-flomo.mdx";
  slug: "plugins/obsidian-to-flomo";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-to-notion.mdx": {
	id: "plugins/obsidian-to-notion.mdx";
  slug: "plugins/obsidian-to-notion";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-todoist-link.mdx": {
	id: "plugins/obsidian-todoist-link.mdx";
  slug: "plugins/obsidian-todoist-link";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-toggl-integration.mdx": {
	id: "plugins/obsidian-toggl-integration.mdx";
  slug: "plugins/obsidian-toggl-integration";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-toggle-case.mdx": {
	id: "plugins/obsidian-toggle-case.mdx";
  slug: "plugins/obsidian-toggle-case";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-toggle-list.mdx": {
	id: "plugins/obsidian-toggle-list.mdx";
  slug: "plugins/obsidian-toggle-list";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-toggle-meta-yaml-plugin.mdx": {
	id: "plugins/obsidian-toggle-meta-yaml-plugin.mdx";
  slug: "plugins/obsidian-toggle-meta-yaml-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-tomorrows-daily-note.mdx": {
	id: "plugins/obsidian-tomorrows-daily-note.mdx";
  slug: "plugins/obsidian-tomorrows-daily-note";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-topic-linking.mdx": {
	id: "plugins/obsidian-topic-linking.mdx";
  slug: "plugins/obsidian-topic-linking";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-tracker.mdx": {
	id: "plugins/obsidian-tracker.mdx";
  slug: "plugins/obsidian-tracker";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-transcription.mdx": {
	id: "plugins/obsidian-transcription.mdx";
  slug: "plugins/obsidian-transcription";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-translator.mdx": {
	id: "plugins/obsidian-translator.mdx";
  slug: "plugins/obsidian-translator";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-trash-explorer.mdx": {
	id: "plugins/obsidian-trash-explorer.mdx";
  slug: "plugins/obsidian-trash-explorer";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-trello.mdx": {
	id: "plugins/obsidian-trello.mdx";
  slug: "plugins/obsidian-trello";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-tressel.mdx": {
	id: "plugins/obsidian-tressel.mdx";
  slug: "plugins/obsidian-tressel";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-trim-whitespace.mdx": {
	id: "plugins/obsidian-trim-whitespace.mdx";
  slug: "plugins/obsidian-trim-whitespace";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-tts.mdx": {
	id: "plugins/obsidian-tts.mdx";
  slug: "plugins/obsidian-tts";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-tweaks.mdx": {
	id: "plugins/obsidian-tweaks.mdx";
  slug: "plugins/obsidian-tweaks";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-tweet-to-markdown.mdx": {
	id: "plugins/obsidian-tweet-to-markdown.mdx";
  slug: "plugins/obsidian-tweet-to-markdown";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-underline.mdx": {
	id: "plugins/obsidian-underline.mdx";
  slug: "plugins/obsidian-underline";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-upcoming.mdx": {
	id: "plugins/obsidian-upcoming.mdx";
  slug: "plugins/obsidian-upcoming";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-user-plugins.mdx": {
	id: "plugins/obsidian-user-plugins.mdx";
  slug: "plugins/obsidian-user-plugins";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-vale.mdx": {
	id: "plugins/obsidian-vale.mdx";
  slug: "plugins/obsidian-vale";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-vault-changelog.mdx": {
	id: "plugins/obsidian-vault-changelog.mdx";
  slug: "plugins/obsidian-vault-changelog";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-vault-statistics-plugin.mdx": {
	id: "plugins/obsidian-vault-statistics-plugin.mdx";
  slug: "plugins/obsidian-vault-statistics-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-vega.mdx": {
	id: "plugins/obsidian-vega.mdx";
  slug: "plugins/obsidian-vega";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-version-history-diff.mdx": {
	id: "plugins/obsidian-version-history-diff.mdx";
  slug: "plugins/obsidian-version-history-diff";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-view-mode-by-frontmatter.mdx": {
	id: "plugins/obsidian-view-mode-by-frontmatter.mdx";
  slug: "plugins/obsidian-view-mode-by-frontmatter";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-vim-im-switch-plugin.mdx": {
	id: "plugins/obsidian-vim-im-switch-plugin.mdx";
  slug: "plugins/obsidian-vim-im-switch-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-vim-multibyte-char-search.mdx": {
	id: "plugins/obsidian-vim-multibyte-char-search.mdx";
  slug: "plugins/obsidian-vim-multibyte-char-search";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-vimrc-support.mdx": {
	id: "plugins/obsidian-vimrc-support.mdx";
  slug: "plugins/obsidian-vimrc-support";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-vocabulary-view.mdx": {
	id: "plugins/obsidian-vocabulary-view.mdx";
  slug: "plugins/obsidian-vocabulary-view";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-wakatime.mdx": {
	id: "plugins/obsidian-wakatime.mdx";
  slug: "plugins/obsidian-wakatime";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-wavedrom.mdx": {
	id: "plugins/obsidian-wavedrom.mdx";
  slug: "plugins/obsidian-wavedrom";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-webhooks.mdx": {
	id: "plugins/obsidian-webhooks.mdx";
  slug: "plugins/obsidian-webhooks";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-week-planner.mdx": {
	id: "plugins/obsidian-week-planner.mdx";
  slug: "plugins/obsidian-week-planner";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-weread-plugin.mdx": {
	id: "plugins/obsidian-weread-plugin.mdx";
  slug: "plugins/obsidian-weread-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-wikipedia.mdx": {
	id: "plugins/obsidian-wikipedia.mdx";
  slug: "plugins/obsidian-wikipedia";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-word-sprint.mdx": {
	id: "plugins/obsidian-word-sprint.mdx";
  slug: "plugins/obsidian-word-sprint";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-wordle.mdx": {
	id: "plugins/obsidian-wordle.mdx";
  slug: "plugins/obsidian-wordle";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-wordnet-plugin.mdx": {
	id: "plugins/obsidian-wordnet-plugin.mdx";
  slug: "plugins/obsidian-wordnet-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-wordnik.mdx": {
	id: "plugins/obsidian-wordnik.mdx";
  slug: "plugins/obsidian-wordnik";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-wordpress.mdx": {
	id: "plugins/obsidian-wordpress.mdx";
  slug: "plugins/obsidian-wordpress";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-wordy.mdx": {
	id: "plugins/obsidian-wordy.mdx";
  slug: "plugins/obsidian-wordy";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-wrap-with-shortcuts.mdx": {
	id: "plugins/obsidian-wrap-with-shortcuts.mdx";
  slug: "plugins/obsidian-wrap-with-shortcuts";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-yfm-templater-plugin.mdx": {
	id: "plugins/obsidian-yfm-templater-plugin.mdx";
  slug: "plugins/obsidian-yfm-templater-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-youglish-plugin.mdx": {
	id: "plugins/obsidian-youglish-plugin.mdx";
  slug: "plugins/obsidian-youglish-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-zoom.mdx": {
	id: "plugins/obsidian-zoom.mdx";
  slug: "plugins/obsidian-zoom";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian-zotero-desktop-connector.mdx": {
	id: "plugins/obsidian-zotero-desktop-connector.mdx";
  slug: "plugins/obsidian-zotero-desktop-connector";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian42-brat.mdx": {
	id: "plugins/obsidian42-brat.mdx";
  slug: "plugins/obsidian42-brat";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian42-strange-new-worlds.mdx": {
	id: "plugins/obsidian42-strange-new-worlds.mdx";
  slug: "plugins/obsidian42-strange-new-worlds";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidian42-text-transporter.mdx": {
	id: "plugins/obsidian42-text-transporter.mdx";
  slug: "plugins/obsidian42-text-transporter";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidiosaurus.mdx": {
	id: "plugins/obsidiosaurus.mdx";
  slug: "plugins/obsidiosaurus";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsidoom.mdx": {
	id: "plugins/obsidoom.mdx";
  slug: "plugins/obsidoom";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsimian-exporter.mdx": {
	id: "plugins/obsimian-exporter.mdx";
  slug: "plugins/obsimian-exporter";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsimian.mdx": {
	id: "plugins/obsimian.mdx";
  slug: "plugins/obsimian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/obsius-publish.mdx": {
	id: "plugins/obsius-publish.mdx";
  slug: "plugins/obsius-publish";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/oin-gotoheading.mdx": {
	id: "plugins/oin-gotoheading.mdx";
  slug: "plugins/oin-gotoheading";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/ollama.mdx": {
	id: "plugins/ollama.mdx";
  slug: "plugins/ollama";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/omglol-statuslog-publish.mdx": {
	id: "plugins/omglol-statuslog-publish.mdx";
  slug: "plugins/omglol-statuslog-publish";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/omnisearch.mdx": {
	id: "plugins/omnisearch.mdx";
  slug: "plugins/omnisearch";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/onyx-boox-extractor.mdx": {
	id: "plugins/onyx-boox-extractor.mdx";
  slug: "plugins/onyx-boox-extractor";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/open-files-with-commands.mdx": {
	id: "plugins/open-files-with-commands.mdx";
  slug: "plugins/open-files-with-commands";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/open-gate.mdx": {
	id: "plugins/open-gate.mdx";
  slug: "plugins/open-gate";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/open-in-new-tab.mdx": {
	id: "plugins/open-in-new-tab.mdx";
  slug: "plugins/open-in-new-tab";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/open-note-to-window-title.mdx": {
	id: "plugins/open-note-to-window-title.mdx";
  slug: "plugins/open-note-to-window-title";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/open-plugin-settings.mdx": {
	id: "plugins/open-plugin-settings.mdx";
  slug: "plugins/open-plugin-settings";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/open-related-url.mdx": {
	id: "plugins/open-related-url.mdx";
  slug: "plugins/open-related-url";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/open-vscode.mdx": {
	id: "plugins/open-vscode.mdx";
  slug: "plugins/open-vscode";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/open-weather.mdx": {
	id: "plugins/open-weather.mdx";
  slug: "plugins/open-weather";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/open-with.mdx": {
	id: "plugins/open-with.mdx";
  slug: "plugins/open-with";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/optimize-canvas-connections.mdx": {
	id: "plugins/optimize-canvas-connections.mdx";
  slug: "plugins/optimize-canvas-connections";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/order-list.mdx": {
	id: "plugins/order-list.mdx";
  slug: "plugins/order-list";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/oz-calendar.mdx": {
	id: "plugins/oz-calendar.mdx";
  slug: "plugins/oz-calendar";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/oz-clear-unused-images.mdx": {
	id: "plugins/oz-clear-unused-images.mdx";
  slug: "plugins/oz-clear-unused-images";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/oz-image-plugin.mdx": {
	id: "plugins/oz-image-plugin.mdx";
  slug: "plugins/oz-image-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/ozanshare-publish.mdx": {
	id: "plugins/ozanshare-publish.mdx";
  slug: "plugins/ozanshare-publish";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/page-gallery.mdx": {
	id: "plugins/page-gallery.mdx";
  slug: "plugins/page-gallery";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/page-heading-from-links.mdx": {
	id: "plugins/page-heading-from-links.mdx";
  slug: "plugins/page-heading-from-links";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/page-properties.mdx": {
	id: "plugins/page-properties.mdx";
  slug: "plugins/page-properties";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/palta-note.mdx": {
	id: "plugins/palta-note.mdx";
  slug: "plugins/palta-note";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/pane-relief.mdx": {
	id: "plugins/pane-relief.mdx";
  slug: "plugins/pane-relief";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/para-shortcuts.mdx": {
	id: "plugins/para-shortcuts.mdx";
  slug: "plugins/para-shortcuts";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/password-protection.mdx": {
	id: "plugins/password-protection.mdx";
  slug: "plugins/password-protection";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/paste-link.mdx": {
	id: "plugins/paste-link.mdx";
  slug: "plugins/paste-link";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/pdf-to-markdown-plugin.mdx": {
	id: "plugins/pdf-to-markdown-plugin.mdx";
  slug: "plugins/pdf-to-markdown-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/perilous-writing.mdx": {
	id: "plugins/perilous-writing.mdx";
  slug: "plugins/perilous-writing";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/periodic-notes.mdx": {
	id: "plugins/periodic-notes.mdx";
  slug: "plugins/periodic-notes";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/periodic-para.mdx": {
	id: "plugins/periodic-para.mdx";
  slug: "plugins/periodic-para";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/periodic-table.mdx": {
	id: "plugins/periodic-table.mdx";
  slug: "plugins/periodic-table";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/permalink-opener.mdx": {
	id: "plugins/permalink-opener.mdx";
  slug: "plugins/permalink-opener";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/persistent-graph.mdx": {
	id: "plugins/persistent-graph.mdx";
  slug: "plugins/persistent-graph";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/persistent-links.mdx": {
	id: "plugins/persistent-links.mdx";
  slug: "plugins/persistent-links";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/personal-assistant.mdx": {
	id: "plugins/personal-assistant.mdx";
  slug: "plugins/personal-assistant";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/pf2-action-icons.mdx": {
	id: "plugins/pf2-action-icons.mdx";
  slug: "plugins/pf2-action-icons";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/phone-to-roam-to-obsidian.mdx": {
	id: "plugins/phone-to-roam-to-obsidian.mdx";
  slug: "plugins/phone-to-roam-to-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/pickly-page-blend.mdx": {
	id: "plugins/pickly-page-blend.mdx";
  slug: "plugins/pickly-page-blend";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/pieces-for-developers.mdx": {
	id: "plugins/pieces-for-developers.mdx";
  slug: "plugins/pieces-for-developers";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/pinboard-sync.mdx": {
	id: "plugins/pinboard-sync.mdx";
  slug: "plugins/pinboard-sync";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/pivotal-tracker-integration.mdx": {
	id: "plugins/pivotal-tracker-integration.mdx";
  slug: "plugins/pivotal-tracker-integration";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/plugin-changelogs.mdx": {
	id: "plugins/plugin-changelogs.mdx";
  slug: "plugins/plugin-changelogs";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/plugin-manager.mdx": {
	id: "plugins/plugin-manager.mdx";
  slug: "plugins/plugin-manager";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/plugins-galore.mdx": {
	id: "plugins/plugins-galore.mdx";
  slug: "plugins/plugins-galore";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/pocketbook-cloud-highlight-importer.mdx": {
	id: "plugins/pocketbook-cloud-highlight-importer.mdx";
  slug: "plugins/pocketbook-cloud-highlight-importer";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/podcast-note.mdx": {
	id: "plugins/podcast-note.mdx";
  slug: "plugins/podcast-note";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/podnotes.mdx": {
	id: "plugins/podnotes.mdx";
  slug: "plugins/podnotes";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/poker.mdx": {
	id: "plugins/poker.mdx";
  slug: "plugins/poker";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/postfix.mdx": {
	id: "plugins/postfix.mdx";
  slug: "plugins/postfix";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/postgresql-obsidian.mdx": {
	id: "plugins/postgresql-obsidian.mdx";
  slug: "plugins/postgresql-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/potato-indexer.mdx": {
	id: "plugins/potato-indexer.mdx";
  slug: "plugins/potato-indexer";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/prioritize.mdx": {
	id: "plugins/prioritize.mdx";
  slug: "plugins/prioritize";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/privacy-glasses.mdx": {
	id: "plugins/privacy-glasses.mdx";
  slug: "plugins/privacy-glasses";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/progressbar.mdx": {
	id: "plugins/progressbar.mdx";
  slug: "plugins/progressbar";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/prompt.mdx": {
	id: "plugins/prompt.mdx";
  slug: "plugins/prompt";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/pseudocode-in-obs.mdx": {
	id: "plugins/pseudocode-in-obs.mdx";
  slug: "plugins/pseudocode-in-obs";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/publish-url.mdx": {
	id: "plugins/publish-url.mdx";
  slug: "plugins/publish-url";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/pubscale.mdx": {
	id: "plugins/pubscale.mdx";
  slug: "plugins/pubscale";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/python-lab-plugin.mdx": {
	id: "plugins/python-lab-plugin.mdx";
  slug: "plugins/python-lab-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/python-scripter.mdx": {
	id: "plugins/python-scripter.mdx";
  slug: "plugins/python-scripter";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/qatt.mdx": {
	id: "plugins/qatt.mdx";
  slug: "plugins/qatt";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/qb-reader-parser.mdx": {
	id: "plugins/qb-reader-parser.mdx";
  slug: "plugins/qb-reader-parser";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/qiniu-image-uploader.mdx": {
	id: "plugins/qiniu-image-uploader.mdx";
  slug: "plugins/qiniu-image-uploader";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/qmd-as-md-obsidian.mdx": {
	id: "plugins/qmd-as-md-obsidian.mdx";
  slug: "plugins/qmd-as-md-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/quail.mdx": {
	id: "plugins/quail.mdx";
  slug: "plugins/quail";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/quick-explorer.mdx": {
	id: "plugins/quick-explorer.mdx";
  slug: "plugins/quick-explorer";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/quick-latex.mdx": {
	id: "plugins/quick-latex.mdx";
  slug: "plugins/quick-latex";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/quick-links.mdx": {
	id: "plugins/quick-links.mdx";
  slug: "plugins/quick-links";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/quick-plugin-switcher.mdx": {
	id: "plugins/quick-plugin-switcher.mdx";
  slug: "plugins/quick-plugin-switcher";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/quick-preview.mdx": {
	id: "plugins/quick-preview.mdx";
  slug: "plugins/quick-preview";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/quick-snippets-and-navigation.mdx": {
	id: "plugins/quick-snippets-and-navigation.mdx";
  slug: "plugins/quick-snippets-and-navigation";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/quick-tagger.mdx": {
	id: "plugins/quick-tagger.mdx";
  slug: "plugins/quick-tagger";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/quickadd.mdx": {
	id: "plugins/quickadd.mdx";
  slug: "plugins/quickadd";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/quickly.mdx": {
	id: "plugins/quickly.mdx";
  slug: "plugins/quickly";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/quicknote.mdx": {
	id: "plugins/quicknote.mdx";
  slug: "plugins/quicknote";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/quip.mdx": {
	id: "plugins/quip.mdx";
  slug: "plugins/quip";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/quote-of-the-day.mdx": {
	id: "plugins/quote-of-the-day.mdx";
  slug: "plugins/quote-of-the-day";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/quote-share.mdx": {
	id: "plugins/quote-share.mdx";
  slug: "plugins/quote-share";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/quoth.mdx": {
	id: "plugins/quoth.mdx";
  slug: "plugins/quoth";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/random-numbers-generator.mdx": {
	id: "plugins/random-numbers-generator.mdx";
  slug: "plugins/random-numbers-generator";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/random-structural-diary-plugin.mdx": {
	id: "plugins/random-structural-diary-plugin.mdx";
  slug: "plugins/random-structural-diary-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/readability-score.mdx": {
	id: "plugins/readability-score.mdx";
  slug: "plugins/readability-score";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/readavocado-sync.mdx": {
	id: "plugins/readavocado-sync.mdx";
  slug: "plugins/readavocado-sync";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/reading-comments.mdx": {
	id: "plugins/reading-comments.mdx";
  slug: "plugins/reading-comments";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/reading-view-enhancer.mdx": {
	id: "plugins/reading-view-enhancer.mdx";
  slug: "plugins/reading-view-enhancer";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/readwise-mirror.mdx": {
	id: "plugins/readwise-mirror.mdx";
  slug: "plugins/readwise-mirror";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/readwise-official.mdx": {
	id: "plugins/readwise-official.mdx";
  slug: "plugins/readwise-official";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/recent-files-obsidian.mdx": {
	id: "plugins/recent-files-obsidian.mdx";
  slug: "plugins/recent-files-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/recipe-grabber.mdx": {
	id: "plugins/recipe-grabber.mdx";
  slug: "plugins/recipe-grabber";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/recipe-view.mdx": {
	id: "plugins/recipe-view.mdx";
  slug: "plugins/recipe-view";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/reclipped-official.mdx": {
	id: "plugins/reclipped-official.mdx";
  slug: "plugins/reclipped-official";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/red-pen.mdx": {
	id: "plugins/red-pen.mdx";
  slug: "plugins/red-pen";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/reference-map.mdx": {
	id: "plugins/reference-map.mdx";
  slug: "plugins/reference-map";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/reflection.mdx": {
	id: "plugins/reflection.mdx";
  slug: "plugins/reflection";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/remember-cursor-position.mdx": {
	id: "plugins/remember-cursor-position.mdx";
  slug: "plugins/remember-cursor-position";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/remotely-save.mdx": {
	id: "plugins/remotely-save.mdx";
  slug: "plugins/remotely-save";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/remotely-secure.mdx": {
	id: "plugins/remotely-secure.mdx";
  slug: "plugins/remotely-secure";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/remove-empty-folders.mdx": {
	id: "plugins/remove-empty-folders.mdx";
  slug: "plugins/remove-empty-folders";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/rendered-block-link-suggestions.mdx": {
	id: "plugins/rendered-block-link-suggestions.mdx";
  slug: "plugins/rendered-block-link-suggestions";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/repeat-plugin.mdx": {
	id: "plugins/repeat-plugin.mdx";
  slug: "plugins/repeat-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/rescuetime.mdx": {
	id: "plugins/rescuetime.mdx";
  slug: "plugins/rescuetime";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/restore-tab-key.mdx": {
	id: "plugins/restore-tab-key.mdx";
  slug: "plugins/restore-tab-key";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/reveal-active-file-button.mdx": {
	id: "plugins/reveal-active-file-button.mdx";
  slug: "plugins/reveal-active-file-button";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/reveal-file-in-explorer.mdx": {
	id: "plugins/reveal-file-in-explorer.mdx";
  slug: "plugins/reveal-file-in-explorer";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/review-obsidian.mdx": {
	id: "plugins/review-obsidian.mdx";
  slug: "plugins/review-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/ring-a-secretary.mdx": {
	id: "plugins/ring-a-secretary.mdx";
  slug: "plugins/ring-a-secretary";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/rofi-helper.mdx": {
	id: "plugins/rofi-helper.mdx";
  slug: "plugins/rofi-helper";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/rpg-manager.mdx": {
	id: "plugins/rpg-manager.mdx";
  slug: "plugins/rpg-manager";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/rss-reader.mdx": {
	id: "plugins/rss-reader.mdx";
  slug: "plugins/rss-reader";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/ruled-template.mdx": {
	id: "plugins/ruled-template.mdx";
  slug: "plugins/ruled-template";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/run.mdx": {
	id: "plugins/run.mdx";
  slug: "plugins/run";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/runjs.mdx": {
	id: "plugins/runjs.mdx";
  slug: "plugins/runjs";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/s3-attachments-storage.mdx": {
	id: "plugins/s3-attachments-storage.mdx";
  slug: "plugins/s3-attachments-storage";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/s3-image-uploader.mdx": {
	id: "plugins/s3-image-uploader.mdx";
  slug: "plugins/s3-image-uploader";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/safe-filename-linter.mdx": {
	id: "plugins/safe-filename-linter.mdx";
  slug: "plugins/safe-filename-linter";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/samepage.mdx": {
	id: "plugins/samepage.mdx";
  slug: "plugins/samepage";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/scales-chords.mdx": {
	id: "plugins/scales-chords.mdx";
  slug: "plugins/scales-chords";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/scholar.mdx": {
	id: "plugins/scholar.mdx";
  slug: "plugins/scholar";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/screengarden-obsidian.mdx": {
	id: "plugins/screengarden-obsidian.mdx";
  slug: "plugins/screengarden-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/script-launcher.mdx": {
	id: "plugins/script-launcher.mdx";
  slug: "plugins/script-launcher";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/scroll-speed.mdx": {
	id: "plugins/scroll-speed.mdx";
  slug: "plugins/scroll-speed";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/scrybble.ink.mdx": {
	id: "plugins/scrybble.ink.mdx";
  slug: "plugins/scrybbleink";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/search-obsidian-in-google.mdx": {
	id: "plugins/search-obsidian-in-google.mdx";
  slug: "plugins/search-obsidian-in-google";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/search-on-internet.mdx": {
	id: "plugins/search-on-internet.mdx";
  slug: "plugins/search-on-internet";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/searchpp.mdx": {
	id: "plugins/searchpp.mdx";
  slug: "plugins/searchpp";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/sekund.mdx": {
	id: "plugins/sekund.mdx";
  slug: "plugins/sekund";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/select-current-line.mdx": {
	id: "plugins/select-current-line.mdx";
  slug: "plugins/select-current-line";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/send-to-ghost.mdx": {
	id: "plugins/send-to-ghost.mdx";
  slug: "plugins/send-to-ghost";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/set-in-obsidian.mdx": {
	id: "plugins/set-in-obsidian.mdx";
  slug: "plugins/set-in-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/sets.mdx": {
	id: "plugins/sets.mdx";
  slug: "plugins/sets";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/settings-search.mdx": {
	id: "plugins/settings-search.mdx";
  slug: "plugins/settings-search";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/share-note.mdx": {
	id: "plugins/share-note.mdx";
  slug: "plugins/share-note";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/share-to-cubox.mdx": {
	id: "plugins/share-to-cubox.mdx";
  slug: "plugins/share-to-cubox";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/share-to-notionnext.mdx": {
	id: "plugins/share-to-notionnext.mdx";
  slug: "plugins/share-to-notionnext";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/sheets.mdx": {
	id: "plugins/sheets.mdx";
  slug: "plugins/sheets";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/short-internal-links-to-headings.mdx": {
	id: "plugins/short-internal-links-to-headings.mdx";
  slug: "plugins/short-internal-links-to-headings";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/short-tab-name.mdx": {
	id: "plugins/short-tab-name.mdx";
  slug: "plugins/short-tab-name";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/shortcuts-extender.mdx": {
	id: "plugins/shortcuts-extender.mdx";
  slug: "plugins/shortcuts-extender";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/show-diff.mdx": {
	id: "plugins/show-diff.mdx";
  slug: "plugins/show-diff";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/show-whitespace-cm6.mdx": {
	id: "plugins/show-whitespace-cm6.mdx";
  slug: "plugins/show-whitespace-cm6";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/shukuchi.mdx": {
	id: "plugins/shukuchi.mdx";
  slug: "plugins/shukuchi";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/sigma.mdx": {
	id: "plugins/sigma.mdx";
  slug: "plugins/sigma";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/silicon.mdx": {
	id: "plugins/silicon.mdx";
  slug: "plugins/silicon";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/simple-canvasearch.mdx": {
	id: "plugins/simple-canvasearch.mdx";
  slug: "plugins/simple-canvasearch";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/simple-dice-roller.mdx": {
	id: "plugins/simple-dice-roller.mdx";
  slug: "plugins/simple-dice-roller";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/simple-embeds.mdx": {
	id: "plugins/simple-embeds.mdx";
  slug: "plugins/simple-embeds";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/simple-note-quiz.mdx": {
	id: "plugins/simple-note-quiz.mdx";
  slug: "plugins/simple-note-quiz";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/simple-note-review.mdx": {
	id: "plugins/simple-note-review.mdx";
  slug: "plugins/simple-note-review";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/simple-rss.mdx": {
	id: "plugins/simple-rss.mdx";
  slug: "plugins/simple-rss";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/simple-time-tracker.mdx": {
	id: "plugins/simple-time-tracker.mdx";
  slug: "plugins/simple-time-tracker";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/size-history.mdx": {
	id: "plugins/size-history.mdx";
  slug: "plugins/size-history";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/slackify-note.mdx": {
	id: "plugins/slackify-note.mdx";
  slug: "plugins/slackify-note";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/slated-obsidian.mdx": {
	id: "plugins/slated-obsidian.mdx";
  slug: "plugins/slated-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/slide-note.mdx": {
	id: "plugins/slide-note.mdx";
  slug: "plugins/slide-note";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/sliding-panes-obsidian.mdx": {
	id: "plugins/sliding-panes-obsidian.mdx";
  slug: "plugins/sliding-panes-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/smart-connections.mdx": {
	id: "plugins/smart-connections.mdx";
  slug: "plugins/smart-connections";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/smart-random-note.mdx": {
	id: "plugins/smart-random-note.mdx";
  slug: "plugins/smart-random-note";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/smart-rename.mdx": {
	id: "plugins/smart-rename.mdx";
  slug: "plugins/smart-rename";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/smort-obsidian.mdx": {
	id: "plugins/smort-obsidian.mdx";
  slug: "plugins/smort-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/snippet-commands-obsidian.mdx": {
	id: "plugins/snippet-commands-obsidian.mdx";
  slug: "plugins/snippet-commands-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/snippets.mdx": {
	id: "plugins/snippets.mdx";
  slug: "plugins/snippets";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/solve.mdx": {
	id: "plugins/solve.mdx";
  slug: "plugins/solve";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/soomda.mdx": {
	id: "plugins/soomda.mdx";
  slug: "plugins/soomda";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/sort-frontmatter.mdx": {
	id: "plugins/sort-frontmatter.mdx";
  slug: "plugins/sort-frontmatter";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/source-code-note.mdx": {
	id: "plugins/source-code-note.mdx";
  slug: "plugins/source-code-note";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/source-scanner.mdx": {
	id: "plugins/source-scanner.mdx";
  slug: "plugins/source-scanner";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/speech2text-helper.mdx": {
	id: "plugins/speech2text-helper.mdx";
  slug: "plugins/speech2text-helper";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/spoiler-block-obsidian.mdx": {
	id: "plugins/spoiler-block-obsidian.mdx";
  slug: "plugins/spoiler-block-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/spotify-link.mdx": {
	id: "plugins/spotify-link.mdx";
  slug: "plugins/spotify-link";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/spotify-links.mdx": {
	id: "plugins/spotify-links.mdx";
  slug: "plugins/spotify-links";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/spreadsheets.mdx": {
	id: "plugins/spreadsheets.mdx";
  slug: "plugins/spreadsheets";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/squiggle.mdx": {
	id: "plugins/squiggle.mdx";
  slug: "plugins/squiggle";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/status-bar-quote.mdx": {
	id: "plugins/status-bar-quote.mdx";
  slug: "plugins/status-bar-quote";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/stenography-obsidian.mdx": {
	id: "plugins/stenography-obsidian.mdx";
  slug: "plugins/stenography-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/studier.mdx": {
	id: "plugins/studier.mdx";
  slug: "plugins/studier";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/style-importer.mdx": {
	id: "plugins/style-importer.mdx";
  slug: "plugins/style-importer";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/style-text.mdx": {
	id: "plugins/style-text.mdx";
  slug: "plugins/style-text";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/supercharged-links-obsidian.mdx": {
	id: "plugins/supercharged-links-obsidian.mdx";
  slug: "plugins/supercharged-links-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/supsub.mdx": {
	id: "plugins/supsub.mdx";
  slug: "plugins/supsub";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/surfing.mdx": {
	id: "plugins/surfing.mdx";
  slug: "plugins/surfing";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/swipe-navigation.mdx": {
	id: "plugins/swipe-navigation.mdx";
  slug: "plugins/swipe-navigation";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/swiss-army-knife.mdx": {
	id: "plugins/swiss-army-knife.mdx";
  slug: "plugins/swiss-army-knife";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/symbols-prettifier.mdx": {
	id: "plugins/symbols-prettifier.mdx";
  slug: "plugins/symbols-prettifier";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/sync-contacts-macos.mdx": {
	id: "plugins/sync-contacts-macos.mdx";
  slug: "plugins/sync-contacts-macos";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/sync-google-calendar.mdx": {
	id: "plugins/sync-google-calendar.mdx";
  slug: "plugins/sync-google-calendar";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/sync-graph-settings.mdx": {
	id: "plugins/sync-graph-settings.mdx";
  slug: "plugins/sync-graph-settings";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/sync-to-xlog.mdx": {
	id: "plugins/sync-to-xlog.mdx";
  slug: "plugins/sync-to-xlog";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/sync-version-history.mdx": {
	id: "plugins/sync-version-history.mdx";
  slug: "plugins/sync-version-history";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/syncftp.mdx": {
	id: "plugins/syncftp.mdx";
  slug: "plugins/syncftp";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/syncthing-integration.mdx": {
	id: "plugins/syncthing-integration.mdx";
  slug: "plugins/syncthing-integration";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/tab-rotator.mdx": {
	id: "plugins/tab-rotator.mdx";
  slug: "plugins/tab-rotator";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/table-checkboxes.mdx": {
	id: "plugins/table-checkboxes.mdx";
  slug: "plugins/table-checkboxes";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/table-editor-obsidian.mdx": {
	id: "plugins/table-editor-obsidian.mdx";
  slug: "plugins/table-editor-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/table-extended.mdx": {
	id: "plugins/table-extended.mdx";
  slug: "plugins/table-extended";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/tabout.mdx": {
	id: "plugins/tabout.mdx";
  slug: "plugins/tabout";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/tag-breakdown-generator.mdx": {
	id: "plugins/tag-breakdown-generator.mdx";
  slug: "plugins/tag-breakdown-generator";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/tag-buddy.mdx": {
	id: "plugins/tag-buddy.mdx";
  slug: "plugins/tag-buddy";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/tag-many.mdx": {
	id: "plugins/tag-many.mdx";
  slug: "plugins/tag-many";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/tag-page-md.mdx": {
	id: "plugins/tag-page-md.mdx";
  slug: "plugins/tag-page-md";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/tag-page-preview.mdx": {
	id: "plugins/tag-page-preview.mdx";
  slug: "plugins/tag-page-preview";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/tag-project-odaimoko.mdx": {
	id: "plugins/tag-project-odaimoko.mdx";
  slug: "plugins/tag-project-odaimoko";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/tag-search.mdx": {
	id: "plugins/tag-search.mdx";
  slug: "plugins/tag-search";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/tag-summary-plugin.mdx": {
	id: "plugins/tag-summary-plugin.mdx";
  slug: "plugins/tag-summary-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/tag-word-cloud.mdx": {
	id: "plugins/tag-word-cloud.mdx";
  slug: "plugins/tag-word-cloud";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/tag-wrangler.mdx": {
	id: "plugins/tag-wrangler.mdx";
  slug: "plugins/tag-wrangler";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/tags-overview.mdx": {
	id: "plugins/tags-overview.mdx";
  slug: "plugins/tags-overview";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/taskbone-ocr-plugin.mdx": {
	id: "plugins/taskbone-ocr-plugin.mdx";
  slug: "plugins/taskbone-ocr-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/tasks-calendar-wrapper.mdx": {
	id: "plugins/tasks-calendar-wrapper.mdx";
  slug: "plugins/tasks-calendar-wrapper";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/tasks-packrat-plugin.mdx": {
	id: "plugins/tasks-packrat-plugin.mdx";
  slug: "plugins/tasks-packrat-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/tasks-to-omnifocus.mdx": {
	id: "plugins/tasks-to-omnifocus.mdx";
  slug: "plugins/tasks-to-omnifocus";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/tckr.mdx": {
	id: "plugins/tckr.mdx";
  slug: "plugins/tckr";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/telegram-sync.mdx": {
	id: "plugins/telegram-sync.mdx";
  slug: "plugins/telegram-sync";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/teleprompter.mdx": {
	id: "plugins/teleprompter.mdx";
  slug: "plugins/teleprompter";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/template-search-library.mdx": {
	id: "plugins/template-search-library.mdx";
  slug: "plugins/template-search-library";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/templater-obsidian.mdx": {
	id: "plugins/templater-obsidian.mdx";
  slug: "plugins/templater-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/tenki.mdx": {
	id: "plugins/tenki.mdx";
  slug: "plugins/tenki";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/terminal.mdx": {
	id: "plugins/terminal.mdx";
  slug: "plugins/terminal";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/testing-vault.mdx": {
	id: "plugins/testing-vault.mdx";
  slug: "plugins/testing-vault";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/text-conversions.mdx": {
	id: "plugins/text-conversions.mdx";
  slug: "plugins/text-conversions";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/text-extractor.mdx": {
	id: "plugins/text-extractor.mdx";
  slug: "plugins/text-extractor";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/text-progress-bar.mdx": {
	id: "plugins/text-progress-bar.mdx";
  slug: "plugins/text-progress-bar";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/text-snippets-obsidian.mdx": {
	id: "plugins/text-snippets-obsidian.mdx";
  slug: "plugins/text-snippets-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/text-transform.mdx": {
	id: "plugins/text-transform.mdx";
  slug: "plugins/text-transform";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/text2anki-openai.mdx": {
	id: "plugins/text2anki-openai.mdx";
  slug: "plugins/text2anki-openai";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/textanalysis.mdx": {
	id: "plugins/textanalysis.mdx";
  slug: "plugins/textanalysis";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/theme-picker.mdx": {
	id: "plugins/theme-picker.mdx";
  slug: "plugins/theme-picker";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/things-logbook.mdx": {
	id: "plugins/things-logbook.mdx";
  slug: "plugins/things-logbook";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/ticktick.mdx": {
	id: "plugins/ticktick.mdx";
  slug: "plugins/ticktick";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/tiddlywiki-import-export.mdx": {
	id: "plugins/tiddlywiki-import-export.mdx";
  slug: "plugins/tiddlywiki-import-export";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/time-ruler.mdx": {
	id: "plugins/time-ruler.mdx";
  slug: "plugins/time-ruler";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/timeline-view.mdx": {
	id: "plugins/timeline-view.mdx";
  slug: "plugins/timeline-view";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/timer.mdx": {
	id: "plugins/timer.mdx";
  slug: "plugins/timer";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/timestamp-link.mdx": {
	id: "plugins/timestamp-link.mdx";
  slug: "plugins/timestamp-link";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/timethings.mdx": {
	id: "plugins/timethings.mdx";
  slug: "plugins/timethings";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/timetracker.mdx": {
	id: "plugins/timetracker.mdx";
  slug: "plugins/timetracker";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/tinypng-image.mdx": {
	id: "plugins/tinypng-image.mdx";
  slug: "plugins/tinypng-image";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/tistory-poster.mdx": {
	id: "plugins/tistory-poster.mdx";
  slug: "plugins/tistory-poster";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/tistory-publisher.mdx": {
	id: "plugins/tistory-publisher.mdx";
  slug: "plugins/tistory-publisher";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/tistory.mdx": {
	id: "plugins/tistory.mdx";
  slug: "plugins/tistory";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/title-generator.mdx": {
	id: "plugins/title-generator.mdx";
  slug: "plugins/title-generator";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/todo-sort.mdx": {
	id: "plugins/todo-sort.mdx";
  slug: "plugins/todo-sort";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/todo-txt.mdx": {
	id: "plugins/todo-txt.mdx";
  slug: "plugins/todo-txt";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/todoist-completed-tasks-plugin.mdx": {
	id: "plugins/todoist-completed-tasks-plugin.mdx";
  slug: "plugins/todoist-completed-tasks-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/todoist-sync-plugin.mdx": {
	id: "plugins/todoist-sync-plugin.mdx";
  slug: "plugins/todoist-sync-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/todoist-text.mdx": {
	id: "plugins/todoist-text.mdx";
  slug: "plugins/todoist-text";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/todotxt-codeblocks.mdx": {
	id: "plugins/todotxt-codeblocks.mdx";
  slug: "plugins/todotxt-codeblocks";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/todotxt.mdx": {
	id: "plugins/todotxt.mdx";
  slug: "plugins/todotxt";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/tokei.mdx": {
	id: "plugins/tokei.mdx";
  slug: "plugins/tokei";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/tolino-notes-import.mdx": {
	id: "plugins/tolino-notes-import.mdx";
  slug: "plugins/tolino-notes-import";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/tor2e-statblocks.mdx": {
	id: "plugins/tor2e-statblocks.mdx";
  slug: "plugins/tor2e-statblocks";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/touchbar-macros.mdx": {
	id: "plugins/touchbar-macros.mdx";
  slug: "plugins/touchbar-macros";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/tq-obsidian.mdx": {
	id: "plugins/tq-obsidian.mdx";
  slug: "plugins/tq-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/tracker-plus.mdx": {
	id: "plugins/tracker-plus.mdx";
  slug: "plugins/tracker-plus";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/translate.mdx": {
	id: "plugins/translate.mdx";
  slug: "plugins/translate";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/tray.mdx": {
	id: "plugins/tray.mdx";
  slug: "plugins/tray";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/treefocus.mdx": {
	id: "plugins/treefocus.mdx";
  slug: "plugins/treefocus";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/txt-as-md-obsidian.mdx": {
	id: "plugins/txt-as-md-obsidian.mdx";
  slug: "plugins/txt-as-md-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/typewriter-mode.mdx": {
	id: "plugins/typewriter-mode.mdx";
  slug: "plugins/typewriter-mode";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/typing-assistant.mdx": {
	id: "plugins/typing-assistant.mdx";
  slug: "plugins/typing-assistant";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/typing-speed.mdx": {
	id: "plugins/typing-speed.mdx";
  slug: "plugins/typing-speed";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/typing-transformer-obsidian.mdx": {
	id: "plugins/typing-transformer-obsidian.mdx";
  slug: "plugins/typing-transformer-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/typing.mdx": {
	id: "plugins/typing.mdx";
  slug: "plugins/typing";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/typst.mdx": {
	id: "plugins/typst.mdx";
  slug: "plugins/typst";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/ultimate-todoist-sync.mdx": {
	id: "plugins/ultimate-todoist-sync.mdx";
  slug: "plugins/ultimate-todoist-sync";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/uncheck-all.mdx": {
	id: "plugins/uncheck-all.mdx";
  slug: "plugins/uncheck-all";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/unfilled-stats-highlighter.mdx": {
	id: "plugins/unfilled-stats-highlighter.mdx";
  slug: "plugins/unfilled-stats-highlighter";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/unicode-search.mdx": {
	id: "plugins/unicode-search.mdx";
  slug: "plugins/unicode-search";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/unique-attachments.mdx": {
	id: "plugins/unique-attachments.mdx";
  slug: "plugins/unique-attachments";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/unitade.mdx": {
	id: "plugins/unitade.mdx";
  slug: "plugins/unitade";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/update-relative-links.mdx": {
	id: "plugins/update-relative-links.mdx";
  slug: "plugins/update-relative-links";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/update-time-on-edit.mdx": {
	id: "plugins/update-time-on-edit.mdx";
  slug: "plugins/update-time-on-edit";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/uri-commands.mdx": {
	id: "plugins/uri-commands.mdx";
  slug: "plugins/uri-commands";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/url-display.mdx": {
	id: "plugins/url-display.mdx";
  slug: "plugins/url-display";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/url-into-selection.mdx": {
	id: "plugins/url-into-selection.mdx";
  slug: "plugins/url-into-selection";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/url-namer.mdx": {
	id: "plugins/url-namer.mdx";
  slug: "plugins/url-namer";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/vantage-obsidian.mdx": {
	id: "plugins/vantage-obsidian.mdx";
  slug: "plugins/vantage-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/various-complements.mdx": {
	id: "plugins/various-complements.mdx";
  slug: "plugins/various-complements";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/vault-2-book.mdx": {
	id: "plugins/vault-2-book.mdx";
  slug: "plugins/vault-2-book";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/vault-chat.mdx": {
	id: "plugins/vault-chat.mdx";
  slug: "plugins/vault-chat";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/vault-name-status-bar.mdx": {
	id: "plugins/vault-name-status-bar.mdx";
  slug: "plugins/vault-name-status-bar";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/vertical-tabs-view.mdx": {
	id: "plugins/vertical-tabs-view.mdx";
  slug: "plugins/vertical-tabs-view";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/vextab.mdx": {
	id: "plugins/vextab.mdx";
  slug: "plugins/vextab";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/vika-sync.mdx": {
	id: "plugins/vika-sync.mdx";
  slug: "plugins/vika-sync";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/vim-im-select.mdx": {
	id: "plugins/vim-im-select.mdx";
  slug: "plugins/vim-im-select";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/vim-toggle.mdx": {
	id: "plugins/vim-toggle.mdx";
  slug: "plugins/vim-toggle";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/vocabulary-cards.mdx": {
	id: "plugins/vocabulary-cards.mdx";
  slug: "plugins/vocabulary-cards";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/vocabulary-highlighter.mdx": {
	id: "plugins/vocabulary-highlighter.mdx";
  slug: "plugins/vocabulary-highlighter";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/voice.mdx": {
	id: "plugins/voice.mdx";
  slug: "plugins/voice";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/vox.mdx": {
	id: "plugins/vox.mdx";
  slug: "plugins/vox";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/vscode-editor.mdx": {
	id: "plugins/vscode-editor.mdx";
  slug: "plugins/vscode-editor";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/waka_time_box.mdx": {
	id: "plugins/waka_time_box.mdx";
  slug: "plugins/waka_time_box";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/wallabag.mdx": {
	id: "plugins/wallabag.mdx";
  slug: "plugins/wallabag";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/waypoint.mdx": {
	id: "plugins/waypoint.mdx";
  slug: "plugins/waypoint";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/weather-fetcher.mdx": {
	id: "plugins/weather-fetcher.mdx";
  slug: "plugins/weather-fetcher";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/webdav-file-explorer.mdx": {
	id: "plugins/webdav-file-explorer.mdx";
  slug: "plugins/webdav-file-explorer";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/webpage-html-export.mdx": {
	id: "plugins/webpage-html-export.mdx";
  slug: "plugins/webpage-html-export";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/weekly-review.mdx": {
	id: "plugins/weekly-review.mdx";
  slug: "plugins/weekly-review";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/whisper.mdx": {
	id: "plugins/whisper.mdx";
  slug: "plugins/whisper";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/wielder.mdx": {
	id: "plugins/wielder.mdx";
  slug: "plugins/wielder";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/wikidata-importer.mdx": {
	id: "plugins/wikidata-importer.mdx";
  slug: "plugins/wikidata-importer";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/wikilinks-to-mdlinks-obsidian.mdx": {
	id: "plugins/wikilinks-to-mdlinks-obsidian.mdx";
  slug: "plugins/wikilinks-to-mdlinks-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/wikipedia-search.mdx": {
	id: "plugins/wikipedia-search.mdx";
  slug: "plugins/wikipedia-search";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/window-collapse.mdx": {
	id: "plugins/window-collapse.mdx";
  slug: "plugins/window-collapse";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/word-bank.mdx": {
	id: "plugins/word-bank.mdx";
  slug: "plugins/word-bank";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/workbench-obsidian.mdx": {
	id: "plugins/workbench-obsidian.mdx";
  slug: "plugins/workbench-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/workbooks.mdx": {
	id: "plugins/workbooks.mdx";
  slug: "plugins/workbooks";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/workona-to-obsidian.mdx": {
	id: "plugins/workona-to-obsidian.mdx";
  slug: "plugins/workona-to-obsidian";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/workspaces-plus.mdx": {
	id: "plugins/workspaces-plus.mdx";
  slug: "plugins/workspaces-plus";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/wrangle-todos.mdx": {
	id: "plugins/wrangle-todos.mdx";
  slug: "plugins/wrangle-todos";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/writeas-publisher.mdx": {
	id: "plugins/writeas-publisher.mdx";
  slug: "plugins/writeas-publisher";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/writing-goals.mdx": {
	id: "plugins/writing-goals.mdx";
  slug: "plugins/writing-goals";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/writing.mdx": {
	id: "plugins/writing.mdx";
  slug: "plugins/writing";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/wucai-highlights-official.mdx": {
	id: "plugins/wucai-highlights-official.mdx";
  slug: "plugins/wucai-highlights-official";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/x86-flow-graphing.mdx": {
	id: "plugins/x86-flow-graphing.mdx";
  slug: "plugins/x86-flow-graphing";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/yet-another-obsidian-synchronizer.mdx": {
	id: "plugins/yet-another-obsidian-synchronizer.mdx";
  slug: "plugins/yet-another-obsidian-synchronizer";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/youhavebeenstaring-plugin.mdx": {
	id: "plugins/youhavebeenstaring-plugin.mdx";
  slug: "plugins/youhavebeenstaring-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/youversion-linker.mdx": {
	id: "plugins/youversion-linker.mdx";
  slug: "plugins/youversion-linker";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/ytranscript.mdx": {
	id: "plugins/ytranscript.mdx";
  slug: "plugins/ytranscript";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/zen.mdx": {
	id: "plugins/zen.mdx";
  slug: "plugins/zen";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/zettelflow.mdx": {
	id: "plugins/zettelflow.mdx";
  slug: "plugins/zettelflow";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/zettelgpt.mdx": {
	id: "plugins/zettelgpt.mdx";
  slug: "plugins/zettelgpt";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/zettelkasten-llm-tools.mdx": {
	id: "plugins/zettelkasten-llm-tools.mdx";
  slug: "plugins/zettelkasten-llm-tools";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/zettelkasten-outliner.mdx": {
	id: "plugins/zettelkasten-outliner.mdx";
  slug: "plugins/zettelkasten-outliner";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/zhongwen-block.mdx": {
	id: "plugins/zhongwen-block.mdx";
  slug: "plugins/zhongwen-block";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/zoottelkeeper-obsidian-plugin.mdx": {
	id: "plugins/zoottelkeeper-obsidian-plugin.mdx";
  slug: "plugins/zoottelkeeper-obsidian-plugin";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/zotero-bridge.mdx": {
	id: "plugins/zotero-bridge.mdx";
  slug: "plugins/zotero-bridge";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/zotero-link.mdx": {
	id: "plugins/zotero-link.mdx";
  slug: "plugins/zotero-link";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/zotero-sync-client.mdx": {
	id: "plugins/zotero-sync-client.mdx";
  slug: "plugins/zotero-sync-client";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
"plugins/zotlit.mdx": {
	id: "plugins/zotlit.mdx";
  slug: "plugins/zotlit";
  body: string;
  collection: "docs";
  data: InferEntrySchema<"docs">
} & { render(): Render[".mdx"] };
};

	};

	type DataEntryMap = {
		
	};

	type AnyEntryMap = ContentEntryMap & DataEntryMap;

	type ContentConfig = typeof import("../src/content/config");
}
