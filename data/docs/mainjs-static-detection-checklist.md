# Main.js Static Detection Checklist

Useful signals we can detect from a released Obsidian plugin `main.js` bundle without cloning the repo or executing code.

These checks should be treated as static heuristics, not proof of runtime behavior. Prefer AST-based detections where possible, and use string scanning for bundled/minified code patterns that are hard to recover from the AST. Store counts and confidence where a boolean would hide uncertainty.

## Bundle Shape And Compatibility

- [ ] Estimated ES target version from syntax features.
- [ ] Parse success/failure with SWC, including whether tolerant parsing was required.
- [ ] Probable minification and minification score.
- [ ] Source map comment presence.
- [ ] Inline source map presence.
- [ ] Large base64-like blob count and largest blob length.
- [ ] Guessed embedded blob types, such as wasm, image, font, zip, or unknown.
- [ ] WebAssembly usage via `WebAssembly.*`, `.wasm` strings, or wasm-like base64.
- [ ] Worker usage via `new Worker`, `new SharedWorker`, worker blob URLs, or worker-like bundled strings.
- [ ] Dynamic import usage via `import(...)`.
- [ ] Common bundler fingerprints, such as esbuild, Rollup, Webpack, Vite, Parcel, Browserify, SWC, or Terser wrappers.
- [ ] Module system fingerprints, such as CommonJS `require`, AMD `define`, UMD wrappers, or ESM import/export.
- [ ] Bundle size buckets and line count buckets.
- [ ] Repeated embedded dependency names or license banners that reveal bundled libraries.
- [ ] Optional chaining/nullish coalescing/private fields/top-level await feature flags for more granular compatibility output.

## Network And Remote Data

- [ ] Browser `fetch` usage.
- [ ] Obsidian `requestUrl` usage.
- [ ] Legacy Obsidian `request` usage.
- [ ] `XMLHttpRequest` usage.
- [ ] Node `http` or `https` module usage.
- [ ] WebSocket or EventSource usage.
- [ ] `navigator.sendBeacon` usage.
- [ ] Remote image/script/style loading through DOM-created elements.
- [ ] URL construction through `new URL`, `URLSearchParams`, or string literals that look like `http://` or `https://`.
- [ ] Known API hostnames in string literals, grouped by domain.
- [ ] Localhost/private-network calls, such as `localhost`, `127.0.0.1`, `0.0.0.0`, `.local`, or RFC1918 IPs.
- [ ] Cloud AI provider endpoints or SDK fingerprints, such as OpenAI, Anthropic, Google Gemini, Ollama, OpenRouter, Replicate, or Hugging Face.
- [ ] Sync/storage provider endpoints or SDK fingerprints, such as GitHub, GitLab, Dropbox, Google Drive, OneDrive, S3, Supabase, Firebase, Notion, Airtable, Todoist, Telegram, Discord, Slack, or Mastodon.
- [ ] Hard-coded API keys, bearer tokens, webhook URLs, or client secrets.
- [ ] User-agent/header manipulation.
- [ ] Retry, polling, interval, or background sync patterns around network calls.
- [ ] Remote telemetry/analytics SDK fingerprints, such as Sentry, PostHog, Plausible, Google Analytics, Mixpanel, Segment, Amplitude, or Datadog.

## Filesystem And Process Access

- [ ] Node `fs` or `fs/promises` usage.
- [ ] Node `path`, `os`, `crypto`, `stream`, `buffer`, or `zlib` usage.
- [ ] Node `child_process` usage.
- [ ] Electron `shell`, `remote`, `clipboard`, `ipcRenderer`, or `nativeImage` usage.
- [ ] Direct filesystem access outside Obsidian's vault API.
- [ ] Absolute path strings, home-directory markers, Windows drive paths, or `/tmp` usage.
- [ ] File dialog usage through Electron or DOM file inputs.
- [ ] Clipboard reads/writes through DOM, Electron, or Obsidian APIs.
- [ ] Persistent browser storage usage, such as `localStorage`, `sessionStorage`, `indexedDB`, or Cache API.
- [ ] Compression/archive handling, such as JSZip, tar, gzip, or zlib.
- [ ] Cryptography/hash usage through Web Crypto, Node crypto, or common crypto libraries.

## Obsidian Vault API

- [ ] `this.app.vault` access.
- [ ] Vault file reads through `read`, `cachedRead`, or `readBinary`.
- [ ] Vault file writes through `create`, `createBinary`, `modify`, `modifyBinary`, `append`, `rename`, `delete`, `trash`, or `copy`.
- [ ] Folder operations through `createFolder`, `getFolderByPath`, or folder traversal.
- [ ] Attachment/resource access through `getResourcePath`, `adapter.getResourcePath`, or embedded file handling.
- [ ] Adapter-level access through `app.vault.adapter`.
- [ ] Adapter reads/writes through `read`, `write`, `append`, `mkdir`, `rmdir`, `remove`, `rename`, `copy`, `exists`, `list`, or `stat`.
- [ ] Direct `.obsidian` config file access through vault APIs.
- [ ] Full-vault enumeration through `getFiles`, `getMarkdownFiles`, `getAllLoadedFiles`, `getRoot`, or recursive folder traversal.
- [ ] Active file access through `workspace.getActiveFile` followed by vault reads/writes.
- [ ] File open/create flows through `workspace.openLinkText`, `workspace.getLeaf().openFile`, or file manager APIs.
- [ ] Obsidian URI generation or handling through `obsidian://` string literals.

## Obsidian Metadata And Search

- [ ] `this.app.metadataCache` access.
- [ ] File metadata reads through `getFileCache`, `getCache`, `getFirstLinkpathDest`, `resolvedLinks`, or `unresolvedLinks`.
- [ ] Full metadata traversal through cache maps or repeated `getFileCache` over all files.
- [ ] Frontmatter access through `frontmatter`, `getFileCache(file).frontmatter`, or `processFrontMatter`.
- [ ] Tag/link/embed/block extraction through metadata cache fields.
- [ ] Metadata event listeners, such as `metadataCache.on("changed")`, `on("deleted")`, or `on("resolved")`.
- [ ] Search or query indexing patterns.
- [ ] Dataview/DataCore plugin API usage or fallback detection by plugin ID.

## Obsidian Workspace And UI

- [ ] `this.app.workspace` access.
- [ ] Custom view registration through `registerView`.
- [ ] Leaves/panes manipulation through `getLeaf`, `getLeavesOfType`, `detachLeavesOfType`, `setViewState`, or `revealLeaf`.
- [ ] Active editor access through `workspace.activeEditor`, `MarkdownView`, or editor references.
- [ ] Ribbon icon registration through `addRibbonIcon`.
- [ ] Status bar usage through `addStatusBarItem`.
- [ ] Command registration through `addCommand`.
- [ ] Editor command/callback usage.
- [ ] Context menu registration through `registerEvent` on menu events.
- [ ] Modal, Notice, SuggestModal, FuzzySuggestModal, or setting UI usage.
- [ ] Declarative settings usage through `getSettingDefinitions`.
- [ ] Imperative settings tab usage through `PluginSettingTab` and `display`.
- [ ] Workspace/layout persistence manipulation.
- [ ] DOM-heavy UI manipulation through `createEl`, direct `document` access, MutationObserver, or custom CSS injection.

## Obsidian Editor And Markdown Integration

- [ ] CodeMirror extension registration through `registerEditorExtension`.
- [ ] Markdown post processor registration through `registerMarkdownPostProcessor`.
- [ ] Markdown code block processor registration through `registerMarkdownCodeBlockProcessor`.
- [ ] Markdown view/editor API usage, such as `Editor`, `MarkdownView`, `editor.getValue`, `replaceRange`, or `transaction`.
- [ ] Decorations, gutters, facets, state fields, or view plugins from CodeMirror 6.
- [ ] EditorSuggest registration.
- [ ] File menu, editor menu, or command palette integrations.
- [ ] Markdown renderer usage through `MarkdownRenderer.render`.
- [ ] Link parsing/generation helpers, such as `parseLinktext`, `normalizePath`, or `getLinkpath`.

## Obsidian Plugin Lifecycle And Settings

- [ ] Plugin lifecycle method names: `onload`, `onunload`, `loadData`, `saveData`.
- [ ] Settings persistence via `loadData` and `saveData`.
- [ ] High-frequency save patterns, such as `saveData` inside loops, key handlers, or intervals.
- [ ] Registered intervals through `registerInterval` or `setInterval`.
- [ ] Registered DOM events through `registerDomEvent`.
- [ ] Registered Obsidian events through `registerEvent`.
- [ ] Monkey-patching or `around`-style wrapping of Obsidian methods.
- [ ] Access to `app.plugins`, `enabledPlugins`, plugin manifests, or other plugin instances.
- [ ] Dependency on specific core/community plugin IDs in string literals.
- [ ] Mobile/desktop branching via `Platform.isMobile`, `Platform.isDesktop`, `Platform.isMacOS`, etc.

## Security And Privacy Risk Signals

- [ ] Combines full-vault enumeration with network transmission.
- [ ] Combines metadata/frontmatter reads with network transmission.
- [ ] Combines adapter-level filesystem access with network transmission.
- [ ] Uses `child_process` and network APIs.
- [ ] Uses Electron IPC or shell APIs.
- [ ] Uses `eval`, `new Function`, dynamic script injection, or remote code loading.
- [ ] Uses obfuscated string tables, packed code, or runtime string decoding.
- [ ] References credential-like setting names, such as `apiKey`, `token`, `secret`, `password`, or `webhook`.
- [ ] Logs note contents, frontmatter, paths, or settings to console.
- [ ] Registers broad keyboard handlers or clipboard hooks.
- [ ] Reads browser/device environment data, such as user agent, platform, language, timezone, screen size, geolocation, camera, microphone, or notifications.
- [ ] Permission-sensitive browser API usage, such as geolocation, media devices, notifications, WebRTC, or Bluetooth.

## Performance And Reliability Signals

- [ ] `setInterval`, `setTimeout`, `requestAnimationFrame`, or polling usage.
- [ ] Interval registration without clear plugin cleanup patterns.
- [ ] Full-vault scans during `onload`.
- [ ] Full-vault scans inside event listeners.
- [ ] Large synchronous loops over file lists.
- [ ] Large JSON parsing/stringifying.
- [ ] Large regex literals or repeated regex construction.
- [ ] DOM MutationObserver usage.
- [ ] File watcher-like behavior through repeated vault/metadata events.
- [ ] Expensive operations in editor change handlers.
- [ ] Memory-heavy caches keyed by path or file content.
- [ ] Use of workers or wasm as potential positive performance signals.

## User-Facing Disclosure Signals

These should answer questions a user might reasonably ask while evaluating a plugin. The goal is not to label the whole plugin category, but to disclose observed capabilities and surprising combinations.

- [ ] Does this plugin make network requests?
- [ ] Does this plugin use Obsidian's CORS-free request APIs?
- [ ] Does this plugin talk to known third-party services?
- [ ] Does this plugin talk to localhost or private-network addresses?
- [ ] Does this plugin include telemetry or error-reporting libraries?
- [ ] Does this plugin access note contents?
- [ ] Does this plugin access frontmatter, tags, links, embeds, or other metadata?
- [ ] Does this plugin enumerate the full vault or all markdown files?
- [ ] Does this plugin write, rename, delete, trash, or copy vault files?
- [ ] Does this plugin access `.obsidian` config paths?
- [ ] Does this plugin use adapter-level file access instead of only high-level vault APIs?
- [ ] Does this plugin use direct Node filesystem APIs?
- [ ] Does this plugin launch subprocesses or use Electron shell/IPC APIs?
- [ ] Does this plugin read or write the clipboard?
- [ ] Does this plugin inject scripts, evaluate dynamic code, or load remote code?
- [ ] Does this plugin register editor extensions or alter editor behavior?
- [ ] Does this plugin register markdown post-processors or code block processors?
- [ ] Does this plugin manipulate workspace panes, views, or layout state?
- [ ] Does this plugin register global-ish event handlers, timers, or polling loops?
- [ ] Does this plugin access other plugins or core plugin internals?
- [ ] Does this plugin depend on mobile/desktop-specific platform branches?
- [ ] Does this plugin combine vault reads with network requests?
- [ ] Does this plugin combine full-vault enumeration with network requests?
- [ ] Does this plugin combine metadata/frontmatter reads with network requests?
- [ ] Does this plugin combine direct filesystem/process access with network requests?
- [ ] Does this plugin handle credential-like settings such as API keys, tokens, secrets, passwords, or webhooks?
- [ ] Does this plugin have signals that its behavior may be expensive on large vaults?

## Declarative Obsidian API Classification

Create a small Rust rule catalog that maps observed `main.js` symbols to stable capability groups. The catalog should be declarative in usage, but implemented as typed Rust builders instead of external YAML/JSON. This keeps rule definitions easy to scan while preserving compile-time checks, IDE completion, and refactor safety.

Example shape:

```rust
fn obsidian_api_rules() -> Vec<ApiRule> {
    vec![
        ApiRule::new("vault.read")
            .label("Reads vault files")
            .category(ApiCategory::Vault)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::High)
            .member_calls([
                "app.vault.read",
                "app.vault.cachedRead",
                "app.vault.readBinary",
            ])
            .implies(["disclosure.note_content_access"])
            .build(),

        ApiRule::new("vault.enumerate")
            .label("Enumerates vault files")
            .category(ApiCategory::Vault)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::High)
            .member_calls([
                "app.vault.getFiles",
                "app.vault.getMarkdownFiles",
                "app.vault.getAllLoadedFiles",
            ])
            .implies(["disclosure.full_vault_access"])
            .build(),

        ApiRule::new("network.obsidian_request")
            .label("Uses Obsidian request API")
            .category(ApiCategory::Network)
            .severity(ApiSeverity::Notice)
            .confidence(Confidence::High)
            .calls(["request", "requestUrl"])
            .implies([
                "disclosure.network_access",
                "disclosure.cors_free_network_access",
            ])
            .build(),

        ApiRule::new("correlation.vault_read_plus_network")
            .label("Reads vault data and uses network")
            .category(ApiCategory::Correlation)
            .severity(ApiSeverity::Warning)
            .confidence(Confidence::Medium)
            .when_all([
                "disclosure.note_content_access",
                "disclosure.network_access",
            ])
            .build(),
    ]
}
```

Suggested Rust types:

- [ ] `ApiRule`: finished immutable rule consumed by the analyzer.
- [ ] `ApiRuleBuilder`: fluent builder returned by `ApiRule::new(id)`.
- [ ] `ApiCategory`: enum for `Network`, `Vault`, `Metadata`, `Workspace`, `Editor`, `Ui`, `Settings`, `Lifecycle`, `Filesystem`, `Electron`, `Browser`, `Dependency`, `DynamicCode`, and `Correlation`.
- [ ] `ApiSeverity`: enum for `Info`, `Notice`, `Warning`, and `Critical`.
- [ ] `Confidence`: enum for `High`, `Medium`, and `Low`.
- [ ] `ApiMatcher`: typed matcher collection with calls, member calls, member reads, imports, string literals, classes, and constructor calls.
- [ ] `ApiCapability`: emitted low-level capability match, such as `vault.read`.
- [ ] `Disclosure`: emitted user-facing disclosure, such as `disclosure.note_content_access`.
- [ ] `ApiEvidence`: symbolic evidence record with match kind, normalized symbol, count, and optional capped source location.

Suggested builder methods:

- [ ] `ApiRule::new(id)`: starts a rule with a stable machine-readable identifier.
- [ ] `.label(text)`: short user-facing capability text.
- [ ] `.category(category)`: typed capability category.
- [ ] `.severity(severity)`: display/risk weight.
- [ ] `.confidence(confidence)`: default confidence for matches from this rule.
- [ ] `.calls(items)`: direct function names, such as `fetch`, `requestUrl`, or `eval`.
- [ ] `.member_calls(items)`: normalized member call chains, such as `app.vault.read`.
- [ ] `.member_reads(items)`: normalized property reads, such as `app.metadataCache.resolvedLinks`.
- [ ] `.imports(items)`: module imports or requires, such as `obsidian`, `electron`, `fs`, or `child_process`.
- [ ] `.string_literals(items)`: string markers, such as `obsidian://`, `.obsidian/`, or known service hostnames.
- [ ] `.classes(items)`: referenced or constructed classes, such as `Notice`, `Modal`, `PluginSettingTab`, or `MarkdownView`.
- [ ] `.constructors(items)`: constructor call matches, such as `new Worker`, `new Function`, or `new WebSocket`.
- [ ] `.implies(disclosures)`: disclosure IDs emitted when the primitive rule matches.
- [ ] `.when_all(capabilities_or_disclosures)`: correlation dependencies that must all be present.
- [ ] `.when_any(capabilities_or_disclosures)`: correlation dependencies where any one is enough.
- [ ] `.evidence_limit(limit)`: maximum number of symbolic evidence items to keep.
- [ ] `.build()`: validates required fields and returns `ApiRule`.

Builder validation rules:

- [ ] Require non-empty `id`, `label`, `category`, `severity`, and `confidence`.
- [ ] Require either at least one primitive matcher or at least one correlation condition.
- [ ] Reject rules that mix primitive matchers and correlation conditions unless explicitly allowed.
- [ ] Reject duplicate IDs at catalog construction time.
- [ ] Reject unknown disclosure IDs in `.implies`, `.when_all`, or `.when_any`.
- [ ] Keep all rule IDs and disclosure IDs stable once exposed in generated data.

Suggested initial API capability groups:

- [ ] `network.browser`: `fetch`, `XMLHttpRequest`, `WebSocket`, `EventSource`, `sendBeacon`.
- [ ] `network.obsidian`: `request`, `requestUrl`.
- [ ] `vault.read`: `read`, `cachedRead`, `readBinary`.
- [ ] `vault.write`: `create`, `createBinary`, `modify`, `modifyBinary`, `append`.
- [ ] `vault.destructive`: `delete`, `trash`, `rename`, `copy`.
- [ ] `vault.enumerate`: `getFiles`, `getMarkdownFiles`, `getAllLoadedFiles`, `getRoot`.
- [ ] `vault.adapter`: `app.vault.adapter.*`.
- [ ] `metadata.read`: `getFileCache`, `getCache`, `resolvedLinks`, `unresolvedLinks`, `getFirstLinkpathDest`.
- [ ] `metadata.frontmatter`: `frontmatter`, `processFrontMatter`.
- [ ] `workspace.views`: `registerView`, `getLeaf`, `getLeavesOfType`, `setViewState`, `detachLeavesOfType`.
- [ ] `workspace.active_file`: `getActiveFile`, `activeEditor`.
- [ ] `editor.extension`: `registerEditorExtension`, CodeMirror extension classes/functions.
- [ ] `markdown.processing`: `registerMarkdownPostProcessor`, `registerMarkdownCodeBlockProcessor`, `MarkdownRenderer.render`.
- [ ] `ui.commands`: `addCommand`, `addRibbonIcon`, `addStatusBarItem`.
- [ ] `ui.modals_notices`: `Modal`, `Notice`, `SuggestModal`, `FuzzySuggestModal`.
- [ ] `settings.persistence`: `loadData`, `saveData`.
- [ ] `settings.ui`: `addSettingTab`, `PluginSettingTab`, `getSettingDefinitions`, `Setting`.
- [ ] `lifecycle.events`: `registerEvent`, `registerDomEvent`, `registerInterval`.
- [ ] `plugins.internal_access`: `app.plugins`, `enabledPlugins`, `getPlugin`, `manifests`.
- [ ] `platform.branching`: `Platform.isMobile`, `Platform.isDesktop`, `Platform.isMacOS`, `Platform.isWin`, `Platform.isLinux`.
- [ ] `filesystem.node`: `fs`, `fs/promises`, `path`, `os`.
- [ ] `process.node`: `child_process`, `process.env`, `process.platform`.
- [ ] `electron.desktop`: `electron`, `shell`, `ipcRenderer`, `clipboard`, `remote`.
- [ ] `browser.storage`: `localStorage`, `sessionStorage`, `indexedDB`, Cache API.
- [ ] `browser.permissions`: geolocation, media devices, notifications, WebRTC, Bluetooth.
- [ ] `dynamic_code`: `eval`, `new Function`, dynamic script elements, remote script URLs.

## Implementation Notes

- [ ] Parse `main.js` once and share the AST between checks.
- [ ] Keep raw source scanning available for minified bundles and string-literal detection.
- [ ] Normalize member chains so detections work for `this.app.vault.read`, `app.vault.read`, destructured aliases, and optional chaining where possible.
- [ ] Track import/require aliases for `obsidian`, `electron`, Node modules, and major SDKs.
- [ ] Run primitive API rules first, then run correlation/disclosure rules on the emitted capabilities.
- [ ] Record evidence snippets as symbolic matches, not raw note/user data.
- [ ] Cap expensive scans, base64 decoding, and string-literal extraction.
- [ ] Prefer count fields plus confidence over hard booleans for ambiguous signals.
- [ ] Keep each check independently non-fatal.
- [ ] Add tests with minified and non-minified fixtures.
- [ ] Add correlation checks separately from primitive detections, such as "vault read plus network".
