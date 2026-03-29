#[cfg(test)]
mod tests {
    use super::*;

    use crate::app_state::{AppState, ScrollSource};
    use crate::preview_pane::PreviewPane;
    use katana_platform::PaneOrder;

    use eframe::egui::{self, pos2, Rect};
    use eframe::App as _;
    use egui::load::{BytesLoadResult, BytesLoader, LoadError};
    use katana_core::{document::Document, workspace::TreeEntry};
    use std::path::{Path, PathBuf};
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    pub(crate) const PREVIEW_CONTENT_PADDING: f32 = 12.0;

    /// Custom testing egui Context that pre-populates dummy font mappings for Markdown
    /// layout families. PreviewPane panics if these are missing natively.
    fn test_context() -> egui::Context {
        let ctx = egui::Context::default();
        let mut fonts = egui::FontDefinitions::default();
        let md_prop = fonts
            .families
            .get(&egui::FontFamily::Proportional)
            .cloned()
            .unwrap_or_default();
        let md_mono = fonts
            .families
            .get(&egui::FontFamily::Monospace)
            .cloned()
            .unwrap_or_default();
        fonts.families.insert(
            egui::FontFamily::Name("MarkdownProportional".into()),
            md_prop,
        );
        fonts
            .families
            .insert(egui::FontFamily::Name("MarkdownMonospace".into()), md_mono);
        ctx.set_fonts(fonts);
        ctx
    }

    fn test_input(size: egui::Vec2) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), size)),
            ..Default::default()
        }
    }

    fn flatten_shapes<'a>(
        shapes: impl IntoIterator<Item = &'a egui::epaint::ClippedShape>,
    ) -> Vec<&'a egui::epaint::Shape> {
        fn visit<'a>(shape: &'a egui::epaint::Shape, acc: &mut Vec<&'a egui::epaint::Shape>) {
            match shape {
                egui::epaint::Shape::Vec(children) => {
                    for child in children {
                        visit(child, acc);
                    }
                }
                _ => acc.push(shape),
            }
        }

        let mut flat = Vec::new();
        for clipped in shapes {
            visit(&clipped.shape, &mut flat);
        }
        flat
    }

    fn state_with_active_doc(path: &std::path::Path) -> AppState {
        let mut state = AppState::new(
            Default::default(),
            Default::default(),
            Default::default(),
            std::sync::Arc::new(katana_platform::InMemoryCacheService::default()),
        );
        state
            .document
            .open_documents
            .push(Document::new(path, "# Heading\n\nBody"));
        state.document.active_doc_idx = Some(0);
        state
    }

    fn app_with_preview_doc(path: &Path, markdown: &str) -> KatanaApp {
        let mut app = KatanaApp::new(state_with_active_doc(path));
        if let Some(doc) = app.state.active_document_mut() {
            doc.buffer = markdown.to_string();
        }
        let mut pane = PreviewPane::default();
        let cache = app.state.config.cache.clone();
        let concurrency = app
            .state
            .config
            .settings
            .settings()
            .performance
            .diagram_concurrency;
        pane.full_render(markdown, path, cache, false, concurrency);
        pane.wait_for_renders();
        app.tab_previews.push(crate::shell::TabPreviewCache {
            path: path.to_path_buf(),
            pane,
            hash: 0,
        });
        app
    }

    struct CountingBytesLoader {
        forget_all_calls: Arc<AtomicUsize>,
    }

    impl BytesLoader for CountingBytesLoader {
        fn id(&self) -> &str {
            egui::generate_loader_id!(CountingBytesLoader)
        }

        fn load(&self, _ctx: &egui::Context, _uri: &str) -> BytesLoadResult {
            Err(LoadError::NotSupported)
        }

        fn forget(&self, _uri: &str) {}

        fn forget_all(&self) {
            self.forget_all_calls.fetch_add(1, Ordering::SeqCst);
        }

        fn byte_size(&self) -> usize {
            0
        }

        fn has_pending(&self) -> bool {
            false
        }
    }

    #[test]
    fn preview_header_leaves_height_for_preview_body() {
        let ctx = test_context();
        let state = state_with_active_doc(std::path::Path::new("/tmp/preview.md"));
        let mut action = AppAction::None;
        let mut before_height = 0.0;
        let mut remaining_height = 0.0;

        let _ = ctx.run(test_input(egui::vec2(800.0, 600.0)), |ctx| {
            egui::CentralPanel::default()
                .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.0))
                .show(ctx, |ui| {
                    before_height = ui.available_height();
                    crate::views::panels::preview::render_preview_header(ui, &state, &mut action);
                    remaining_height = ui.available_height();
                });
        });

        assert!(
            (before_height - remaining_height).abs() <= 1.0,
            "preview header must overlay without consuming layout height, before={before_height}, after={remaining_height}"
        );
    }

    #[test]
    fn active_file_highlight_is_painted_before_text() {
        let ctx = test_context();
        let path = std::path::PathBuf::from("/tmp/CHANGELOG.md");
        let entry = TreeEntry::File { path: path.clone() };
        let mut action = AppAction::None;
        let mut expanded_directories = std::collections::HashSet::new();

        let output = ctx.run(test_input(egui::vec2(320.0, 200.0)), |ctx| {
            egui::CentralPanel::default()
                .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.0))
                .show(ctx, |ui| {
                    let mut render_ctx = TreeRenderContext {
                        action: &mut action,
                        depth: 0,
                        active_path: Some(path.as_path()),
                        filter_set: None,
                        expanded_directories: &mut expanded_directories,
                        disable_context_menu: false,
                    };
                    crate::views::panels::workspace::render_file_entry(
                        ui,
                        &entry,
                        &path,
                        &mut render_ctx,
                    );
                });
        });

        let shapes = flatten_shapes(output.shapes.iter());
        let highlight_idx = shapes.iter().position(|shape| {
            matches!(
                shape,
                egui::epaint::Shape::Rect(rect)
                    if rect.fill == ctx.style().visuals.selection.bg_fill
            )
        });
        let text_idx = shapes.iter().position(|shape| {
            matches!(
                shape,
                egui::epaint::Shape::Text(text)
                    if text.galley.job.text.contains("CHANGELOG.md")
            )
        });

        let highlight_idx = highlight_idx.expect("active row highlight was not painted");
        let text_idx = text_idx.expect("active row label text was not painted");

        assert!(
            highlight_idx < text_idx,
            "active row background must be behind its text, got rect index {highlight_idx} and text index {text_idx}"
        );
    }

    #[test]
    fn split_preview_left_padding_is_consistent() {
        let ctx = test_context();
        let path = PathBuf::from("/tmp/padding.md");
        let mut app = app_with_preview_doc(&path, "# PaddingHeading\n\nBody");
        let output = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
            crate::views::layout::split::render_horizontal_split(
                ctx,
                &mut app,
                PaneOrder::EditorFirst,
            );
        });

        let preview_rect = egui::containers::panel::PanelState::load(
            &ctx,
            crate::views::panels::preview::preview_panel_id(
                Some(path.as_path()),
                "preview_panel_h_right",
            ),
        )
        .expect("preview panel rect")
        .rect;
        let shapes = flatten_shapes(output.shapes.iter());
        let heading_rect = shapes
            .iter()
            .find_map(|shape| match shape {
                egui::epaint::Shape::Text(text)
                    if text.galley.job.text.contains("PaddingHeading") =>
                {
                    let rect = text.visual_bounding_rect();
                    if rect.left() >= preview_rect.left() - 1.0 {
                        Some(rect)
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .expect("heading text shape");

        let left_padding = heading_rect.left() - preview_rect.left();
        assert!(
            (left_padding - PREVIEW_CONTENT_PADDING).abs() <= 2.0,
            "preview left padding must be {}px, got {left_padding}",
            PREVIEW_CONTENT_PADDING
        );
    }

    #[test]
    fn new_horizontal_split_starts_at_half_width_even_if_another_tab_has_panel_state() {
        let ctx = test_context();
        let active = PathBuf::from("/tmp/active.md");
        let stale = PathBuf::from("/tmp/stale.md");
        let mut app = app_with_preview_doc(&active, "Body");

        ctx.data_mut(|data| {
            data.insert_persisted(
                crate::views::panels::preview::preview_panel_id(
                    Some(stale.as_path()),
                    "preview_panel_h_right",
                ),
                egui::containers::panel::PanelState {
                    rect: Rect::from_min_size(pos2(0.0, 0.0), egui::vec2(240.0, 800.0)),
                },
            );
        });

        let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
            crate::views::layout::split::render_horizontal_split(
                ctx,
                &mut app,
                PaneOrder::EditorFirst,
            );
        });

        let preview_rect = egui::containers::panel::PanelState::load(
            &ctx,
            crate::views::panels::preview::preview_panel_id(
                Some(active.as_path()),
                "preview_panel_h_right",
            ),
        )
        .expect("preview panel rect")
        .rect;
        assert!(
            (preview_rect.width() - 600.0).abs() <= 4.0,
            "fresh horizontal split must start at 50%, got {}",
            preview_rect.width()
        );
    }

    #[test]
    fn horizontal_split_width_stays_stable_across_initial_frames() {
        let ctx = test_context();
        let active = PathBuf::from("/tmp/active.md");
        let mut app = app_with_preview_doc(&active, "# Title\n\nBody");

        let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
            crate::views::layout::split::render_horizontal_split(
                ctx,
                &mut app,
                PaneOrder::EditorFirst,
            );
        });
        let first_rect = egui::containers::panel::PanelState::load(
            &ctx,
            crate::views::panels::preview::preview_panel_id(
                Some(active.as_path()),
                "preview_panel_h_right",
            ),
        )
        .expect("preview panel rect after first frame")
        .rect;

        let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
            crate::views::layout::split::render_horizontal_split(
                ctx,
                &mut app,
                PaneOrder::EditorFirst,
            );
        });
        let second_rect = egui::containers::panel::PanelState::load(
            &ctx,
            crate::views::panels::preview::preview_panel_id(
                Some(active.as_path()),
                "preview_panel_h_right",
            ),
        )
        .expect("preview panel rect after second frame")
        .rect;

        assert!(
            (first_rect.width() - 600.0).abs() <= 4.0,
            "first frame must start at 50%, got {}",
            first_rect.width()
        );
        assert!(
            (second_rect.width() - first_rect.width()).abs() <= 4.0,
            "horizontal split width must remain stable across frames, first={} second={}",
            first_rect.width(),
            second_rect.width()
        );
    }

    #[test]
    fn horizontal_split_width_stays_stable_with_readme_like_preview_content() {
        let ctx = test_context();
        let active = PathBuf::from("/tmp/readme.md");
        let markdown = concat!(
            "# KatanA Desktop\n\n",
            "> Note: On macOS Sequoia (15.x), Gatekeeper requires this command for apps not notarized with Apple.\n",
            "> Alternatively, go to System Settings -> Privacy & Security -> \"Open Anyway\" after the first launch attempt.\n\n",
            "Current Status\n\n",
            "KatanA Desktop is under active development. See the Releases page for the latest version and changelog.\n"
        );
        let mut app = app_with_preview_doc(&active, markdown);

        let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
            crate::views::layout::split::render_horizontal_split(
                ctx,
                &mut app,
                PaneOrder::EditorFirst,
            );
        });
        let first_rect = egui::containers::panel::PanelState::load(
            &ctx,
            crate::views::panels::preview::preview_panel_id(
                Some(active.as_path()),
                "preview_panel_h_right",
            ),
        )
        .expect("preview panel rect after first frame")
        .rect;

        let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
            crate::views::layout::split::render_horizontal_split(
                ctx,
                &mut app,
                PaneOrder::EditorFirst,
            );
        });
        let second_rect = egui::containers::panel::PanelState::load(
            &ctx,
            crate::views::panels::preview::preview_panel_id(
                Some(active.as_path()),
                "preview_panel_h_right",
            ),
        )
        .expect("preview panel rect after second frame")
        .rect;

        assert!(
            (first_rect.width() - 600.0).abs() <= 4.0,
            "first frame must start at 50%, got {}",
            first_rect.width()
        );
        assert!(
            (second_rect.width() - first_rect.width()).abs() <= 4.0,
            "horizontal split width must remain stable with README-like preview content, first={} second={}",
            first_rect.width(),
            second_rect.width()
        );
    }

    #[test]
    fn horizontal_split_width_stays_stable_with_changelog_like_list_content() {
        let ctx = test_context();
        let active = PathBuf::from("/tmp/changelog.md");
        let markdown = concat!(
            "## Fixes\n\n",
            "- Dark theme DrawIO contrast fix using `drawio_label_color`\n",
            "- Fixed `mmdc` lookup when launched from `.dmg/.app` without a standard PATH\n",
            "- Stabilized i18n tests under parallel execution\n\n",
            "## Improvements\n\n",
            "- Expanded `mmdc` binary resolution across Homebrew, env vars, and shell fallback\n",
            "- Extracted `CHANNEL_MAX`, `LUMA_R/G/B`, and `RENDER_POLL_INTERVAL_MS`\n"
        );
        let mut app = app_with_preview_doc(&active, markdown);

        let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
            crate::views::layout::split::render_horizontal_split(
                ctx,
                &mut app,
                PaneOrder::EditorFirst,
            );
        });
        let first_rect = egui::containers::panel::PanelState::load(
            &ctx,
            crate::views::panels::preview::preview_panel_id(
                Some(active.as_path()),
                "preview_panel_h_right",
            ),
        )
        .expect("preview panel rect after first frame")
        .rect;

        let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
            crate::views::layout::split::render_horizontal_split(
                ctx,
                &mut app,
                PaneOrder::EditorFirst,
            );
        });
        let second_rect = egui::containers::panel::PanelState::load(
            &ctx,
            crate::views::panels::preview::preview_panel_id(
                Some(active.as_path()),
                "preview_panel_h_right",
            ),
        )
        .expect("preview panel rect after second frame")
        .rect;

        assert!(
            (first_rect.width() - 600.0).abs() <= 4.0,
            "first frame must start at 50%, got {}",
            first_rect.width()
        );
        assert!(
            (second_rect.width() - first_rect.width()).abs() <= 4.0,
            "horizontal split width must remain stable with changelog-like list content, first={} second={}",
            first_rect.width(),
            second_rect.width()
        );
    }

    #[test]
    fn new_vertical_split_starts_at_half_height_even_if_another_tab_has_panel_state() {
        let ctx = test_context();
        let active = PathBuf::from("/tmp/active.md");
        let stale = PathBuf::from("/tmp/stale.md");
        let mut app = app_with_preview_doc(&active, "Body");

        ctx.data_mut(|data| {
            data.insert_persisted(
                crate::views::panels::preview::preview_panel_id(
                    Some(stale.as_path()),
                    "preview_panel_v_bottom",
                ),
                egui::containers::panel::PanelState {
                    rect: Rect::from_min_size(pos2(0.0, 0.0), egui::vec2(1200.0, 180.0)),
                },
            );
        });

        let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
            crate::views::layout::split::render_vertical_split(
                ctx,
                &mut app,
                PaneOrder::EditorFirst,
            );
        });

        let preview_rect = egui::containers::panel::PanelState::load(
            &ctx,
            crate::views::panels::preview::preview_panel_id(
                Some(active.as_path()),
                "preview_panel_v_bottom",
            ),
        )
        .expect("preview panel rect")
        .rect;
        assert!(
            (preview_rect.height() - 400.0).abs() <= 4.0,
            "fresh vertical split must start at 50%, got {}",
            preview_rect.height()
        );
    }

    #[test]
    fn split_preview_wraps_long_lines_without_horizontal_overflow() {
        let ctx = test_context();
        let path = PathBuf::from("/tmp/long-line.md");
        let long_line = "\u{3042}".repeat(240);
        let mut app = app_with_preview_doc(&path, &long_line);

        let output = ctx.run(test_input(egui::vec2(900.0, 700.0)), |ctx| {
            crate::views::layout::split::render_horizontal_split(
                ctx,
                &mut app,
                PaneOrder::EditorFirst,
            );
        });

        let preview_rect = egui::containers::panel::PanelState::load(
            &ctx,
            crate::views::panels::preview::preview_panel_id(
                Some(path.as_path()),
                "preview_panel_h_right",
            ),
        )
        .expect("preview panel rect")
        .rect;
        let shapes = flatten_shapes(output.shapes.iter());
        let text_shape = shapes
            .iter()
            .find_map(|shape| match shape {
                egui::epaint::Shape::Text(text)
                    if text.galley.job.text.contains(&long_line[..60]) =>
                {
                    Some(text)
                }
                _ => None,
            })
            .expect("long preview text shape");

        assert!(
            text_shape.galley.rows.len() > 1,
            "long preview line must wrap instead of staying on a single row"
        );
        assert!(
            text_shape.visual_bounding_rect().right()
                <= preview_rect.right() - PREVIEW_CONTENT_PADDING + 4.0,
            "wrapped preview text must stay within the preview panel"
        );
    }

    #[test]
    fn split_preview_wraps_long_inline_code_without_horizontal_overflow() {
        let ctx = test_context();
        let path = PathBuf::from("/tmp/long-inline-code.md");
        let inline_code = format!("`{}`", "\u{3042}".repeat(240));
        let mut app = app_with_preview_doc(&path, &inline_code);

        let output = ctx.run(test_input(egui::vec2(900.0, 700.0)), |ctx| {
            crate::views::layout::split::render_horizontal_split(
                ctx,
                &mut app,
                PaneOrder::EditorFirst,
            );
        });

        let preview_rect = egui::containers::panel::PanelState::load(
            &ctx,
            crate::views::panels::preview::preview_panel_id(
                Some(path.as_path()),
                "preview_panel_h_right",
            ),
        )
        .expect("preview panel rect")
        .rect;
        let shapes = flatten_shapes(output.shapes.iter());
        let text_shape = shapes
            .iter()
            .find_map(|shape| match shape {
                egui::epaint::Shape::Text(text)
                    if text.galley.job.text.contains(&"\u{3042}".repeat(60)) =>
                {
                    Some(text)
                }
                _ => None,
            })
            .expect("long inline code text shape");

        assert!(
            text_shape.galley.rows.len() > 1,
            "long inline code must wrap instead of staying on a single row"
        );
        assert!(
            text_shape.visual_bounding_rect().right()
                <= preview_rect.right() - PREVIEW_CONTENT_PADDING + 4.0,
            "wrapped inline code must stay within the preview panel"
        );
    }

    #[test]
    fn split_preview_wraps_long_markdown_with_mixed_inline_styles() {
        let ctx = test_context();
        let path = PathBuf::from("/tmp/blockquote-strong.md");
        let markdown = concat!(
            "> **Note:** On macOS Sequoia (15.x), Gatekeeper requires this command for apps not notarized with Apple. ",
            "Alternatively, go to System Settings -> Privacy & Security -> \"Open Anyway\" after the first launch attempt.\n"
        );
        let mut app = app_with_preview_doc(&path, markdown);

        let output = ctx.run(test_input(egui::vec2(900.0, 700.0)), |ctx| {
            crate::views::layout::split::render_horizontal_split(
                ctx,
                &mut app,
                PaneOrder::EditorFirst,
            );
        });

        let preview_rect = egui::containers::panel::PanelState::load(
            &ctx,
            crate::views::panels::preview::preview_panel_id(
                Some(path.as_path()),
                "preview_panel_h_right",
            ),
        )
        .expect("preview panel rect")
        .rect;
        let shapes = flatten_shapes(output.shapes.iter());
        let text_shapes: Vec<&egui::epaint::TextShape> = shapes
            .iter()
            .filter_map(|shape| match shape {
                egui::epaint::Shape::Text(text)
                    if text.galley.job.text.contains("Note:")
                        || text.galley.job.text.contains("Gatekeeper requires") =>
                {
                    Some(text)
                }
                _ => None,
            })
            .collect();

        assert!(
            !text_shapes.is_empty(),
            "expected mixed-style blockquote text shapes"
        );

        let max_right = text_shapes
            .iter()
            .map(|text| text.visual_bounding_rect().right())
            .fold(f32::NEG_INFINITY, f32::max);
        let max_rows = text_shapes
            .iter()
            .map(|text| text.galley.rows.len())
            .max()
            .unwrap_or(0);

        assert!(
            max_rows > 1,
            "mixed-style blockquote must wrap to multiple rows"
        );
        assert!(
            max_right <= preview_rect.right() - PREVIEW_CONTENT_PADDING + 4.0,
            "mixed-style blockquote must stay within preview width, got right edge {max_right}"
        );
    }

    // ── TDD(RED): Vertical split must leave sufficient height for editor scrolling ──

    /// When the split direction is vertical (top/bottom), the editor's
    /// CentralPanel must occupy at least 30% of the total height so that
    /// the TextEdit inside can scroll.
    ///
    /// The bug: `render_preview_content` calls `allocate_rect(outer_rect)` which
    /// consumes the full available height of the TopBottomPanel. Combined with
    /// no `max_height` constraint, the preview panel grows beyond its `default_height`,
    /// starving the CentralPanel.
    #[test]
    fn vertical_split_editor_has_sufficient_height_for_scrolling() {
        let ctx = test_context();
        let active = PathBuf::from("/tmp/vsplit_scroll.md");
        let long_content = (0..100).map(|i| format!("Line {i}\n")).collect::<String>();
        let mut app = app_with_preview_doc(&active, &long_content);
        let total_height = 800.0_f32;

        // Run 3 frames for layout stabilization
        for _ in 0..3 {
            let _ = ctx.run(test_input(egui::vec2(1200.0, total_height)), |ctx| {
                crate::views::layout::split::render_vertical_split(
                    ctx,
                    &mut app,
                    PaneOrder::EditorFirst,
                );
            });
        }

        let preview_rect = egui::containers::panel::PanelState::load(
            &ctx,
            crate::views::panels::preview::preview_panel_id(
                Some(active.as_path()),
                "preview_panel_v_bottom",
            ),
        )
        .expect("preview panel rect")
        .rect;

        // The preview panel should not consume more than 70% of the total height.
        // The remaining >= 30% is the editor's CentralPanel.
        let editor_height = total_height - preview_rect.height();
        let min_editor_ratio = 0.30;

        assert!(
            editor_height >= total_height * min_editor_ratio,
            "Editor panel in vertical split must have at least {:.0}% of total height for scrolling. \
             Got editor_height={editor_height:.1}, preview_height={:.1}, total={total_height:.1}",
            min_editor_ratio * 100.0,
            preview_rect.height(),
        );
    }

    // ── TDD(RED): Bidirectional scroll sync in vertical split ──
    //
    // Scenario 3: Scroll sync works bidirectionally in vertical split.
    // Scenario 5: Scroll sync works bidirectionally after order swap.

    /// When the editor reports a scroll (scroll_source=Editor, fraction=0.5),
    /// the preview must consume it within the next frame, transitioning
    /// scroll_source to Neither. This verifies editor→preview sync works.
    #[test]
    fn vertical_split_editor_to_preview_scroll_sync() {
        let ctx = test_context();
        let active = PathBuf::from("/tmp/vsplit_sync_e2p.md");
        let long_content = (0..100).map(|i| format!("Line {i}\n")).collect::<String>();
        let mut app = app_with_preview_doc(&active, &long_content);

        // Stabilize layout (5 frames)
        for _ in 0..5 {
            let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
                crate::views::layout::split::render_vertical_split(
                    ctx,
                    &mut app,
                    PaneOrder::EditorFirst,
                );
            });
        }

        // Simulate editor scroll by setting scroll state
        app.state.scroll.fraction = 0.5;
        app.state.scroll.source = ScrollSource::Editor;

        // Run 3 frames for sync to propagate
        for _ in 0..3 {
            let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
                crate::views::layout::split::render_vertical_split(
                    ctx,
                    &mut app,
                    PaneOrder::EditorFirst,
                );
            });
        }

        // After sync, scroll_source must settle to Neither.
        // If it bounces to Preview, the sync is creating an oscillation loop.
        assert_eq!(
            app.state.scroll.source,
            ScrollSource::Neither,
            "Editor→Preview sync must settle to Neither after consumption. \
             Got {:?}, fraction={:.4}",
            app.state.scroll.source,
            app.state.scroll.fraction,
        );
    }

    /// Same editor→preview sync test for horizontal split — expected to PASS.
    #[test]
    fn horizontal_split_editor_to_preview_scroll_sync() {
        let ctx = test_context();
        let active = PathBuf::from("/tmp/hsplit_sync_e2p.md");
        let long_content = (0..100).map(|i| format!("Line {i}\n")).collect::<String>();
        let mut app = app_with_preview_doc(&active, &long_content);

        for _ in 0..5 {
            let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
                crate::views::layout::split::render_horizontal_split(
                    ctx,
                    &mut app,
                    PaneOrder::EditorFirst,
                );
            });
        }

        app.state.scroll.fraction = 0.5;
        app.state.scroll.source = ScrollSource::Editor;

        for _ in 0..3 {
            let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
                crate::views::layout::split::render_horizontal_split(
                    ctx,
                    &mut app,
                    PaneOrder::EditorFirst,
                );
            });
        }

        assert_eq!(
            app.state.scroll.source,
            ScrollSource::Neither,
            "Editor→Preview sync must settle to Neither in horizontal split. \
             Got {:?}, fraction={:.4}",
            app.state.scroll.source,
            app.state.scroll.fraction,
        );
    }

    /// Scenario 5: After swapping order (PreviewFirst), the same
    /// editor→preview sync must work in vertical split.
    #[test]
    fn vertical_split_editor_to_preview_scroll_sync_after_swap() {
        let ctx = test_context();
        let active = PathBuf::from("/tmp/vsplit_sync_swap.md");
        let long_content = (0..100).map(|i| format!("Line {i}\n")).collect::<String>();
        let mut app = app_with_preview_doc(&active, &long_content);

        // Use PreviewFirst (swapped order)
        for _ in 0..5 {
            let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
                crate::views::layout::split::render_vertical_split(
                    ctx,
                    &mut app,
                    PaneOrder::PreviewFirst,
                );
            });
        }

        app.state.scroll.fraction = 0.5;
        app.state.scroll.source = ScrollSource::Editor;

        for _ in 0..3 {
            let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
                crate::views::layout::split::render_vertical_split(
                    ctx,
                    &mut app,
                    PaneOrder::PreviewFirst,
                );
            });
        }

        assert_eq!(
            app.state.scroll.source,
            ScrollSource::Neither,
            "Editor→Preview sync must settle to Neither after order swap. \
             Got {:?}, fraction={:.4}",
            app.state.scroll.source,
            app.state.scroll.fraction,
        );
    }

    /// Verify preview→editor sync direction also works in vertical split.
    /// Set scroll_source=Preview and verify it transitions to Neither.
    #[test]
    fn vertical_split_preview_to_editor_scroll_sync() {
        let ctx = test_context();
        let active = PathBuf::from("/tmp/vsplit_sync_p2e.md");
        let long_content = (0..100).map(|i| format!("Line {i}\n")).collect::<String>();
        let mut app = app_with_preview_doc(&active, &long_content);

        for _ in 0..5 {
            let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
                crate::views::layout::split::render_vertical_split(
                    ctx,
                    &mut app,
                    PaneOrder::EditorFirst,
                );
            });
        }

        // Simulate preview scroll
        app.state.scroll.fraction = 0.5;
        app.state.scroll.source = ScrollSource::Preview;

        for _ in 0..3 {
            let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
                crate::views::layout::split::render_vertical_split(
                    ctx,
                    &mut app,
                    PaneOrder::EditorFirst,
                );
            });
        }

        assert_eq!(
            app.state.scroll.source,
            ScrollSource::Neither,
            "Preview→Editor sync must settle to Neither in vertical split. \
             Got {:?}, fraction={:.4}",
            app.state.scroll.source,
            app.state.scroll.fraction,
        );
    }

    #[test]
    fn refresh_diagrams_update_clears_image_caches() {
        let ctx = test_context();
        let mut frame = eframe::Frame::_new_kittest();
        let path = PathBuf::from("/tmp/refresh-cache.md");
        let mut app = app_with_preview_doc(&path, "# Refresh cache");
        let forget_all_calls = Arc::new(AtomicUsize::new(0));

        ctx.add_bytes_loader(Arc::new(CountingBytesLoader {
            forget_all_calls: Arc::clone(&forget_all_calls),
        }));
        app.pending_action = AppAction::RefreshDiagrams;

        let _ = ctx.run(test_input(egui::vec2(1200.0, 800.0)), |ctx| {
            app.update(ctx, &mut frame);
        });

        assert_eq!(
            forget_all_calls.load(Ordering::SeqCst),
            1,
            "RefreshDiagrams must clear image caches before rerendering preview"
        );
    }
}
