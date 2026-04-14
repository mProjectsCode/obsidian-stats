# Extra Analysis Plan (Main.js + Repo)

## Goal
Split extra analysis into small parts that are easy to build, test, and extend.

We want one simple pipeline that can run:

1. Main.js analysis from release artifacts
2. Repo analysis from cloned plugin repos

The first batch of new checks is:

1. Is main.js probably minified?
2. Are there large base64 blobs?
3. Is Worker used?
4. Is WebAssembly used?

## Why Change
Current logic is spread across a few files and is partly tied to ES inference only.

A shared pipeline will make it easier to:

- add new checks without touching unrelated code
- keep errors local (one failed check should not fail all extraction)
- test each check on its own
- keep naming and output format consistent

## Simple Naming
Use short names:

- `analysis/` for shared pipeline code
- `analysis/mainjs/` for main.js checks
- `analysis/repo/` for cloned repo checks
- `check` for one small rule
- `result` for outputs

## Proposed File Layout
Under `data/src/plugins/extra/`:

- `analysis/mod.rs`
- `analysis/pipeline.rs`
- `analysis/types.rs`
- `analysis/mainjs/mod.rs`
- `analysis/mainjs/check_es.rs`
- `analysis/mainjs/check_minified.rs`
- `analysis/mainjs/check_base64.rs`
- `analysis/mainjs/check_worker.rs`
- `analysis/mainjs/check_wasm.rs`
- `analysis/repo/mod.rs`
- `analysis/repo/check_manifest.rs`
- `analysis/repo/check_package.rs`
- `analysis/repo/check_license.rs`
- `analysis/repo/check_files.rs`
- `analysis/repo/check_i18n.rs`

Existing files can call this new pipeline first, then we can slowly move old code over.

## Shared Pipeline Plan

### Pipeline Input
A single plugin run gets a context object with:

- plugin id
- plugin metadata entry
- repo path (if available)
- release main.js source (if available)
- release state info (if available)

### Pipeline Stages

1. Build context
2. Run main.js checks
3. Run repo checks
4. Merge results
5. Write to `PluginRepoData`
6. Update run stats

Each check returns either:

- success with partial fields
- or a local error message (logged, but non-fatal)

### Shared Types
`analysis/types.rs` should define:

- `AnalysisContext`
- `MainJsResult`
- `RepoResult`
- `AnalysisResult` (merged)
- `AnalysisError` (check name + message)

## Main.js Checks (First Set)

### check_es
Move current ES inference logic here.

### check_minified
Use a score from simple signals:

- long lines
- low newline ratio
- high punctuation vs whitespace
- many tiny local names

Output:

- `is_probably_minified`
- `minification_score`

### check_base64
Find large base64-like strings.

Rules:

- minimum length (start with 1024)
- valid charset/padding
- optional decode for top N largest

Output:

- `large_base64_blob_count`
- `largest_base64_blob_length`
- optional guessed type list (wasm/image/unknown)

### check_worker
Detect:

- `new Worker(...)`
- `new SharedWorker(...)`
- service worker usage markers

Output:

- worker usage count fields

### check_wasm
Detect:

- `WebAssembly.instantiate(...)`
- `WebAssembly.instantiateStreaming(...)`
- `WebAssembly.compile(...)`
- `new WebAssembly.Module(...)`

Output:

- wasm usage count fields

## Repo Checks (Compartmentalized)
Split current repo extraction into small checks too.

### check_manifest
Reads and parses `manifest.json`.

### check_package
Reads `package.json` and extracts:

- dependencies
- devDependencies
- package manager hints
- bundlers
- test frameworks

### check_license
Finds and compares license data from:

- package field
- license file content

### check_files
Handles file-system based signals:

- file type counts
- beta manifest presence
- test files
- LOC data

### check_i18n
Handles i18n signals:

- i18n dependencies
- locale/i18n file presence

## Unified Output Mapping
Map both main.js and repo check outputs into `PluginRepoData`.

Keep old fields working. Add optional fields for new main.js signals.

Suggested new fields (simple flat shape):

- `main_js_is_probably_minified: Option<bool>`
- `main_js_minification_score: Option<f32>`
- `main_js_large_base64_blob_count: Option<u32>`
- `main_js_largest_base64_blob_length: Option<u32>`
- `main_js_worker_usage_count: Option<u32>`
- `main_js_webassembly_usage_count: Option<u32>`

## Rollout Steps

### Step 0 - Skeleton
Create `analysis/` modules and shared types.

### Step 1 - Main.js Move
Move ES logic into `analysis/mainjs/check_es.rs` and keep old wrapper.

### Step 2 - New Main.js Checks
Add minified/base64/worker/wasm checks.

### Step 3 - Repo Split
Refactor current repo extraction into the repo check files.

### Step 4 - Shared Pipeline Wiring
Call one pipeline from extra extraction, then map results.

### Step 5 - Clean Up
Remove temporary wrappers after parity is confirmed.

## Testing Plan

### Unit Tests

- each main.js check in isolation
- each repo check in isolation
- score and threshold tests for minified/base64

### Integration Tests

- full plugin extra extraction with cached samples
- parse failure should not fail the full plugin
- missing repo or missing main.js should produce partial results, not crash

### Parity Checks

- compare ES inference old vs new on sample set
- compare existing repo fields before/after refactor

## Performance Rules

- parse main.js once
- keep heavy work capped (top N decode attempts)
- never store raw base64 blobs in output
- keep per-check errors local and non-fatal

## Open Questions

1. Should minification score be shown in the website or only stored for now?
2. Do we want separate counts for Worker and Service Worker?
3. Should guessed blob types be stored now or added later?
4. Do we want one combined analysis log file per run?

## Checklist

- [ ] Create shared `analysis/` pipeline types
- [ ] Move ES check into new main.js check file
- [ ] Add minified/base64/worker/wasm checks
- [ ] Split repo extraction into small repo checks
- [ ] Wire shared pipeline into extra extraction
- [ ] Add new optional fields to `PluginRepoData`
- [ ] Add run stats per check
- [ ] Add unit + integration tests
- [ ] Remove temporary compatibility wrappers
