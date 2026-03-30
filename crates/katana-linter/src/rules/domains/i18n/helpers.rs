use crate::utils::is_emoji_or_symbol;

/// List of UI method names to inspect.
pub fn ui_methods() -> Vec<&'static str> {
    vec![
        "label",
        "heading",
        "button",
        "on_hover_text",
        "selectable_label",
        "checkbox",
        "radio",
        "radio_value",
        "small_button",
        "text_edit_singleline",
        "hyperlink_to",
        "collapsing",
    ]
}

/// List of function calls (`Type::func()` format) to inspect.
pub fn ui_functions() -> Vec<&'static str> {
    vec!["new"]
}

/// Target type names for function calls.
pub fn ui_types_for_new() -> Vec<&'static str> {
    vec!["RichText", "Button"]
}

pub fn is_format_macro(mac: &syn::Macro) -> bool {
    mac.path
        .segments
        .last()
        .map(|it| it.ident == "format")
        .unwrap_or(false)
}

pub fn extract_type_from_call(func: &syn::Expr) -> Option<String> {
    if let syn::Expr::Path(expr_path) = func {
        let segments = &expr_path.path.segments;
        if segments.len() >= 2 {
            return Some(segments[segments.len() - 2].ident.to_string());
        }
    }
    None
}

pub fn is_raw_icon(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed == "x" || trimmed == "X" {
        return true;
    }
    trimmed.chars().any(is_emoji_or_symbol)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_emoji_or_symbol_tag_range() {
        assert!(is_emoji_or_symbol('\u{E0001}'));
        assert!(is_emoji_or_symbol('\u{E007F}'));
    }
}
