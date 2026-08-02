use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read},
    path::Path,
    sync::{Arc, OnceLock},
};

use data_lib::plugin::{
    MainJsDiagnostic, MainJsDisclosure, MainJsEvidence, MainJsFinding, MainJsLocation,
};
use glass_lint_core::{
    EcmaVersion, Linter, MatchCertainty, analyze_ecma_version,
    project::{Diagnostic, SourceFile},
};
use glass_lint_obsidian::obsidian_config;

mod check_minified;
mod disclosures;

#[derive(Debug, Default, PartialEq)]
pub(super) struct MainJsResult {
    pub(super) estimated_target_es_version: Option<String>,
    pub(super) findings: Vec<MainJsFinding>,
    pub(super) disclosures: Vec<MainJsDisclosure>,
    pub(super) diagnostics: Vec<MainJsDiagnostic>,
    pub(super) is_probably_minified: Option<bool>,
    pub(super) minification_score: Option<f32>,
}

fn linter() -> &'static Linter {
    static LINTER: OnceLock<Linter> = OnceLock::new();
    LINTER.get_or_init(|| {
        // Glass Lint's pinned core has a fixed 8 MiB source limit. Its
        // parser emits source_too_large through the normal report path, so
        // the release pipeline intentionally does not add a second byte
        // limit or parser here.
        Linter::new(obsidian_config()).expect("pinned Glass Lint configuration is valid")
    })
}

#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn analyze_main_js(source: &str) -> MainJsResult {
    analyze_main_js_source(Arc::<str>::from(source.to_owned()))
}

pub(super) fn analyze_main_js_file(path: &Path) -> std::io::Result<MainJsResult> {
    let file = File::open(path)?;
    let mut bytes = Vec::with_capacity(glass_lint_core::MAX_SOURCE_BYTES + 1);
    BufReader::new(file)
        .take((glass_lint_core::MAX_SOURCE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;

    if bytes.len() > glass_lint_core::MAX_SOURCE_BYTES {
        return Ok(size_limit_result());
    }

    let source = match String::from_utf8(bytes) {
        Ok(source) => Arc::<str>::from(source),
        Err(_) => return Ok(invalid_utf8_result()),
    };

    Ok(analyze_main_js_source(source))
}

fn analyze_main_js_source(source: Arc<str>) -> MainJsResult {
    let (is_probably_minified, minification_score) = if source.len()
        <= glass_lint_core::MAX_SOURCE_BYTES
    {
        let (is_probably_minified, minification_score) = check_minified::detect_minified(&source);
        (Some(is_probably_minified), Some(minification_score))
    } else {
        (None, None)
    };

    let source_file = SourceFile::new("main.js", source.clone())
        .expect("main.js is a valid relative source path");
    let estimated_target_es_version = detect_ecma_version(&source_file);
    let report = linter()
        .lint_source(source_file)
        .expect("single main.js source is a valid Glass Lint project");
    let metadata = metadata();
    let mut result = MainJsResult {
        estimated_target_es_version,
        diagnostics: report
            .diagnostics()
            .iter()
            .map(diagnostic_to_model)
            .collect(),
        is_probably_minified,
        minification_score,
        ..MainJsResult::default()
    };

    let mut finding_indexes: HashMap<String, usize> = HashMap::new();
    for file in report.files() {
        result
            .diagnostics
            .extend(file.diagnostics().iter().map(diagnostic_to_model));

        for finding in file.findings() {
            let rule_id = finding.rule_id().as_str().to_owned();
            let evidence = finding
                .evidence()
                .traces()
                .iter()
                .flat_map(|trace| {
                    trace.steps().iter().map(|step| MainJsEvidence {
                        message: step.message().to_owned(),
                        count: 1,
                    })
                })
                .collect::<Vec<_>>();
            let finding_data = MainJsFinding {
                description: metadata.get(&rule_id).cloned().unwrap_or_default(),
                message: finding.message().to_owned(),
                severity: finding.severity().as_str().to_owned(),
                confidence: certainty_as_str(finding.certainty()).to_owned(),
                evidence,
                rule_id: rule_id.clone(),
            };

            let is_new = if let Some(index) = finding_indexes.get(&rule_id) {
                let existing = &mut result.findings[*index];
                existing.evidence = aggregate_evidence(
                    existing
                        .evidence
                        .drain(..)
                        .chain(finding_data.evidence)
                        .collect::<Vec<_>>(),
                );
                false
            } else {
                let index = result.findings.len();
                finding_indexes.insert(rule_id.clone(), index);
                result.findings.push(finding_data);
                true
            };

            if is_new {
                for disclosure in disclosures::for_rule(&rule_id) {
                    result.disclosures.push(MainJsDisclosure {
                        id: (*disclosure).to_owned(),
                        from_rule_id: rule_id.clone(),
                    });
                }
            }
        }
    }

    result
}

fn metadata() -> &'static HashMap<String, String> {
    static METADATA: OnceLock<HashMap<String, String>> = OnceLock::new();
    METADATA.get_or_init(|| {
        linter()
            .catalog()
            .metadata()
            .into_iter()
            .map(|rule| (rule.id.as_str().to_owned(), rule.description))
            .collect()
    })
}

fn aggregate_evidence(items: Vec<MainJsEvidence>) -> Vec<MainJsEvidence> {
    let mut indexes: HashMap<String, usize> = HashMap::new();
    let mut result: Vec<MainJsEvidence> = Vec::new();
    for item in items {
        if let Some(index) = indexes.get(&item.message) {
            result[*index].count = result[*index].count.saturating_add(item.count);
        } else {
            indexes.insert(item.message.clone(), result.len());
            result.push(item);
        }
    }
    result
}

fn size_limit_result() -> MainJsResult {
    diagnostic_result(
        "source_too_large",
        format!(
            "source exceeds the {} byte analysis limit",
            glass_lint_core::MAX_SOURCE_BYTES
        ),
    )
}

fn invalid_utf8_result() -> MainJsResult {
    diagnostic_result("invalid_utf8", "main.js is not valid UTF-8".to_string())
}

fn diagnostic_result(code: &str, message: String) -> MainJsResult {
    MainJsResult {
        diagnostics: vec![MainJsDiagnostic {
            code: code.to_string(),
            message,
            location: None,
        }],
        ..MainJsResult::default()
    }
}

fn detect_ecma_version(source: &SourceFile) -> Option<String> {
    analyze_ecma_version(source)
        .ok()
        .and_then(|report| report.minimum_version())
        .map(|version: EcmaVersion| version.to_string())
}

fn certainty_as_str(certainty: MatchCertainty) -> &'static str {
    match certainty {
        MatchCertainty::Definite => "definite",
        MatchCertainty::Possible => "possible",
    }
}

fn diagnostic_to_model(diagnostic: &Diagnostic) -> MainJsDiagnostic {
    MainJsDiagnostic {
        code: diagnostic.code().to_owned(),
        message: diagnostic.message().to_owned(),
        location: diagnostic.range().map(|range| MainJsLocation {
            start_line: range.start().line(),
            start_column: range.start().column(),
            end_line: range.end().line(),
            end_column: range.end().column(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::{analyze_main_js, analyze_main_js_file, disclosures, linter};
    use tempfile::NamedTempFile;

    fn temporary_file(contents: &[u8]) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut file, contents).unwrap();
        file
    }

    #[test]
    fn reports_js_and_obsidian_findings_with_glass_metadata() {
        let result =
            analyze_main_js("fetch('/data'); app.vault.getAbstractFileByPath('notes.md');");

        assert!(result.findings.iter().any(|finding| {
            finding.rule_id.starts_with("browser:")
                && !finding.description.is_empty()
                && !finding.evidence.is_empty()
        }));
        assert!(
            result
                .findings
                .iter()
                .any(|finding| finding.rule_id.starts_with("obsidian:"))
        );
    }

    #[test]
    fn detects_ecma_version_with_glass_lint_core() {
        let result = analyze_main_js("async function run() { await work(); }");

        assert_eq!(
            result.estimated_target_es_version.as_deref(),
            Some("ES2017")
        );
    }

    #[test]
    fn preserves_parse_diagnostics_and_partial_findings() {
        let result = analyze_main_js("app.vault.getAbstractFileByPath('notes.md'); }");

        assert!(
            result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "syntax_error")
        );
    }

    #[test]
    fn empty_and_minified_sources_are_stable() {
        let empty = analyze_main_js("");
        let minified = analyze_main_js("fetch('x');app.vault.getAbstractFileByPath('y');");

        assert!(empty.findings.is_empty());
        assert_eq!(
            minified,
            analyze_main_js("fetch('x');app.vault.getAbstractFileByPath('y');")
        );
    }

    #[test]
    fn evidence_is_counted_once_per_message() {
        let result = analyze_main_js("fetch('one'); fetch('two');");
        for finding in result.findings {
            let mut messages = finding
                .evidence
                .iter()
                .map(|evidence| evidence.message.as_str())
                .collect::<Vec<_>>();
            messages.sort_unstable();
            messages.dedup();
            assert_eq!(messages.len(), finding.evidence.len());
            assert!(finding.evidence.iter().all(|evidence| evidence.count > 0));
        }
    }

    #[test]
    fn reports_the_pinned_parser_size_diagnostic() {
        let result = analyze_main_js(&"x".repeat(glass_lint_core::MAX_SOURCE_BYTES + 1));

        assert!(
            result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "source_too_large")
        );
    }

    #[test]
    fn file_reader_reports_invalid_utf8() {
        let file = temporary_file(&[b'v', b'a', 0xff]);
        let result = analyze_main_js_file(file.path()).unwrap();
        assert_eq!(result.diagnostics[0].code, "invalid_utf8");
    }

    #[test]
    fn oversized_file_takes_precedence_over_invalid_utf8() {
        let mut contents = vec![b'x'; glass_lint_core::MAX_SOURCE_BYTES + 1];
        contents[glass_lint_core::MAX_SOURCE_BYTES] = 0xff;
        let file = temporary_file(&contents);
        let result = analyze_main_js_file(file.path()).unwrap();
        assert_eq!(result.diagnostics[0].code, "source_too_large");
    }

    #[test]
    fn persists_multiple_disclosures_without_filtering_findings() {
        let mapped = analyze_main_js("app.vault.adapter.read('notes.md');");
        assert!(
            mapped
                .findings
                .iter()
                .any(|finding| finding.rule_id == "obsidian:vault.adapter")
        );
        assert_eq!(
            mapped
                .disclosures
                .iter()
                .filter(|disclosure| disclosure.from_rule_id == "obsidian:vault.adapter")
                .map(|disclosure| disclosure.id.as_str())
                .collect::<Vec<_>>(),
            vec!["disclosure.vault_read", "disclosure.vault_write"]
        );

        let unmapped = analyze_main_js("app.vault.getAbstractFileByPath('notes.md');");
        assert!(
            unmapped
                .findings
                .iter()
                .any(|finding| finding.rule_id == "obsidian:vault.enumerate")
        );
        assert!(unmapped.disclosures.iter().any(|disclosure| {
            disclosure.id == "disclosure.full_vault_access"
                && disclosure.from_rule_id == "obsidian:vault.enumerate"
        }));
    }

    #[test]
    fn every_glass_lint_rule_has_a_disclosure_mapping() {
        let missing = linter()
            .catalog()
            .rule_ids()
            .iter()
            .filter(|rule_id| disclosures::for_rule(rule_id.as_str()).is_empty())
            .map(|rule_id| rule_id.as_str())
            .collect::<Vec<_>>();

        assert!(
            missing.is_empty(),
            "Glass Lint rules without disclosures: {}",
            missing.join(", ")
        );
    }
}
