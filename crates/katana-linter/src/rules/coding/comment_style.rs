use crate::Violation;
use std::path::Path;

pub fn lint_comment_style(path: &Path, _syntax: &syn::File) -> Vec<Violation> {
    let Ok(source) = std::fs::read_to_string(path) else {
        return vec![];
    };
    let mut violations = Vec::new();
    let mut in_test = false;

    let mut previous_was_pattern = false;
    let mut in_block = false;
    let mut block_allow = false;

    for (idx, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("#[cfg(test)]") {
            in_test = true;
        }
        if in_test {
            continue;
        }

        if in_block {
            let check_line = trimmed.strip_suffix("*/").unwrap_or(trimmed).trim();
            if !block_allow && !is_allowed(check_line) {
                violations.push(build_viol(path, line.trim(), idx, "Multi-line block comment must start with `/* WHY:` or `/* SAFETY:`."));
            }
            if line.contains("*/") {
                in_block = false;
            }
            continue;
        }

        if let Some((kind, start)) = find_comment_start(line) {
            let text = &line[start..];
            match kind {
                CommentKind::Line => {
                    let body = extract_body(text.trim(), "//").trim_start_matches(|c| c == '/' || c == '!');
                    let b = body.trim();
                    if b.starts_with("WHY:") || b.starts_with("SAFETY:") {
                        if previous_was_pattern {
                            violations.push(build_viol(path, text.trim(), idx, "Consecutive `// WHY:` lines are prohibited. Use `/* WHY: ... */` block comments for multi-line reasons."));
                        }
                        previous_was_pattern = true;
                    } else if !is_allowed(b) {
                        violations.push(build_viol(path, text.trim(), idx, "Comment must start with `WHY:` or `SAFETY:`. Doc comments (`///`) are strictly prohibited unless starting with `WHY:` or `SAFETY:`."));
                        previous_was_pattern = false;
                    } else {
                        previous_was_pattern = false;
                    }
                }
                CommentKind::Block => {
                    previous_was_pattern = false;
                    let body = extract_body(text.trim(), "/*").trim_start_matches(|c| c == '*' || c == '!');
                    let mut b = body;
                    if let Some(end) = b.find("*/") {
                        b = &b[..end]; // Single line block
                    } else {
                        in_block = true;
                    }

                    b = b.trim();
                    if b.starts_with("WHY:") || b.starts_with("SAFETY:") {
                        block_allow = true;
                    } else if !is_allowed(b) {
                        violations.push(build_viol(path, text.trim(), idx, "Block comment must start with `/* WHY:` or `/* SAFETY:`."));
                        block_allow = false;
                    } else {
                        block_allow = true; // explicitly allowed symbol strings like /* --- */
                    }
                }
            }
        } else {
            previous_was_pattern = false;
        }
    }
    violations
}

#[derive(Debug, PartialEq)]
enum CommentKind {
    Line,
    Block,
}

fn find_comment_start(line: &str) -> Option<(CommentKind, usize)> {
    if line.contains("r#\"") {
        return None;
    }
    let mut in_str = false;
    let mut in_char = false;
    let mut escape = false;
    let bytes = line.as_bytes();
    for i in 0..bytes.len() {
        if escape {
            escape = false;
            continue;
        }
        match bytes[i] {
            b'\\' => escape = true,
            b'"' if !in_char => in_str = !in_str,
            b'\'' if !in_str => in_char = !in_char,
            b'/' if !in_str && !in_char => {
                if i + 1 < bytes.len() {
                    if bytes[i + 1] == b'/' {
                        return Some((CommentKind::Line, i));
                    } else if bytes[i + 1] == b'*' {
                        return Some((CommentKind::Block, i));
                    }
                }
            }
            _ => {}
        }
    }
    None
}

fn extract_body<'a>(trimmed: &'a str, prefix: &str) -> &'a str {
    trimmed.strip_prefix(prefix).unwrap_or("").trim()
}

fn build_viol(path: &Path, trimmed: &str, line_idx: usize, msg_prefix: &str) -> Violation {
    let msg = format!("{} Found: `{}`", msg_prefix, truncate(trimmed, 60));
    Violation {
        file: path.to_path_buf(),
        line: line_idx + 1,
        column: 1,
        message: msg,
    }
}

fn is_allowed(body: &str) -> bool {
    let clean = body.trim_end_matches("*/").trim();
    clean.is_empty()
        || clean
            .chars()
            .all(|c| matches!(c, '-' | '─' | '═' | '=' | ' ' | '/' | '━' | '*'))
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}...", s.chars().take(max).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn write_tmp(content: &str) -> (tempfile::TempDir, PathBuf) {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.rs");
        std::fs::File::create(&path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();
        (dir, path)
    }

    #[test]
    fn allows_why_safety() {
        let code = "// WHY: reason\n// SAFETY: ptr ok\nfn f() {}\n";
        let (_d, p) = write_tmp(code);
        assert!(lint_comment_style(&p, &syn::parse_file(code).unwrap()).is_empty());
    }

    #[test]
    fn allows_doc_and_sep() {
        let code = "// ---\nfn f() {}\n";
        let (_d, p) = write_tmp(code);
        assert!(lint_comment_style(&p, &syn::parse_file(code).unwrap()).is_empty());
    }

    #[test]
    fn rejects_doc_without_why() {
        let code = "/// some doc\nfn f() {}\n";
        let (_d, p) = write_tmp(code);
        assert_eq!(lint_comment_style(&p, &syn::parse_file(code).unwrap()).len(), 1);
    }

    #[test]
    fn rejects_invalid() {
        let code = "// invalid\nfn f() {}\n";
        let (_d, p) = write_tmp(code);
        assert_eq!(lint_comment_style(&p, &syn::parse_file(code).unwrap()).len(), 1);
    }

    #[test]
    fn allows_multiline_block_why() {
        let code = "/* WHY: Some really long\nreason that spans\nmultiple lines */\nfn f() {}\n";
        let (_d, p) = write_tmp(code);
        assert!(lint_comment_style(&p, &syn::parse_file(code).unwrap()).is_empty());
    }

    #[test]
    fn rejects_invalid_block() {
        let code = "/* INVALID: garbage\nspans\nmultiple lines */\nfn f() {}\n";
        let (_d, p) = write_tmp(code);
        assert_eq!(lint_comment_style(&p, &syn::parse_file(code).unwrap()).len(), 3); // 3 lines of violations
    }

    #[test]
    fn rejects_consecutive_why_lines() {
        let code = "// WHY: reason 1\n// WHY: reason 2\nfn f() {}\n";
        let (_d, p) = write_tmp(code);
        assert_eq!(lint_comment_style(&p, &syn::parse_file(code).unwrap()).len(), 1);
    }
}
