/// Explicit project disclosures for the pinned Glass Lint rule catalog.
///
/// This is intentionally an exact-ID map. New or removed upstream rules stay
/// visible as findings but do not silently acquire a disclosure category.
pub(super) fn for_rule(rule_id: &str) -> &'static [&'static str] {
    match rule_id {
        "js:dynamic-code.eval"
        | "js:dynamic-code.string-timer"
        | "browser:dynamic-code.script-injection" => &["disclosure.dynamic_code_or_remote_code"],
        "js:dynamic-code.webassembly" => &["disclosure.webassembly_usage"],
        "js:concurrency.shared-memory" | "browser:browser.worker" => &["disclosure.worker_usage"],
        "js:network.url-construction"
        | "js:network.header-indicator"
        | "browser:network.request"
        | "browser:dom.remote-resource"
        | "node:node.network"
        | "obsidian:network.request" => &["disclosure.network_access"],
        "js:network.private-address" => &[
            "disclosure.network_access",
            "disclosure.private_network_access",
        ],
        "js:network.service-indicator" => &[
            "disclosure.network_access",
            "disclosure.third_party_services",
        ],
        "js:network.telemetry-indicator" => &[
            "disclosure.network_access",
            "disclosure.third_party_services",
            "disclosure.telemetry_or_error_reporting",
        ],
        "browser:browser.clipboard-read" | "browser:browser.clipboard-write" => {
            &["disclosure.clipboard_access"]
        }
        "browser:browser.persistent-storage" => &["disclosure.browser_storage_access"],
        "browser:browser.permissions-geolocation"
        | "browser:browser.permissions-hardware"
        | "browser:browser.permissions-media"
        | "browser:browser.permissions-bluetooth"
        | "browser:browser.permissions-notifications"
        | "browser:browser.permissions-query" => &["disclosure.permission_sensitive_browser_api"],
        "browser:browser.environment" => &["disclosure.browser_environment_access"],
        "browser:browser.global-input-hook" => &["disclosure.global_handlers_or_timers"],
        "browser:browser.file-dialog" | "browser:browser.filesystem" => {
            &["disclosure.browser_filesystem_access"]
        }
        "node:node.filesystem" => &["disclosure.node_filesystem_access"],
        "node:node.process-environment" | "node:node.subprocess" => {
            &["disclosure.process_or_shell_access"]
        }
        "node:archive.compression" => &["disclosure.archive_or_compression"],
        "node:crypto.operation" => &["disclosure.cryptographic_operations"],
        "electron:electron.module"
        | "electron:electron.ipc"
        | "electron:electron.shell"
        | "electron:electron.dialog" => &["disclosure.electron_api_access"],
        "obsidian:vault.access" => &["disclosure.vault_access"],
        "obsidian:vault.read" => &["disclosure.vault_read"],
        "obsidian:vault.write" | "obsidian:vault.delete" | "obsidian:vault.move-copy" => {
            &["disclosure.vault_write"]
        }
        "obsidian:vault.adapter" => &["disclosure.vault_read", "disclosure.vault_write"],
        "obsidian:vault.enumerate" => &["disclosure.full_vault_access"],
        "obsidian:vault.config-directory" => &["disclosure.obsidian_config_access"],
        "obsidian:vault.resource-url" | "obsidian:vault.events" => &["disclosure.vault_access"],
        "obsidian:metadata.cache-read"
        | "obsidian:metadata.frontmatter-read"
        | "obsidian:metadata.events"
        | "obsidian:metadata.traversal"
        | "obsidian:metadata.extract" => &["disclosure.metadata_access"],
        "obsidian:workspace.active-file" | "obsidian:workspace.events" => {
            &["disclosure.workspace_access"]
        }
        "obsidian:workspace.active-editor" => &["disclosure.editor_behavior"],
        "obsidian:workspace.open"
        | "obsidian:workspace.leaf-management"
        | "obsidian:workspace.layout" => &["disclosure.workspace_layout"],
        "obsidian:view.register"
        | "obsidian:ui.command"
        | "obsidian:ui.ribbon"
        | "obsidian:ui.status-bar"
        | "obsidian:ui.modal"
        | "obsidian:ui.notice"
        | "obsidian:ui.menu"
        | "obsidian:ui.settings-tab" => &["disclosure.user_interface_access"],
        "obsidian:editor.content" | "obsidian:editor.extension" | "obsidian:editor.suggest" => {
            &["disclosure.editor_behavior"]
        }
        "obsidian:file-manager.frontmatter-write" => {
            &["disclosure.metadata_access", "disclosure.vault_write"]
        }
        "obsidian:markdown.postprocessor"
        | "obsidian:markdown.code-block-processor"
        | "obsidian:markdown.render"
        | "obsidian:markdown.link" => &["disclosure.markdown_processing"],
        "obsidian:codemirror.extension" => &["disclosure.codemirror_integration"],
        "obsidian:storage.app-data"
        | "obsidian:storage.plugin-data-read"
        | "obsidian:storage.plugin-data-write" => &["disclosure.plugin_storage_access"],
        "obsidian:lifecycle.events" => &["disclosure.lifecycle_events"],
        "obsidian:bases.register" => &["disclosure.bases_integration"],
        "obsidian:cli.register" => &["disclosure.cli_integration"],
        "obsidian:platform.branching" => &["disclosure.platform_branching"],
        "obsidian:plugins.access"
        | "obsidian:plugins.enable-disable"
        | "obsidian:plugins.load-unload" => &["disclosure.plugin_management"],
        _ => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::for_rule;

    #[test]
    fn maps_exact_ids_and_leaves_unknown_findings_unmapped() {
        assert_eq!(for_rule("obsidian:vault.read"), &["disclosure.vault_read"]);
        assert_eq!(
            for_rule("obsidian:vault.adapter"),
            &["disclosure.vault_read", "disclosure.vault_write"]
        );
        assert!(for_rule("obsidian:unknown").is_empty());
    }
}
