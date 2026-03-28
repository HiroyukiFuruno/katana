use std::sync::mpsc::Sender;

const GITHUB_RAW_BASE: &str =
    "https://raw.githubusercontent.com/HiroyukiFuruno/KatanA/refs/heads/master";

/// A single version section parsed from the changelog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangelogSection {
    /// Version string (e.g. "0.8.0", "Unreleased").
    pub version: String,
    /// Full heading line (e.g. "## [0.8.0] - 2026-03-28").
    pub heading: String,
    /// The body content (markdown) for this version.
    pub body: String,
    /// Whether this section should be initially expanded.
    pub default_open: bool,
}

pub enum ChangelogEvent {
    Success(Vec<ChangelogSection>),
    Error(String),
}

/// Start fetching the changelog in the background.
fn handle_fetch_result(
    result: Result<ehttp::Response, String>,
    tx: &Sender<ChangelogEvent>,
    current_version: &str,
    previous_version: Option<&str>,
) {
    match result {
        Ok(response) => {
            let text = match response.text() {
                Some(t) => t.to_string(),
                None => {
                    if response.ok {
                        let _ = tx.send(ChangelogEvent::Error(
                            "Failed to decode response text".to_string(),
                        ));
                    } else {
                        let _ = tx.send(ChangelogEvent::Error(format!(
                            "HTTP error: {}",
                            response.status
                        )));
                    }
                    return;
                }
            };

            if !response.ok {
                let _ = tx.send(ChangelogEvent::Error(format!(
                    "HTTP error {}: {}",
                    response.status,
                    text.chars().take(200).collect::<String>()
                )));
                return;
            }

            let sections = parse_changelog(&text, current_version, previous_version);
            let _ = tx.send(ChangelogEvent::Success(sections));
        }
        Err(err) => {
            let _ = tx.send(ChangelogEvent::Error(err));
        }
    }
}

pub fn fetch_changelog(
    language: &str,
    current_version: String,
    previous_version: Option<String>,
    tx: Sender<ChangelogEvent>,
) {
    let filename = if language.starts_with("ja") {
        "CHANGELOG.ja.md"
    } else {
        "CHANGELOG.md"
    };

    let url = format!("{}/{}", GITHUB_RAW_BASE, filename);
    let request = ehttp::Request::get(&url);

    fn do_fetch(
        request: ehttp::Request,
        tx: Sender<ChangelogEvent>,
        current_version: String,
        previous_version: Option<String>,
    ) {
        ehttp::fetch(request, move |result| {
            handle_fetch_result(result, &tx, &current_version, previous_version.as_deref());
        });
    }

    do_fetch(
        request,
        tx,
        current_version,
        previous_version.map(|s| s.to_string()),
    );
}

/// Parses raw changelog markdown into structured sections.
///
/// Sections between `previous_version` (exclusive) and `current_version`
/// (inclusive) are marked as `default_open = true`. All others are closed.
fn parse_changelog(
    raw_markdown: &str,
    current_version: &str,
    previous_version: Option<&str>,
) -> Vec<ChangelogSection> {
    let prev_ver = previous_version.unwrap_or("0.0.0");
    let mut sections = Vec::new();
    let mut parts = raw_markdown.split("\n## [");

    // Skip the introductory text (title, description, etc.)
    let _ = parts.next();

    for part in parts {
        let bracket_end = part.find(']').unwrap_or(0);
        let version_str = part[..bracket_end].trim().to_string();

        // Extract the heading line (up to the first newline)
        let heading_end = part.find('\n').unwrap_or(part.len());

        let date_part = if bracket_end + 1 < heading_end {
            part[bracket_end + 1..heading_end]
                .trim_start_matches([']', ' ', '-'])
                .trim()
        } else {
            ""
        };

        let heading = if date_part.is_empty() {
            format!("v{}", version_str)
        } else {
            format!("v{} - {}", version_str, date_part)
        };

        // Everything after the heading line is the body
        let body = if heading_end < part.len() {
            part[heading_end..].trim_end().to_string()
        } else {
            String::new()
        };

        let default_open = version_str != "Unreleased"
            && is_newer_or_equal(current_version, &version_str)
            && is_older(prev_ver, &version_str);

        sections.push(ChangelogSection {
            version: version_str,
            heading,
            body,
            default_open,
        });
    }

    sections
}

/// A naive semantic version comparison for versions like "0.8.0".
fn is_newer_or_equal(ver_a: &str, ver_b: &str) -> bool {
    let a = ver_a.trim_start_matches('v');
    let b = ver_b.trim_start_matches('v');
    compare_versions(a, b) >= 0
}

fn is_older(ver_a: &str, ver_b: &str) -> bool {
    let a = ver_a.trim_start_matches('v');
    let b = ver_b.trim_start_matches('v');
    compare_versions(a, b) < 0
}

fn compare_versions(a: &str, b: &str) -> i32 {
    let a_parts: Vec<u32> = a.split('.').filter_map(|s| s.parse().ok()).collect();
    let b_parts: Vec<u32> = b.split('.').filter_map(|s| s.parse().ok()).collect();
    for i in 0..std::cmp::max(a_parts.len(), b_parts.len()) {
        let va = a_parts.get(i).unwrap_or(&0);
        let vb = b_parts.get(i).unwrap_or(&0);
        if va > vb {
            return 1;
        }
        if va < vb {
            return -1;
        }
    }
    0
}

/// Renders the release notes as a tab content instead of a modal window.
pub(crate) fn render_release_notes_tab(
    ui: &mut egui::Ui,
    sections: &[ChangelogSection],
    is_loading: bool,
) {
    if sections.is_empty() && !is_loading {
        return;
    }

    const TAB_OUTER_MARGIN_X: i8 = 32;
    const TAB_OUTER_MARGIN_Y: i8 = 24;
    const TAB_TITLE_SPACING: f32 = 16.0;
    const TAB_INNER_MARGIN_X: i8 = 16;
    const TAB_INNER_MARGIN_Y: i8 = 8;
    const TAB_BOTTOM_PADDING: f32 = 8.0;
    const TAB_SPINNER_SIZE: f32 = 32.0;

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.add_space(TAB_BOTTOM_PADDING);

            if sections.is_empty() && is_loading {
                ui.centered_and_justified(|ui| {
                    ui.add(egui::Spinner::new().size(TAB_SPINNER_SIZE));
                });
                return;
            }

            // Apply padding around the content
            egui::Frame::default()
                .inner_margin(egui::Margin::symmetric(
                    TAB_OUTER_MARGIN_X,
                    TAB_OUTER_MARGIN_Y,
                ))
                .show(ui, |ui| {
                    let title_text = format!(
                        "{} v{}",
                        crate::i18n::get().menu.release_notes,
                        env!("CARGO_PKG_VERSION")
                    );
                    ui.heading(egui::RichText::new(title_text));
                    ui.add_space(TAB_TITLE_SPACING);

                    for section in sections {
                        let id = ui.make_persistent_id(&section.version);
                        let mut state =
                            egui::collapsing_header::CollapsingState::load_with_default_open(
                                ui.ctx(),
                                id,
                                section.default_open,
                            );

                        let icon = if state.is_open() { "▼" } else { "▶" };
                        let text = format!("{} {}", icon, section.heading);

                        let response = ui.add(
                            egui::Label::new(egui::RichText::new(text).strong())
                                .sense(egui::Sense::click()),
                        );

                        if response.hovered() {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);

                            let stroke = egui::Stroke::new(1.0, ui.visuals().strong_text_color());
                            const UNDERLINE_Y_OFFSET: f32 = 1.5;
                            let underline_y = response.rect.max.y - UNDERLINE_Y_OFFSET;
                            ui.painter()
                                .hline(response.rect.x_range(), underline_y, stroke);
                        }

                        if response.clicked() {
                            state.toggle(ui);
                        }

                        state.show_body_indented(&response, ui, |ui| {
                            egui::Frame::default()
                                .inner_margin(egui::Margin::symmetric(
                                    TAB_INNER_MARGIN_X,
                                    TAB_INNER_MARGIN_Y,
                                ))
                                .show(ui, |ui| {
                                    // Render body as markdown
                                    let mut cache = egui_commonmark::CommonMarkCache::default();
                                    egui_commonmark::CommonMarkViewer::new().show(
                                        ui,
                                        &mut cache,
                                        &section.body,
                                    );
                                });
                        });

                        ui.add_space(2.0);
                    }
                });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_changelog_marks_new_versions_open() {
        let md = "# Changelog\n\n## [0.8.0] - 2026-03-28\n### Added\n- Feature A\n\n## [0.7.0] - 2026-02-01\n### Fixed\n- Bug B";

        let sections = parse_changelog(md, "0.8.0", Some("0.7.0"));

        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].version, "0.8.0");
        assert!(sections[0].default_open);
        assert_eq!(sections[1].version, "0.7.0");
        assert!(!sections[1].default_open);
    }

    #[test]
    fn test_parse_changelog_unreleased_is_closed() {
        let md = "# Changelog\n\n## [Unreleased]\n### Added\n- Feature\n\n## [v0.8.0] - DATE";
        let sections = parse_changelog(md, "0.8.0", Some("0.7.0"));
        assert_eq!(sections[0].version, "Unreleased");
        assert!(!sections[0].default_open);
    }

    #[test]
    fn test_parse_changelog_no_previous_opens_all_up_to_current() {
        let md = "# Changelog\n\n## [0.8.0] - DATE\n### Added\n- A\n\n## [0.7.0] - DATE\n### Fixed\n- B\n\n## [0.6.0] - DATE\n### Changed\n- C";
        let sections = parse_changelog(md, "0.8.0", None);

        // With no previous version (defaults to "0.0.0"), all versions
        // up to and including current_version are open.
        assert!(sections[0].default_open); // 0.8.0
        assert!(sections[1].default_open); // 0.7.0
        assert!(sections[2].default_open); // 0.6.0
    }

    #[test]
    fn test_parse_changelog_body_extraction() {
        let md = "# Changelog\n\n## [0.8.0] - 2026-03-28\n### Added\n- Feature A\n- Feature B";
        let sections = parse_changelog(md, "0.8.0", Some("0.7.0"));
        assert!(sections[0].body.contains("### Added"));
        assert!(sections[0].body.contains("- Feature A"));
        assert!(sections[0].body.contains("- Feature B"));
    }

    #[test]
    fn test_compare_versions() {
        assert_eq!(compare_versions("0.8.0", "0.7.0"), 1);
        assert_eq!(compare_versions("0.7.0", "0.8.0"), -1);
        assert_eq!(compare_versions("0.8.0", "0.8.0"), 0);
        assert_eq!(compare_versions("1.0.0", "0.9.9"), 1);
        assert_eq!(compare_versions("0.8.0.1", "0.8.0"), 1);
    }

    #[test]
    fn test_is_newer_or_equal() {
        assert!(is_newer_or_equal("v0.8.0", "0.7.0"));
        assert!(is_newer_or_equal("v0.8.0", "v0.8.0"));
        assert!(!is_newer_or_equal("0.7.0", "v0.8.0"));
    }

    #[test]
    fn test_is_older() {
        assert!(is_older("0.7.0", "v0.8.0"));
        assert!(!is_older("v0.8.0", "v0.8.0"));
        assert!(!is_older("0.8.0", "0.7.0"));
    }

    #[test]
    fn test_fetch_changelog_coverage() {
        let (tx, _rx) = std::sync::mpsc::channel();
        fetch_changelog("en", "0.8.0".to_string(), None, tx.clone());
        fetch_changelog("ja", "0.8.0".to_string(), None, tx);
    }

    #[test]
    fn test_render_release_notes_tab_ui() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let sections = vec![ChangelogSection {
                    version: "0.8.0".to_string(),
                    heading: "v0.8.0".to_string(),
                    body: "# Test\n- Item".to_string(),
                    default_open: true,
                }];

                // Test loading state
                render_release_notes_tab(ui, &[], true);

                // Test content state
                render_release_notes_tab(ui, &sections, false);

                // Test empty state
                render_release_notes_tab(ui, &[], false);
            });
        });
    }

    #[test]
    fn test_handle_fetch_result_network_error() {
        let (tx, rx) = std::sync::mpsc::channel();
        handle_fetch_result(Err("Offline".to_string()), &tx, "0.1.0", None);
        match rx.try_recv().unwrap() {
            ChangelogEvent::Error(e) => assert_eq!(e, "Offline"),
            _ => panic!("Expected Error event"),
        }
    }

    #[test]
    fn test_handle_fetch_result_http_error_with_text() {
        let (tx, rx) = std::sync::mpsc::channel();
        let response = ehttp::Response {
            url: "https://example.com".to_string(),
            ok: false,
            status: 404,
            status_text: "Not Found".to_string(),
            bytes: b"Not Found Data".to_vec(),
            headers: ehttp::Headers::new(&[]),
        };
        handle_fetch_result(Ok(response), &tx, "0.1.0", None);
        match rx.try_recv().unwrap() {
            ChangelogEvent::Error(e) => assert_eq!(e, "HTTP error 404: Not Found Data"),
            _ => panic!("Expected Error"),
        }
    }

    #[test]
    fn test_handle_fetch_result_ok_response_decode_error() {
        let (tx, rx) = std::sync::mpsc::channel();
        // A response with invalid utf8 bytes
        let response = ehttp::Response {
            url: "https://example.com".to_string(),
            ok: true,
            status: 200,
            status_text: "OK".to_string(),
            bytes: vec![0xFF, 0xFE, 0xFD],
            headers: ehttp::Headers::new(&[]),
        };
        handle_fetch_result(Ok(response), &tx, "0.1.0", None);
        match rx.try_recv().unwrap() {
            ChangelogEvent::Error(e) => assert_eq!(e, "Failed to decode response text"),
            _ => panic!("Expected Error"),
        }
    }

    #[test]
    fn test_handle_fetch_result_failure_response_decode_error() {
        let (tx, rx) = std::sync::mpsc::channel();
        // A bad status response with invalid utf8 bytes
        let response = ehttp::Response {
            url: "https://example.com".to_string(),
            ok: false,
            status: 500,
            status_text: "Server Error".to_string(),
            bytes: vec![0xFF, 0xFE, 0xFD],
            headers: ehttp::Headers::new(&[]),
        };
        handle_fetch_result(Ok(response), &tx, "0.1.0", None);
        match rx.try_recv().unwrap() {
            ChangelogEvent::Error(e) => assert_eq!(e, "HTTP error: 500"),
            _ => panic!("Expected Error"),
        }
    }

    #[test]
    fn test_handle_fetch_result_success() {
        let (tx, rx) = std::sync::mpsc::channel();
        let md = "# Changelog\n## [0.8.0]\n### Added\n- Ok!";
        let response = ehttp::Response {
            url: "https://example.com".to_string(),
            ok: true,
            status: 200,
            status_text: "OK".to_string(),
            bytes: md.as_bytes().to_vec(),
            headers: ehttp::Headers::new(&[]),
        };
        handle_fetch_result(Ok(response), &tx, "0.8.0", None);
        match rx.try_recv().unwrap() {
            ChangelogEvent::Success(sections) => {
                assert_eq!(sections.len(), 1);
                assert_eq!(sections[0].version, "0.8.0");
            }
            _ => panic!("Expected Success"),
        }
    }
}
