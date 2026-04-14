use swc_ecma_ast::{Ident, Program};
use swc_ecma_visit::{Visit, VisitWith};

pub(super) fn detect_minified(source: &str, program: Option<&Program>) -> (bool, f32) {
    let line_count = source.lines().count().max(1) as f32;
    let source_len = source.len().max(1) as f32;
    let avg_line_len = source_len / line_count;

    let whitespace_count = source.chars().filter(|ch| ch.is_whitespace()).count() as f32;
    let punctuation_count = source
        .chars()
        .filter(|ch| "{}[]();,:.=+-*/<>!?&|".contains(*ch))
        .count() as f32;

    let newline_ratio = source.matches('\n').count() as f32 / source_len;
    let whitespace_ratio = whitespace_count / source_len;
    let punctuation_ratio = punctuation_count / source_len;

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
    (score >= 0.75, score)
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
