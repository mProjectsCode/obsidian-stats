use std::collections::BTreeMap;

use regex::Regex;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) struct StringSignals {
    pub(super) embedded_blob_type_counts: BTreeMap<String, u32>,
    pub(super) known_api_host_counts: BTreeMap<String, u32>,
    pub(super) dependency_name_counts: BTreeMap<String, u32>,
    pub(super) license_banner_count: u32,
    pub(super) credential_literal_count: u32,
}

pub(super) fn detect_string_signals(source: &str) -> StringSignals {
    StringSignals {
        embedded_blob_type_counts: detect_embedded_blob_type_counts(source),
        known_api_host_counts: detect_known_api_host_counts(source),
        dependency_name_counts: detect_dependency_name_counts(source),
        license_banner_count: count_license_banners(source),
        credential_literal_count: count_credential_literals(source),
    }
}

fn detect_embedded_blob_type_counts(source: &str) -> BTreeMap<String, u32> {
    let mut counts = BTreeMap::new();
    for (marker, blob_type) in [
        ("data:application/wasm", "wasm"),
        ("data:image/", "image"),
        ("data:font/", "font"),
        ("data:application/font", "font"),
        ("data:application/zip", "zip"),
        ("data:application/x-zip", "zip"),
        (".wasm", "wasm"),
        ("UEsDB", "zip"),
        ("AGFzbQE", "wasm"),
        ("iVBORw0KGgo", "image"),
        ("/9j/", "image"),
        ("R0lGOD", "image"),
        ("AAEAAA", "font"),
        ("d09GMg", "font"),
        ("d09GRg", "font"),
    ] {
        let count = count_marker(source, marker);
        if count > 0 {
            *counts.entry(blob_type.to_string()).or_insert(0) += count;
        }
    }

    let known_blob_markers = [
        "data:application/wasm",
        "data:image/",
        "data:font/",
        "data:application/font",
        "data:application/zip",
        "data:application/x-zip",
        "UEsDB",
        "AGFzbQE",
        "iVBORw0KGgo",
        "/9j/",
        "R0lGOD",
        "AAEAAA",
        "d09GMg",
        "d09GRg",
    ];
    if let Ok(re) = Regex::new(r"[A-Za-z0-9+/]{1024,}={0,2}") {
        for blob in re.find_iter(source).map(|match_| match_.as_str()) {
            if blob.len().is_multiple_of(4)
                && !known_blob_markers
                    .iter()
                    .any(|marker| blob.contains(marker))
            {
                *counts.entry("unknown".to_string()).or_insert(0) += 1;
            }
        }
    }

    counts
}

fn detect_known_api_host_counts(source: &str) -> BTreeMap<String, u32> {
    let Ok(re) = Regex::new(r#"https?://([^/"'\s\\?#]+)"#) else {
        return BTreeMap::new();
    };

    let mut counts = BTreeMap::new();
    for captures in re.captures_iter(source) {
        let Some(host) = captures.get(1) else {
            continue;
        };
        let host = host.as_str().trim_end_matches('.').to_ascii_lowercase();
        if is_known_api_host(&host) {
            *counts.entry(host).or_insert(0) += 1;
        }
    }

    counts
}

fn is_known_api_host(host: &str) -> bool {
    [
        "api.",
        "openai.com",
        "anthropic.com",
        "googleapis.com",
        "openrouter.ai",
        "replicate.com",
        "huggingface.co",
        "github.com",
        "gitlab.com",
        "dropboxapi.com",
        "graph.microsoft.com",
        "amazonaws.com",
        "supabase.co",
        "firebaseio.com",
        "notion.com",
        "airtable.com",
        "todoist.com",
        "telegram.org",
        "discord.com",
        "slack.com",
        "sentry.io",
        "posthog",
        "plausible.io",
        "google-analytics.com",
        "googletagmanager.com",
        "mixpanel.com",
        "segment.com",
        "amplitude.com",
        "datadoghq.com",
    ]
    .iter()
    .any(|marker| host.contains(marker))
}

fn detect_dependency_name_counts(source: &str) -> BTreeMap<String, u32> {
    let mut counts = BTreeMap::new();
    for dependency in [
        "react",
        "lodash",
        "moment",
        "dayjs",
        "axios",
        "jquery",
        "svelte",
        "vue",
        "solid-js",
        "zustand",
        "codemirror",
        "prosemirror",
        "jszip",
        "pdfjs",
        "mermaid",
        "openai",
        "anthropic",
        "firebase",
        "supabase",
    ] {
        let count = count_case_insensitive_word(source, dependency);
        if count > 0 {
            counts.insert(dependency.to_string(), count);
        }
    }

    counts
}

fn count_license_banners(source: &str) -> u32 {
    [
        "/*!",
        "license",
        "licensed",
        "copyright",
        "MIT License",
        "Apache License",
        "BSD License",
        "GPL",
    ]
    .iter()
    .map(|marker| count_case_insensitive(source, marker))
    .sum()
}

fn count_credential_literals(source: &str) -> u32 {
    [
        "apiKey",
        "api_key",
        "accessToken",
        "access_token",
        "authToken",
        "bearer ",
        "client_secret",
        "password",
        "secret",
        "webhook",
    ]
    .iter()
    .map(|marker| count_case_insensitive(source, marker))
    .sum()
}

fn count_marker(source: &str, marker: &str) -> u32 {
    source
        .matches(marker)
        .count()
        .try_into()
        .unwrap_or(u32::MAX)
}

fn count_case_insensitive(source: &str, marker: &str) -> u32 {
    source
        .to_ascii_lowercase()
        .matches(&marker.to_ascii_lowercase())
        .count()
        .try_into()
        .unwrap_or(u32::MAX)
}

fn count_case_insensitive_word(source: &str, word: &str) -> u32 {
    let pattern = format!(r"(?i)(^|[^a-z0-9_-]){}([^a-z0-9_-]|$)", regex::escape(word));
    let Ok(re) = Regex::new(&pattern) else {
        return 0;
    };
    re.find_iter(source).count().try_into().unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::detect_string_signals;

    #[test]
    fn detects_string_derived_static_signals() {
        let unknown_blob = "A".repeat(1024);
        let source = format!(
            r#"
            /*! lodash MIT License */
            const png = "data:image/png;base64,iVBORw0KGgoAAA";
            const wasm = "data:application/wasm;base64,AGFzbQEAAAA";
            const url = "https://api.openai.com/v1/chat/completions";
            const keyName = "apiKey";
            const unknown = "{unknown_blob}";
        "#
        );

        let signals = detect_string_signals(&source);

        assert_eq!(signals.embedded_blob_type_counts.get("image"), Some(&2));
        assert_eq!(signals.embedded_blob_type_counts.get("wasm"), Some(&2));
        assert_eq!(
            signals.known_api_host_counts.get("api.openai.com"),
            Some(&1)
        );
        assert_eq!(signals.dependency_name_counts.get("lodash"), Some(&1));
        assert_eq!(signals.embedded_blob_type_counts.get("unknown"), Some(&1));
        assert!(signals.license_banner_count >= 2);
        assert_eq!(signals.credential_literal_count, 1);
    }
}
