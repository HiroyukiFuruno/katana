use crate::Violation;
use std::path::Path;
use syn::{ImplItem, Item, ItemEnum, ItemImpl, ItemStruct};

const MAX_LENGTH_FOR_MIXED_FILE: usize = 50;

fn check_item_for_types_and_logic(item: &Item) -> (bool, bool, usize) {
    match item {
        Item::Struct(ItemStruct {
            vis: syn::Visibility::Public(_),
            ident,
            ..
        }) => (true, false, ident.span().start().line),
        Item::Enum(ItemEnum {
            vis: syn::Visibility::Public(_),
            ident,
            ..
        }) => (true, false, ident.span().start().line),
        Item::Impl(ItemImpl { items, .. }) => {
            let has_logic = items.iter().any(|i| matches!(i, ImplItem::Fn(_)));
            (false, has_logic, 0)
        }
        _ => (false, false, 0),
    }
}

fn check_syntax_for_types_and_logic(syntax: &syn::File) -> (bool, bool, usize) {
    let mut has_pub_type = false;
    let mut has_logic_impl = false;
    let mut first_pub_type_line = 0;

    for item in &syntax.items {
        let (is_type, is_logic, line) = check_item_for_types_and_logic(item);
        if is_type {
            has_pub_type = true;
            if first_pub_type_line == 0 {
                first_pub_type_line = line;
            }
        }
        if is_logic {
            has_logic_impl = true;
        }
    }
    (has_pub_type, has_logic_impl, first_pub_type_line)
}

fn create_violation(path: &Path, line: usize, num_lines: usize) -> Violation {
    let rel_path = path
        .strip_prefix(std::env::current_dir().unwrap_or_default())
        .unwrap_or(path)
        .to_path_buf();
    Violation {
        file: rel_path,
        line,
        column: 1,
        message: format!("Mixed logic and data. File ({num_lines} lines) defines pub struct/enum but also contains method logic. Move types to `types.rs` or `types/` dir, or keep file under {MAX_LENGTH_FOR_MIXED_FILE} lines."),
    }
}

pub fn lint_type_separation(path: &Path, syntax: &syn::File) -> Vec<Violation> {
    let mut violations = Vec::new();
    let path_str = path.to_string_lossy();
    if path_str.contains("/tests/") || path_str.ends_with("tests.rs") {
        return violations;
    }
    let Ok(source) = std::fs::read_to_string(path) else {
        return violations;
    };
    let num_lines = source.lines().count();
    if is_whitelisted_type_file(&path_str) || num_lines <= MAX_LENGTH_FOR_MIXED_FILE {
        return violations;
    }

    let (has_pub_type, has_logic, line) = check_syntax_for_types_and_logic(syntax);

    if has_pub_type && has_logic {
        violations.push(create_violation(path, line, num_lines));
    }

    violations
}

fn is_whitelisted_type_file(path_str: &str) -> bool {
    if path_str.ends_with("types.rs")
        || path_str.ends_with("type.rs")
        || path_str.ends_with("models.rs")
        || path_str.ends_with("model.rs")
        || path_str.ends_with("state.rs")
    {
        return true;
    }
    if path_str.contains("/types/") || path_str.contains("/models/") || path_str.contains("/state/")
    {
        return true;
    }
    if path_str.ends_with("lib.rs") || path_str.ends_with("main.rs") {
        return true;
    }
    false
}
