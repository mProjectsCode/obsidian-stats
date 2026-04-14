use swc_ecma_ast::{Ident, Program};
use swc_ecma_visit::{Visit, VisitWith};

pub(super) fn detect_minified(source: &str, program: Option<&Program>) -> (bool, f32) {
    let normalized_source = strip_sourcemap_comments(source);

    let line_count = normalized_source.lines().count().max(1) as f32;
    let source_len = normalized_source.len().max(1) as f32;
    let avg_line_len = source_len / line_count;

    let whitespace_count = normalized_source
        .chars()
        .filter(|ch| ch.is_whitespace())
        .count() as f32;
    let punctuation_count = normalized_source
        .chars()
        .filter(|ch| "{}[]();,:.=+-*/<>!?&|".contains(*ch))
        .count() as f32;

    let newline_ratio = normalized_source.matches('\n').count() as f32 / source_len;
    let whitespace_ratio = whitespace_count / source_len;
    let punctuation_ratio = punctuation_count / source_len;
    let comment_ratio = estimate_comment_ratio(&normalized_source);

    let mut score = 0.0f32;

    if avg_line_len > 300.0 {
        score += 0.30;
    } else if avg_line_len > 180.0 {
        score += 0.18;
    }

    if newline_ratio < 0.004 {
        score += 0.25;
    } else if newline_ratio < 0.008 {
        score += 0.12;
    }

    if whitespace_ratio < 0.10 {
        score += 0.20;
    } else if whitespace_ratio < 0.14 {
        score += 0.10;
    }

    if punctuation_ratio > 0.16 {
        score += 0.12;
    } else if punctuation_ratio > 0.12 {
        score += 0.06;
    }

    if comment_ratio > 0.10 {
        score -= 0.10;
    } else if comment_ratio > 0.04 {
        score -= 0.05;
    }

    if let Some(program) = program {
        let mut visitor = IdentStats::default();
        program.visit_with(&mut visitor);
        if visitor.total > 0 {
            let short_ratio = visitor.short as f32 / visitor.total as f32;
            if short_ratio > 0.55 {
                score += 0.20;
            } else if short_ratio > 0.40 {
                score += 0.10;
            }
        }
    }

    let score = score.clamp(0.0, 1.0);
    (score >= 0.5, score)
}

fn strip_sourcemap_comments(source: &str) -> String {
    source
        .lines()
        .filter(|line| {
            let trimmed = line.trim();

            !trimmed.starts_with("//# sourceMappingURL=")
                && !trimmed.starts_with("//@ sourceMappingURL=")
                && !trimmed.starts_with("/*# sourceMappingURL=")
                && !trimmed.starts_with("/*@ sourceMappingURL=")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn estimate_comment_ratio(source: &str) -> f32 {
    let total_lines = source.lines().count().max(1) as f32;
    let mut comment_lines = 0u32;
    let mut in_block_comment = false;

    for line in source.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        if in_block_comment {
            comment_lines += 1;
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }

        if trimmed.starts_with("//") {
            comment_lines += 1;
            continue;
        }

        if trimmed.starts_with("/*") {
            comment_lines += 1;
            if !trimmed.contains("*/") {
                in_block_comment = true;
            }
            continue;
        }

        if trimmed.starts_with('*') {
            comment_lines += 1;
        }
    }

    comment_lines as f32 / total_lines
}

#[derive(Default)]
struct IdentStats {
    total: usize,
    short: usize,
}

impl Visit for IdentStats {
    fn visit_ident(&mut self, ident: &Ident) {
        self.total += 1;
        if ident.sym.len() <= 2 {
            self.short += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{detect_minified, strip_sourcemap_comments};

    #[test]
    fn removes_all_common_sourcemap_comment_forms() {
        let source =
            "const a=1;\n//# sourceMappingURL=main.js.map\n/*# sourceMappingURL=vendor.js.map */";
        let stripped = strip_sourcemap_comments(source);

        assert_eq!(stripped, "const a=1;");
    }

    #[test]
    fn sourcemap_line_does_not_change_score() {
        let base = "const veryLongIdentifierName=1;const anotherVeryLongIdentifierName=2;";
        let with_map = format!("{base}\n//# sourceMappingURL=main.js.map");

        let (_, base_score) = detect_minified(base, None);
        let (_, with_map_score) = detect_minified(&with_map, None);

        assert_eq!(base_score, with_map_score);
    }

    #[test]
    fn comments_reduce_minification_score() {
        let mostly_code = "const a=1;const b=2;const c=a+b;";
        let with_comments = "// comment\n// comment\n// comment\nconst a=1;const b=2;const c=a+b;";

        let (_, score_without_comments) = detect_minified(mostly_code, None);
        let (_, score_with_comments) = detect_minified(with_comments, None);

        assert!(score_with_comments < score_without_comments);
    }
}
