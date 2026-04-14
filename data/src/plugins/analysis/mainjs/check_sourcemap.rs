pub(super) fn detect_sourcemap_comment(source: &str) -> bool {
    source.lines().any(|line| {
        let trimmed = line.trim();

        trimmed.starts_with("//# sourceMappingURL=")
            || trimmed.starts_with("//@ sourceMappingURL=")
            || trimmed.starts_with("/*# sourceMappingURL=")
            || trimmed.starts_with("/*@ sourceMappingURL=")
    })
}

#[cfg(test)]
mod tests {
    use super::detect_sourcemap_comment;

    #[test]
    fn detects_line_comment_form() {
        let source = "const a=1;\n//# sourceMappingURL=main.js.map";
        assert!(detect_sourcemap_comment(source));
    }

    #[test]
    fn detects_block_comment_form() {
        let source = "const a=1;\n/*# sourceMappingURL=main.js.map */";
        assert!(detect_sourcemap_comment(source));
    }

    #[test]
    fn ignores_string_literal_mentions() {
        let source = "const s = \"//# sourceMappingURL=main.js.map\";";
        assert!(!detect_sourcemap_comment(source));
    }
}
