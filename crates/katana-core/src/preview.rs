//! プレビュー用ドキュメントセクションモデル。
//!
//! Markdown ソースを「通常テキスト」と「ダイアグラムブロック」に分割し、
//! UI 層が各セクションを独立してレンダリングできるようにする。

use crate::markdown::diagram::DiagramKind;

/// ドキュメントを構成するセクションの種別。
#[derive(Debug, Clone)]
pub enum PreviewSection {
    /// 通常の Markdown テキスト。
    Markdown(String),
    /// ダイアグラムフェンスブロック。
    Diagram { kind: DiagramKind, source: String },
}

/// ソーステキストを `PreviewSection` のリストに分割する。
///
/// ダイアグラムフェンス（` ```mermaid` / ` ```plantuml` / ` ```drawio` ）を検出し、
/// それ以外を Markdown セクションとしてまとめる。
pub fn split_into_sections(source: &str) -> Vec<PreviewSection> {
    let mut sections = Vec::new();
    let mut markdown_acc = String::new();
    let mut remaining = source;

    while let Some(fence_pos) = remaining.find("\n```") {
        markdown_acc.push_str(&remaining[..fence_pos + 1]);
        remaining = &remaining[fence_pos + 1..];
        match try_parse_diagram_fence(remaining) {
            Some((kind, fence_source, after)) => {
                flush_markdown(&mut sections, &mut markdown_acc);
                sections.push(PreviewSection::Diagram {
                    kind,
                    source: fence_source,
                });
                remaining = after;
            }
            None => {
                // ダイアグラムでなければ Markdown としてそのまま扱う。
                markdown_acc.push_str("```");
                remaining = &remaining["```".len()..];
            }
        }
    }

    markdown_acc.push_str(remaining);
    flush_markdown(&mut sections, &mut markdown_acc);
    sections
}

/// 蓄積された Markdown テキストが空でなければセクションに追加する。
fn flush_markdown(sections: &mut Vec<PreviewSection>, acc: &mut String) {
    if !acc.is_empty() {
        sections.push(PreviewSection::Markdown(std::mem::take(acc)));
    }
}

/// 先頭がダイアグラムフェンスであれば `(kind, source, after)` を返す。
fn try_parse_diagram_fence(s: &str) -> Option<(DiagramKind, String, &str)> {
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
