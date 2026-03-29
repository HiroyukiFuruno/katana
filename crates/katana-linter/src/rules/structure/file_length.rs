use crate::utils::has_cfg_test_attr;
use crate::Violation;
use std::path::Path;

pub fn lint_file_length(path: &Path, syntax: &syn::File) -> Vec<Violation> {
    const MAX_LINES: usize = 200;

    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    if has_cfg_test_attr(&syntax.attrs) {
        return Vec::new();
    }

    let total_lines = source.lines().count();

    let test_lines = count_test_module_lines(syntax, &source);
    let effective_lines = total_lines.saturating_sub(test_lines);

    if effective_lines > MAX_LINES {
        vec![Violation {
            file: path.to_path_buf(),
            line: 1,
            column: 1,
            message: format!(
                "File has {effective_lines} lines (excluding tests), exceeding the {MAX_LINES}-line limit. \
                 Consider splitting into smaller modules (e.g. types.rs + impls.rs)."
            ),
        }]
    } else {
        Vec::new()
    }
}

fn count_test_module_lines(syntax: &syn::File, source: &str) -> usize {
    use syn::spanned::Spanned;

    let source_lines: Vec<&str> = source.lines().collect();
    let mut test_lines = 0;

    for item in &syntax.items {
        if let syn::Item::Mod(item_mod) = item {
            if has_cfg_test_attr(&item_mod.attrs) {
                let start = item_mod.span().start().line;
                let end = item_mod.span().end().line;
                // WHY: attributes like #[cfg(test)] sit above the mod keyword
                let attr_start = item_mod
                    .attrs
                    .first()
                    .map(|a| a.span().start().line)
                    .unwrap_or(start);
                let effective_start = attr_start.min(start);
                let line_count = end.saturating_sub(effective_start) + 1;
                test_lines += line_count.min(source_lines.len());
            }
        }
    }

    test_lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn write_temp_file(content: &str) -> (tempfile::TempDir, PathBuf) {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        let mut f = std::fs::File::create(&file_path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        (dir, file_path)
    }

    #[test]
    fn passes_short_file() {
        let code = "fn foo() {}\nfn bar() {}\n";
        let (_dir, path) = write_temp_file(code);
        let syntax = syn::parse_file(code).unwrap();
        let violations = lint_file_length(&path, &syntax);
        assert!(violations.is_empty());
    }

    #[test]
    fn detects_long_file() {
        let mut code = String::new();
        for i in 0..201 {
            code.push_str(&format!("const C{i}: i32 = 0;\n"));
        }
        let (_dir, path) = write_temp_file(&code);
        let syntax = syn::parse_file(&code).unwrap();
        let violations = lint_file_length(&path, &syntax);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("201 lines"));
    }

    #[test]
    fn excludes_test_module_lines() {
        let mut code = String::new();
        // 150 lines of production code
        for i in 0..150 {
            code.push_str(&format!("const C{i}: i32 = 0;\n"));
        }
        // 100 lines of test code (should be excluded)
        code.push_str("#[cfg(test)]\nmod tests {\n");
        for i in 0..98 {
            code.push_str(&format!("    const T{i}: i32 = 0;\n"));
        }
        code.push_str("}\n");

        let (_dir, path) = write_temp_file(&code);
        let syntax = syn::parse_file(&code).unwrap();
        let violations = lint_file_length(&path, &syntax);
        // 150 production lines should be under 200
        assert!(violations.is_empty());
    }
}
