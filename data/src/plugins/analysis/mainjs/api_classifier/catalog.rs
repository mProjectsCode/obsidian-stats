use std::{collections::BTreeSet, sync::OnceLock};

use super::custom_matchers;
use super::rule::{ApiCatalogError, ApiCategory, ApiRule, ApiSeverity, Confidence};

pub(in crate::plugins::analysis::mainjs) fn obsidian_api_rules() -> &'static [ApiRule] {
    static RULES: OnceLock<Vec<ApiRule>> = OnceLock::new();

    RULES.get_or_init(build_obsidian_api_rules)
}

fn build_obsidian_api_rules() -> Vec<ApiRule> {
    let rules = vec![
        ApiRule::builder("network.browser")
            .label("Uses browser network APIs")
            .category(ApiCategory::Network)
            .severity(ApiSeverity::Notice)
            .confidence(Confidence::High)
            .global_calls(["fetch"])
            .rooted_member_calls(["navigator.sendBeacon"])
            .constructors(["XMLHttpRequest", "WebSocket", "EventSource"])
            .implies(["disclosure.network_access"])
            .build(),
        ApiRule::builder("network.obsidian")
            .label("Uses Obsidian request APIs")
            .category(ApiCategory::Network)
            .severity(ApiSeverity::Notice)
            .confidence(Confidence::High)
            .module_calls("obsidian", ["request", "requestUrl"])
            .module_member_calls("obsidian", ["request", "requestUrl"])
            .implies([
                "disclosure.network_access",
                "disclosure.cors_free_network_access",
            ])
            .build(),
        ApiRule::builder("network.node")
            .label("Uses Node HTTP modules")
            .category(ApiCategory::Network)
            .severity(ApiSeverity::Notice)
            .confidence(Confidence::High)
            .imports(["http", "https", "node:http", "node:https"])
            .implies(["disclosure.network_access"])
            .build(),
        ApiRule::builder("network.url_construction")
            .label("Builds URLs or URL parameters")
            .category(ApiCategory::Network)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::High)
            .constructors(["URL", "URLSearchParams"])
            .string_literals(["http://", "https://"])
            .build(),
        ApiRule::builder("network.private")
            .label("References localhost or private-network addresses")
            .category(ApiCategory::Network)
            .severity(ApiSeverity::Warning)
            .confidence(Confidence::Medium)
            .string_literals([
                "localhost",
                "127.0.0.1",
                "0.0.0.0",
                "http://192.168.",
                "https://192.168.",
                "http://10.",
                "https://10.",
                "http://172.16.",
                "https://172.16.",
                "http://172.17.",
                "https://172.17.",
                "http://172.18.",
                "https://172.18.",
                "http://172.19.",
                "https://172.19.",
                "http://172.20.",
                "https://172.20.",
                "http://172.21.",
                "https://172.21.",
                "http://172.22.",
                "https://172.22.",
                "http://172.23.",
                "https://172.23.",
                "http://172.24.",
                "https://172.24.",
                "http://172.25.",
                "https://172.25.",
                "http://172.26.",
                "https://172.26.",
                "http://172.27.",
                "https://172.27.",
                "http://172.28.",
                "https://172.28.",
                "http://172.29.",
                "https://172.29.",
                "http://172.30.",
                "https://172.30.",
                "http://172.31.",
                "https://172.31.",
            ])
            .implies(["disclosure.private_network_access"])
            .build(),
        ApiRule::builder("network.ai_provider")
            .label("References AI provider endpoints or SDKs")
            .category(ApiCategory::Network)
            .severity(ApiSeverity::Notice)
            .confidence(Confidence::Medium)
            .imports([
                "openai",
                "@anthropic-ai/sdk",
                "@google/generative-ai",
                "@google/genai",
                "ollama",
                "replicate",
                "@huggingface/inference",
            ])
            .string_literals([
                "api.openai.com",
                "anthropic.com",
                "generativelanguage.googleapis.com",
                "openrouter.ai",
                "replicate.com",
                "huggingface.co",
                "localhost:11434",
            ])
            .implies([
                "disclosure.network_access",
                "disclosure.third_party_services",
            ])
            .build(),
        ApiRule::builder("network.sync_storage_provider")
            .label("References sync or storage provider endpoints or SDKs")
            .category(ApiCategory::Network)
            .severity(ApiSeverity::Notice)
            .confidence(Confidence::Medium)
            .imports([
                "@supabase/supabase-js",
                "firebase",
                "firebase-admin",
                "dropbox",
                "@notionhq/client",
                "aws-sdk",
                "@aws-sdk/client-s3",
            ])
            .string_literals([
                "api.github.com",
                "gitlab.com",
                "dropboxapi.com",
                "googleapis.com/drive",
                "graph.microsoft.com",
                "amazonaws.com",
                "supabase.co",
                "firebaseio.com",
                "firestore.googleapis.com",
                "api.notion.com",
                "api.airtable.com",
                "api.todoist.com",
                "api.telegram.org",
                "discord.com/api",
                "hooks.slack.com",
            ])
            .implies([
                "disclosure.network_access",
                "disclosure.third_party_services",
            ])
            .build(),
        ApiRule::builder("network.telemetry")
            .label("References telemetry or analytics SDKs")
            .category(ApiCategory::Network)
            .severity(ApiSeverity::Notice)
            .confidence(Confidence::Medium)
            .imports([
                "@sentry/browser",
                "@sentry/node",
                "posthog-js",
                "mixpanel-browser",
                "analytics",
                "@segment/analytics-node",
                "@datadog/browser-rum",
            ])
            .string_literals([
                "sentry.io",
                "app.posthog.com",
                "us.i.posthog.com",
                "eu.i.posthog.com",
                "plausible.io",
                "google-analytics.com",
                "googletagmanager.com",
                "mixpanel.com",
                "segment.com",
                "amplitude.com",
                "datadoghq.com",
            ])
            .implies([
                "disclosure.network_access",
                "disclosure.telemetry_or_error_reporting",
            ])
            .build(),
        ApiRule::builder("network.headers")
            .label("Manipulates user-agent or request headers")
            .category(ApiCategory::Network)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::Medium)
            .string_literals(["User-Agent", "user-agent", "Authorization"])
            .build(),
        ApiRule::builder("network.remote_dom_loading")
            .label("Loads remote image, script, or style elements")
            .category(ApiCategory::Network)
            .severity(ApiSeverity::Warning)
            .confidence(Confidence::Medium)
            .custom_ast("remote_dom_loading", custom_matchers::remote_dom_loading)
            .implies(["disclosure.network_access"])
            .build(),
        ApiRule::builder("vault.access")
            .label("Accesses Obsidian vault APIs")
            .category(ApiCategory::Vault)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::High)
            .member_reads(["this.app.vault", "app.vault"])
            .build(),
        ApiRule::builder("vault.read")
            .label("Reads vault files")
            .category(ApiCategory::Vault)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::High)
            .rooted_member_calls([
                "this.app.vault.read",
                "this.app.vault.cachedRead",
                "this.app.vault.readBinary",
                "app.vault.read",
                "app.vault.cachedRead",
                "app.vault.readBinary",
            ])
            .implies(["disclosure.note_content_access"])
            .build(),
        ApiRule::builder("vault.write")
            .label("Writes vault files")
            .category(ApiCategory::Vault)
            .severity(ApiSeverity::Notice)
            .confidence(Confidence::High)
            .rooted_member_calls([
                "this.app.vault.create",
                "this.app.vault.createBinary",
                "this.app.vault.modify",
                "this.app.vault.modifyBinary",
                "this.app.vault.append",
                "app.vault.create",
                "app.vault.createBinary",
                "app.vault.modify",
                "app.vault.modifyBinary",
                "app.vault.append",
            ])
            .implies(["disclosure.vault_file_write"])
            .build(),
        ApiRule::builder("vault.destructive")
            .label("Renames, deletes, trashes, or copies vault files")
            .category(ApiCategory::Vault)
            .severity(ApiSeverity::Warning)
            .confidence(Confidence::High)
            .rooted_member_calls([
                "this.app.vault.delete",
                "this.app.vault.trash",
                "this.app.vault.rename",
                "this.app.vault.copy",
                "app.vault.delete",
                "app.vault.trash",
                "app.vault.rename",
                "app.vault.copy",
            ])
            .implies(["disclosure.vault_file_write"])
            .build(),
        ApiRule::builder("vault.enumerate")
            .label("Enumerates vault files")
            .category(ApiCategory::Vault)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::High)
            .rooted_member_calls([
                "this.app.vault.getFiles",
                "this.app.vault.getMarkdownFiles",
                "this.app.vault.getAllLoadedFiles",
                "app.vault.getFiles",
                "app.vault.getMarkdownFiles",
                "app.vault.getAllLoadedFiles",
            ])
            .implies(["disclosure.full_vault_access"])
            .build(),
        ApiRule::builder("vault.folder_ops")
            .label("Uses vault folder APIs")
            .category(ApiCategory::Vault)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::High)
            .rooted_member_calls([
                "this.app.vault.createFolder",
                "this.app.vault.getFolderByPath",
                "this.app.vault.getRoot",
                "app.vault.createFolder",
                "app.vault.getFolderByPath",
                "app.vault.getRoot",
            ])
            .build(),
        ApiRule::builder("vault.resources")
            .label("Accesses attachment resource paths")
            .category(ApiCategory::Vault)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::High)
            .rooted_member_calls([
                "this.app.vault.getResourcePath",
                "this.app.vault.adapter.getResourcePath",
                "app.vault.getResourcePath",
                "app.vault.adapter.getResourcePath",
            ])
            .build(),
        ApiRule::builder("vault.adapter")
            .label("Uses adapter-level vault filesystem APIs")
            .category(ApiCategory::Vault)
            .severity(ApiSeverity::Notice)
            .confidence(Confidence::High)
            .rooted_member_calls([
                "this.app.vault.adapter.read",
                "this.app.vault.adapter.write",
                "this.app.vault.adapter.append",
                "this.app.vault.adapter.mkdir",
                "this.app.vault.adapter.rmdir",
                "this.app.vault.adapter.remove",
                "this.app.vault.adapter.rename",
                "this.app.vault.adapter.copy",
                "this.app.vault.adapter.exists",
                "this.app.vault.adapter.list",
                "this.app.vault.adapter.stat",
                "app.vault.adapter.read",
                "app.vault.adapter.write",
                "app.vault.adapter.append",
                "app.vault.adapter.mkdir",
                "app.vault.adapter.rmdir",
                "app.vault.adapter.remove",
                "app.vault.adapter.rename",
                "app.vault.adapter.copy",
                "app.vault.adapter.exists",
                "app.vault.adapter.list",
                "app.vault.adapter.stat",
            ])
            .custom_ast("adapter_operation", custom_matchers::adapter_operation)
            .implies(["disclosure.adapter_file_access"])
            .build(),
        ApiRule::builder("vault.obsidian_config")
            .label("References .obsidian configuration paths")
            .category(ApiCategory::Vault)
            .severity(ApiSeverity::Notice)
            .confidence(Confidence::Medium)
            .string_literals([".obsidian/", ".obsidian\\"])
            .implies(["disclosure.obsidian_config_access"])
            .build(),
        ApiRule::builder("vault.uri")
            .label("References Obsidian URI links")
            .category(ApiCategory::Vault)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::High)
            .string_literals(["obsidian://"])
            .build(),
        ApiRule::builder("vault.open_create_flows")
            .label("Opens or creates files through workspace or file manager APIs")
            .category(ApiCategory::Vault)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::High)
            .rooted_member_calls([
                "this.app.workspace.openLinkText",
                "this.app.workspace.getLeaf.openFile",
                "this.app.fileManager.createNewMarkdownFile",
                "this.app.fileManager.renameFile",
                "app.workspace.openLinkText",
                "app.workspace.getLeaf.openFile",
                "app.fileManager.createNewMarkdownFile",
                "app.fileManager.renameFile",
            ])
            .build(),
        ApiRule::builder("metadata.read")
            .label("Reads Obsidian metadata cache")
            .category(ApiCategory::Metadata)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::High)
            .member_reads([
                "this.app.metadataCache",
                "app.metadataCache",
                "this.app.metadataCache.resolvedLinks",
                "this.app.metadataCache.unresolvedLinks",
                "app.metadataCache.resolvedLinks",
                "app.metadataCache.unresolvedLinks",
            ])
            .rooted_member_calls([
                "this.app.metadataCache.getFileCache",
                "this.app.metadataCache.getCache",
                "this.app.metadataCache.getFirstLinkpathDest",
                "app.metadataCache.getFileCache",
                "app.metadataCache.getCache",
                "app.metadataCache.getFirstLinkpathDest",
            ])
            .implies(["disclosure.metadata_access"])
            .build(),
        ApiRule::builder("metadata.frontmatter")
            .label("Reads or processes frontmatter")
            .category(ApiCategory::Metadata)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::Medium)
            .member_reads(["frontmatter"])
            .rooted_member_calls([
                "this.app.fileManager.processFrontMatter",
                "app.fileManager.processFrontMatter",
            ])
            .implies(["disclosure.metadata_access"])
            .build(),
        ApiRule::builder("metadata.events")
            .label("Registers metadata cache event listeners")
            .category(ApiCategory::Metadata)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::Medium)
            .rooted_member_calls(["this.app.metadataCache.on"])
            .arg_string(0, ["changed", "deleted", "resolved"])
            .rooted_member_calls(["app.metadataCache.on"])
            .arg_string(0, ["changed", "deleted", "resolved"])
            .build(),
        ApiRule::builder("metadata.traversal")
            .label("Traverses metadata cache maps or cached metadata for many files")
            .category(ApiCategory::Metadata)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::Medium)
            // REVIEW: `Object.keys/values/entries` are not bound to metadataCache maps. This rule can
            // fire when a plugin reads one metadataCache property and uses Object traversal elsewhere.
            .member_reads([
                "this.app.metadataCache.resolvedLinks",
                "this.app.metadataCache.unresolvedLinks",
                "this.app.metadataCache.fileCache",
                "app.metadataCache.resolvedLinks",
                "app.metadataCache.unresolvedLinks",
                "app.metadataCache.fileCache",
            ])
            .member_calls(["Object.entries", "Object.keys", "Object.values"])
            .build(),
        ApiRule::builder("metadata.extraction")
            .label("Extracts tags, links, embeds, blocks, or headings from metadata")
            .category(ApiCategory::Metadata)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::Medium)
            .custom_ast(
                "metadata_cache_extraction",
                custom_matchers::metadata_cache_extraction,
            )
            .implies(["disclosure.metadata_access"])
            .build(),
        ApiRule::builder("dependency.dataview")
            .label("References Dataview or DataCore plugin APIs")
            .category(ApiCategory::Dependency)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::Medium)
            .string_literals(["dataview", "dataviewapi", "data-core", "datacore"])
            .build(),
        ApiRule::builder("workspace.access")
            .label("Accesses Obsidian workspace APIs")
            .category(ApiCategory::Workspace)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::High)
            .member_reads(["this.app.workspace", "app.workspace"])
            .build(),
        ApiRule::builder("workspace.views")
            .label("Registers or manipulates workspace views and panes")
            .category(ApiCategory::Workspace)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::High)
            // REVIEW: `getLeaf` can be used for simple file opening, not only layout manipulation.
            // It still implies workspace layout because requesting leaves can create/split panes; a
            // future call-argument/state matcher could separate read-only lookup from mutation.
            .rooted_member_calls([
                "this.registerView",
                "this.app.workspace.getLeaf",
                "this.app.workspace.getLeavesOfType",
                "this.app.workspace.detachLeavesOfType",
                "this.app.workspace.setViewState",
                "this.app.workspace.revealLeaf",
                "app.workspace.getLeaf",
                "app.workspace.getLeavesOfType",
                "app.workspace.detachLeavesOfType",
                "app.workspace.setViewState",
                "app.workspace.revealLeaf",
            ])
            .implies(["disclosure.workspace_layout"])
            .build(),
        ApiRule::builder("workspace.active_file")
            .label("Accesses the active file or editor")
            .category(ApiCategory::Workspace)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::High)
            .member_reads([
                "this.app.workspace.activeEditor",
                "app.workspace.activeEditor",
            ])
            .rooted_member_calls([
                "this.app.workspace.getActiveFile",
                "app.workspace.getActiveFile",
            ])
            .build(),
        ApiRule::builder("workspace.editor_commands")
            .label("Registers editor callbacks, menus, or command palette integrations")
            .category(ApiCategory::Workspace)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::Medium)
            // REVIEW: this does not bind `editorCallback`, `file-menu`, or `editor-menu` to the
            // `addCommand`/`registerEvent` payload. Same-file unrelated menu strings can contribute
            // evidence.
            .member_calls(["this.addCommand", "this.registerEvent"])
            .string_literals([
                "editorCallback",
                "editorCheckCallback",
                "file-menu",
                "editor-menu",
                "menu",
            ])
            .build(),
        ApiRule::builder("workspace.layout_persistence")
            .label("Reads or writes workspace layout persistence")
            .category(ApiCategory::Workspace)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::Medium)
            .rooted_member_calls([
                "this.app.workspace.getLayout",
                "this.app.workspace.setLayout",
                "this.app.workspace.requestSaveLayout",
                "app.workspace.getLayout",
                "app.workspace.setLayout",
                "app.workspace.requestSaveLayout",
            ])
            .implies(["disclosure.workspace_layout"])
            .build(),
        ApiRule::builder("ui.commands")
            .label("Registers commands, ribbon icons, or status bar items")
            .category(ApiCategory::Ui)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::High)
            .member_calls([
                "this.addCommand",
                "this.addRibbonIcon",
                "this.addStatusBarItem",
            ])
            .build(),
        ApiRule::builder("ui.modals_notices")
            .label("Uses Obsidian modal or notice UI")
            .category(ApiCategory::Ui)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::Medium)
            .constructors([
                "Modal",
                "Notice",
                "SuggestModal",
                "FuzzySuggestModal",
                "obsidian.Modal",
                "obsidian.Notice",
                "obsidian.SuggestModal",
                "obsidian.FuzzySuggestModal",
            ])
            .classes(["Modal", "Notice", "SuggestModal", "FuzzySuggestModal"])
            .build(),
        ApiRule::builder("ui.dom_heavy")
            .label("Performs DOM-heavy UI manipulation")
            .category(ApiCategory::Ui)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::Medium)
            // REVIEW: a single `document.createElement` or `createEl` is ordinary Obsidian UI, not
            // necessarily DOM-heavy. This rule has no disclosure impact, but it can overstate the
            // capability until we can require repeated DOM operations or a denser evidence threshold.
            .member_calls([
                "document.createElement",
                "document.querySelector",
                "document.querySelectorAll",
                "document.body.appendChild",
            ])
            .calls(["createEl"])
            .constructors(["MutationObserver"])
            .string_literals(["style", "script"])
            .build(),
        ApiRule::builder("ui.file_dialog")
            .label("Uses file dialogs or DOM file inputs")
            .category(ApiCategory::Ui)
            .severity(ApiSeverity::Notice)
            .confidence(Confidence::Medium)
            .member_calls(["dialog.showOpenDialog", "dialog.showSaveDialog"])
            .custom_ast("dom_file_input", custom_matchers::dom_file_input)
            .build(),
        ApiRule::builder("editor.extension")
            .label("Registers editor extensions")
            .category(ApiCategory::Editor)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::High)
            .member_calls(["this.registerEditorExtension"])
            .implies(["disclosure.editor_behavior"])
            .build(),
        ApiRule::builder("editor.markdown_processing")
            .label("Registers markdown processors or renderers")
            .category(ApiCategory::Editor)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::High)
            .member_calls([
                "this.registerMarkdownPostProcessor",
                "this.registerMarkdownCodeBlockProcessor",
                "MarkdownRenderer.render",
                "obsidian.MarkdownRenderer.render",
            ])
            .implies(["disclosure.markdown_processing"])
            .build(),
        ApiRule::builder("editor.markdown_api")
            .label("Uses markdown view, editor, or link helper APIs")
            .category(ApiCategory::Editor)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::Medium)
            .member_calls([
                "editor.getValue",
                "editor.replaceRange",
                "editor.transaction",
            ])
            .classes(["MarkdownView", "Editor"])
            .module_calls(
                "obsidian",
                ["parseLinktext", "normalizePath", "getLinkpath"],
            )
            .module_member_calls(
                "obsidian",
                ["parseLinktext", "normalizePath", "getLinkpath"],
            )
            .build(),
        ApiRule::builder("editor.codemirror")
            .label("References CodeMirror extension primitives")
            .category(ApiCategory::Editor)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::Medium)
            .imports([
                "@codemirror/state",
                "@codemirror/view",
                "@codemirror/language",
                "@codemirror/commands",
            ])
            .string_literals(["Decoration", "StateField", "Facet", "ViewPlugin", "gutter"])
            .implies(["disclosure.editor_behavior"])
            .build(),
        ApiRule::builder("editor.suggest")
            .label("Registers editor suggestions")
            .category(ApiCategory::Editor)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::Medium)
            .member_calls(["this.registerEditorSuggest"])
            .classes(["EditorSuggest"])
            .implies(["disclosure.editor_behavior"])
            .build(),
        ApiRule::builder("settings.persistence")
            .label("Persists plugin settings or data")
            .category(ApiCategory::Settings)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::High)
            .member_calls(["this.loadData", "this.saveData"])
            .build(),
        ApiRule::builder("settings.ui")
            .label("Registers plugin settings UI")
            .category(ApiCategory::Settings)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::High)
            .member_calls(["this.addSettingTab"])
            .constructors([
                "PluginSettingTab",
                "Setting",
                "obsidian.PluginSettingTab",
                "obsidian.Setting",
            ])
            .member_calls(["getSettingDefinitions"])
            .build(),
        ApiRule::builder("lifecycle.methods")
            .label("Defines plugin lifecycle methods")
            .category(ApiCategory::Lifecycle)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::Medium)
            .member_calls(["this.onload", "this.onunload"])
            .custom_ast("lifecycle_methods", custom_matchers::lifecycle_methods)
            .build(),
        ApiRule::builder("lifecycle.events")
            .label("Registers events, DOM handlers, or intervals")
            .category(ApiCategory::Lifecycle)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::High)
            .member_calls([
                "this.registerEvent",
                "this.registerDomEvent",
                "this.registerInterval",
            ])
            .global_calls(["setInterval", "setTimeout", "requestAnimationFrame"])
            .implies(["disclosure.global_handlers_or_timers"])
            .build(),
        ApiRule::builder("plugins.internal_access")
            .label("Accesses plugin internals or other plugins")
            .category(ApiCategory::Dependency)
            .severity(ApiSeverity::Notice)
            .confidence(Confidence::Medium)
            .member_reads([
                "this.app.plugins",
                "app.plugins",
                "this.app.plugins.enabledPlugins",
                "app.plugins.enabledPlugins",
                "this.app.plugins.manifests",
                "app.plugins.manifests",
            ])
            .rooted_member_calls(["this.app.plugins.getPlugin", "app.plugins.getPlugin"])
            .implies(["disclosure.plugin_internals"])
            .build(),
        ApiRule::builder("platform.branching")
            .label("Branches on desktop, mobile, or OS platform")
            .category(ApiCategory::Dependency)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::High)
            // REVIEW: this detects reads of Platform flags, not actual conditional branching. The
            // disclosure is conservative because these values usually gate platform-specific behavior.
            .module_member_calls(
                "obsidian",
                [
                    "Platform.isMobile",
                    "Platform.isDesktop",
                    "Platform.isMacOS",
                    "Platform.isWin",
                    "Platform.isLinux",
                ],
            )
            .member_reads([
                "Platform.isMobile",
                "Platform.isDesktop",
                "Platform.isMacOS",
                "Platform.isWin",
                "Platform.isLinux",
                "obsidian.Platform.isMobile",
                "obsidian.Platform.isDesktop",
                "obsidian.Platform.isMacOS",
                "obsidian.Platform.isWin",
                "obsidian.Platform.isLinux",
            ])
            .implies(["disclosure.platform_branching"])
            .build(),
        ApiRule::builder("filesystem.node")
            .label("Uses direct Node filesystem-related modules")
            .category(ApiCategory::Filesystem)
            .severity(ApiSeverity::Warning)
            .confidence(Confidence::High)
            .imports([
                "fs",
                "fs/promises",
                "node:fs",
                "node:fs/promises",
                "path",
                "node:path",
                "os",
                "node:os",
                "stream",
                "node:stream",
                "buffer",
                "node:buffer",
                "zlib",
                "node:zlib",
            ])
            .implies(["disclosure.node_filesystem_access"])
            .build(),
        ApiRule::builder("process.node")
            .label("Uses Node process or subprocess APIs")
            .category(ApiCategory::Filesystem)
            .severity(ApiSeverity::Warning)
            .confidence(Confidence::High)
            .imports(["child_process", "node:child_process"])
            .member_reads(["process.env", "process.platform"])
            .implies(["disclosure.process_or_shell_access"])
            .build(),
        ApiRule::builder("electron.desktop")
            .label("Uses Electron desktop APIs")
            .category(ApiCategory::Electron)
            .severity(ApiSeverity::Warning)
            .confidence(Confidence::High)
            .imports(["electron"])
            .member_calls([
                "shell.openExternal",
                "ipcRenderer.send",
                "ipcRenderer.invoke",
            ])
            .build(),
        ApiRule::builder("electron.ipc_shell")
            .label("Uses Electron IPC or shell APIs")
            .category(ApiCategory::Electron)
            .severity(ApiSeverity::Warning)
            .confidence(Confidence::High)
            .member_calls([
                "shell.openExternal",
                "shell.openPath",
                "ipcRenderer.send",
                "ipcRenderer.invoke",
                "remote.require",
            ])
            .implies(["disclosure.process_or_shell_access"])
            .build(),
        ApiRule::builder("browser.clipboard")
            .label("Reads or writes clipboard data")
            .category(ApiCategory::Browser)
            .severity(ApiSeverity::Notice)
            .confidence(Confidence::High)
            .rooted_member_calls([
                "navigator.clipboard.read",
                "navigator.clipboard.readText",
                "navigator.clipboard.write",
                "navigator.clipboard.writeText",
            ])
            .implies(["disclosure.clipboard_access"])
            .build(),
        ApiRule::builder("browser.storage")
            .label("Uses persistent browser storage")
            .category(ApiCategory::Browser)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::High)
            .member_reads(["localStorage", "sessionStorage", "indexedDB", "caches"])
            .member_calls([
                "localStorage.getItem",
                "localStorage.setItem",
                "sessionStorage.getItem",
                "sessionStorage.setItem",
                "indexedDB.open",
                "caches.open",
            ])
            .build(),
        ApiRule::builder("browser.permissions")
            .label("Uses permission-sensitive browser APIs")
            .category(ApiCategory::Browser)
            .severity(ApiSeverity::Warning)
            .confidence(Confidence::High)
            .rooted_member_calls([
                "navigator.geolocation.getCurrentPosition",
                "navigator.mediaDevices.getUserMedia",
                "Notification.requestPermission",
                "navigator.bluetooth.requestDevice",
            ])
            .implies(["disclosure.permission_sensitive_browser_api"])
            .build(),
        ApiRule::builder("browser.permission_availability")
            .label("Checks permission-sensitive browser API availability")
            .category(ApiCategory::Browser)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::Medium)
            .member_reads([
                "navigator.geolocation",
                "navigator.mediaDevices",
                "Notification",
                "RTCPeerConnection",
                "navigator.bluetooth",
            ])
            .build(),
        ApiRule::builder("browser.environment")
            .label("Reads browser or device environment data")
            .category(ApiCategory::Browser)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::Medium)
            .member_reads([
                "navigator.userAgent",
                "navigator.platform",
                "navigator.language",
                "Intl.DateTimeFormat",
                "screen.width",
                "screen.height",
            ])
            .implies(["disclosure.browser_environment_access"])
            .build(),
        ApiRule::builder("browser.broad_input_hooks")
            .label("Registers broad keyboard handlers or clipboard hooks")
            .category(ApiCategory::Browser)
            .severity(ApiSeverity::Notice)
            .confidence(Confidence::Medium)
            .member_call("document.addEventListener")
            .arg_string(0, ["keydown", "keyup", "paste", "copy", "cut"])
            .member_call("window.addEventListener")
            .arg_string(0, ["keydown", "keyup", "paste", "copy", "cut"])
            .member_calls([
                "navigator.clipboard.readText",
                "navigator.clipboard.writeText",
            ])
            .implies(["disclosure.global_handlers_or_timers"])
            .build(),
        ApiRule::builder("archive.compression")
            .label("Uses compression or archive handling")
            .category(ApiCategory::Filesystem)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::Medium)
            .imports(["jszip", "tar", "zlib", "node:zlib", "fflate"])
            .string_literals(["gzip", "zip", "JSZip"])
            .build(),
        ApiRule::builder("crypto.hashing")
            .label("Uses cryptography or hashing APIs")
            .category(ApiCategory::Filesystem)
            .severity(ApiSeverity::Info)
            .confidence(Confidence::Medium)
            .imports(["crypto", "node:crypto", "crypto-js"])
            .member_calls([
                "crypto.subtle.digest",
                "crypto.subtle.encrypt",
                "crypto.subtle.decrypt",
            ])
            .string_literals(["sha256", "SHA-256", "AES-GCM"])
            .build(),
        ApiRule::builder("dynamic_code")
            .label("Evaluates dynamic code or injects scripts")
            .category(ApiCategory::DynamicCode)
            .severity(ApiSeverity::Critical)
            .confidence(Confidence::High)
            .calls(["import"])
            .global_calls(["eval"])
            .constructors(["Function"])
            .custom_ast(
                "dynamic_code_execution",
                custom_matchers::dynamic_code_execution,
            )
            .custom_ast(
                "remote_dom_script_injection",
                custom_matchers::remote_dom_script_injection,
            )
            .implies(["disclosure.dynamic_code_or_remote_code"])
            .build(),
    ];

    rules
        .into_iter()
        .map(|rule| rule.expect("built-in API classifier rule should be valid"))
        .collect()
}

pub(in crate::plugins::analysis::mainjs) fn validate_catalog(
    rules: &[ApiRule],
) -> Result<(), ApiCatalogError> {
    let mut ids = BTreeSet::new();
    let mut disclosures = known_disclosures();

    for rule in rules {
        if !ids.insert(rule.id.clone()) {
            return Err(ApiCatalogError::DuplicateRule(rule.id.clone()));
        }
        disclosures.extend(rule.implies.iter().cloned());
    }

    for rule in rules {
        for reference in rule
            .implies
            .iter()
            .chain(&rule.when_all)
            .chain(&rule.when_any)
        {
            if reference.starts_with("disclosure.") && !disclosures.contains(reference) {
                return Err(ApiCatalogError::UnknownDisclosure(reference.clone()));
            }
            if !reference.starts_with("disclosure.") && !ids.contains(reference) {
                return Err(ApiCatalogError::UnknownRule(reference.clone()));
            }
        }
    }

    Ok(())
}

fn known_disclosures() -> BTreeSet<String> {
    [
        "disclosure.network_access",
        "disclosure.cors_free_network_access",
        "disclosure.third_party_services",
        "disclosure.private_network_access",
        "disclosure.telemetry_or_error_reporting",
        "disclosure.note_content_access",
        "disclosure.metadata_access",
        "disclosure.full_vault_access",
        "disclosure.vault_file_write",
        "disclosure.obsidian_config_access",
        "disclosure.adapter_file_access",
        "disclosure.node_filesystem_access",
        "disclosure.process_or_shell_access",
        "disclosure.clipboard_access",
        "disclosure.dynamic_code_or_remote_code",
        "disclosure.editor_behavior",
        "disclosure.markdown_processing",
        "disclosure.workspace_layout",
        "disclosure.global_handlers_or_timers",
        "disclosure.plugin_internals",
        "disclosure.platform_branching",
        "disclosure.permission_sensitive_browser_api",
        "disclosure.browser_environment_access",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}
