use crate::markdown::diagram::DiagramKind;

pub fn try_parse_diagram_fence(s: &str) -> Option<(DiagramKind, String, &str)> {
    let body = s.strip_prefix("```")?;
    let info_end = body.find('\n')?;
    let info = body[..info_end].trim();
    let kind = DiagramKind::from_info(info)?;
    let after_info = &body[info_end + 1..];
    let close = after_info.find("\n```")?;
    let source = after_info[..close].to_string();
    let rest_start = close + "\n```".len();
    let after = after_info[rest_start..]
        .strip_prefix('\n')
        .unwrap_or(&after_info[rest_start..]);
    Some((kind, source, after))
}
