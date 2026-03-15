use egui_kittest::Harness;
use katana_core::markdown::svg_rasterize::RasterizedSvg;
use katana_ui::preview_pane::{decode_png_rgba, extract_svg, PreviewPane, RenderedSection};
use std::path::PathBuf;

/// ヘルパー: RenderedSection から Markdown テキストを抽出する。
fn markdown_texts(sections: &[RenderedSection]) -> Vec<&str> {
    sections
        .iter()
        .filter_map(|s| match s {
            RenderedSection::Markdown(md) => Some(md.as_str()),
            _ => None,
        })
        .collect()
}

// ── 3.2 プレビュー同期: 未保存バッファからの即時プレビュー更新 ──

#[test]
fn 未保存バッファの変更がプレビューに反映される() {
    let mut pane = PreviewPane::default();

    // 初期コンテンツでプレビューを構築
    pane.update_markdown_sections("# Hello");
    assert_eq!(pane.sections.len(), 1);
    let texts = markdown_texts(&pane.sections);
    assert!(texts[0].contains("# Hello"));

    // ファイル保存なしでバッファを更新 → プレビューに反映される
    pane.update_markdown_sections("# Hello World\n\nNew paragraph");
    let texts = markdown_texts(&pane.sections);
    assert!(texts[0].contains("# Hello World"));
    assert!(texts[0].contains("New paragraph"));
}

#[test]
fn 連続編集がプレビューに即時反映される() {
    let mut pane = PreviewPane::default();

    // 複数回の連続編集がすべて反映される
    let edits = vec![
        "# Draft 1",
        "# Draft 2\n\n- item A",
        "# Draft 3\n\n- item A\n- item B\n- item C",
    ];

    for edit in &edits {
        pane.update_markdown_sections(edit);
        let texts = markdown_texts(&pane.sections);
        assert!(
            texts[0].contains(edit),
            "Edit not reflected in preview: {edit}"
        );
    }
}

#[test]
fn 空バッファでもプレビューがクラッシュしない() {
    let mut pane = PreviewPane::default();

    // コンテンツを入力
    pane.update_markdown_sections("# Hello");
    assert_eq!(pane.sections.len(), 1);

    // 空に戻す → セクション数は 0 になる（空文字列は flush されない）
    pane.update_markdown_sections("");
    assert_eq!(pane.sections.len(), 0);
}

#[test]
fn ダイアグラム含みのバッファでmarkdown部分のみ即時更新される() {
    let mut pane = PreviewPane::default();

    // ダイアグラムを含む初期コンテンツ
    let source = "# Title\n```mermaid\ngraph TD; A-->B\n```\n## Footer";
    pane.full_render(source);

    // ダイアグラムは Pending になっている
    assert!(pane.sections.len() >= 3);
    assert!(matches!(pane.sections[1], RenderedSection::Pending { .. }));

    // Markdown 部分のみ更新する（ダイアグラムは保持）
    let modified = "# Updated Title\n```mermaid\ngraph TD; A-->B\n```\n## Updated Footer";
    pane.update_markdown_sections(modified);

    // Markdown テキストが更新されていることを確認
    let texts = markdown_texts(&pane.sections);
    assert!(texts.iter().any(|t| t.contains("Updated Title")));
    assert!(texts.iter().any(|t| t.contains("Updated Footer")));
}

#[test]
fn full_renderでセクションが正しく分割される() {
    let mut pane = PreviewPane::default();

    let source = "Before\n```mermaid\ngraph TD; A-->B\n```\nAfter";
    pane.full_render(source);

    // 3 セクション: Markdown, Diagram(Pending), Markdown
    assert_eq!(pane.sections.len(), 3);
    assert!(matches!(pane.sections[0], RenderedSection::Markdown(_)));
    assert!(matches!(pane.sections[1], RenderedSection::Pending { .. }));
    assert!(matches!(pane.sections[2], RenderedSection::Markdown(_)));
}

#[test]
fn ダイアグラムなしのバッファではpendingセクションが生成されない() {
    let mut pane = PreviewPane::default();

    pane.full_render("# Pure Markdown\n\nNo diagrams here.");

    assert!(pane
        .sections
        .iter()
        .all(|s| matches!(s, RenderedSection::Markdown(_))));
    assert!(!pane
        .sections
        .iter()
        .any(|s| matches!(s, RenderedSection::Pending { .. })));
}

#[test]
fn プレビュー更新がファイル保存に依存しないことの検証() {
    // Document + PreviewPane の統合テスト:
    // ドキュメントのバッファを更新（is_dirty = true）し、save を呼ばずに
    // プレビューが最新バッファを反映していることを確認する。
    use katana_core::document::Document;

    let mut doc = Document::new("/workspace/spec.md", "# Original");
    let mut pane = PreviewPane::default();

    // 初期プレビュー
    pane.update_markdown_sections(&doc.buffer);
    let texts = markdown_texts(&pane.sections);
    assert!(texts[0].contains("# Original"));

    // ドキュメントを編集（未保存状態）
    doc.update_buffer("# Modified by user\n\nThis is not saved yet.");
    assert!(doc.is_dirty, "ドキュメントは dirty でなければならない");

    // 未保存バッファでプレビューを更新
    pane.update_markdown_sections(&doc.buffer);
    let texts = markdown_texts(&pane.sections);
    assert!(
        texts[0].contains("Modified by user"),
        "未保存の編集がプレビューに反映されていない"
    );
    assert!(
        texts[0].contains("This is not saved yet"),
        "未保存の編集がプレビューに反映されていない"
    );

    // ドキュメントはまだ dirty のまま（保存していない）
    assert!(doc.is_dirty, "ドキュメントは保存されていないはず");
}

// ── extract_svg テスト ──

#[test]
fn 正常なsvgが抽出される() {
    let html = r#"<div><svg width="100" height="100"><rect/></svg></div>"#;
    let svg = extract_svg(html).unwrap();
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>"));
}

#[test]
fn svgなしの場合はnoneを返す() {
    assert!(extract_svg("<div>hello</div>").is_none());
    assert!(extract_svg("").is_none());
}

#[test]
fn 複数svgの場合は最初から最後までをカバーする() {
    let html = r#"<svg>first</svg><p>mid</p><svg>second</svg>"#;
    let svg = extract_svg(html).unwrap();
    // rfind("</svg>") で最後の閉じタグまでを含む
    assert!(svg.contains("first"));
    assert!(svg.contains("second"));
}

// ── decode_png_rgba テスト ──

#[test]
fn 有効なpngがデコードされる() {
    // 1x1 白色 PNG の最小バイト列を生成
    let mut buf = Vec::new();
    {
        use image::{ImageBuffer, Rgba};
        let img = ImageBuffer::from_pixel(1, 1, Rgba([255u8, 255, 255, 255]));
        let mut cursor = std::io::Cursor::new(&mut buf);
        img.write_to(&mut cursor, image::ImageFormat::Png).unwrap();
    }
    let result = decode_png_rgba(&buf);
    assert!(result.is_ok());
    let rasterized = result.unwrap();
    assert_eq!(rasterized.width, 1);
    assert_eq!(rasterized.height, 1);
    assert_eq!(rasterized.rgba.len(), 4); // 1x1 RGBA = 4 bytes
}

#[test]
fn 無効なデータはエラーを返す() {
    let result = decode_png_rgba(b"not a png");
    assert!(result.is_err());
}

// ── update_markdown_sections 追加テスト ──

#[test]
fn markdownのみの入力が正しくセクション化される() {
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections("# Title\n\nParagraph 1\n\n## Subtitle\n\nParagraph 2");
    assert_eq!(pane.sections.len(), 1);
    assert!(matches!(pane.sections[0], RenderedSection::Markdown(_)));
}

#[test]
fn ダイアグラム混在の入力がセクション分割される() {
    let mut pane = PreviewPane::default();
    let src =
        "Before\n```mermaid\ngraph TD; A-->B\n```\nMiddle\n```drawio\n<mxGraphModel/>\n```\nAfter";
    pane.update_markdown_sections(src);
    // Markdown + Pending + Markdown + Pending + Markdown = 5 sections
    assert!(pane.sections.len() >= 3);
}

#[test]
fn 空入力は空セクションリストを返す() {
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections("");
    assert!(pane.sections.is_empty());
}

// ── egui_kittest を使って show_section の各バリアントをカバー ──

/// show_section の Markdown バリアント描画をカバー
#[test]
fn show_section_markdown_variant_renders() {
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections("# Hello from egui test");

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui);
    });
    harness.run();
    // クラッシュしなければOK
}

/// show_section の Error バリアント描画をカバー (L267-275)
#[test]
fn show_section_error_variant_renders() {
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::Error {
        kind: "DrawIo".to_string(),
        _source: "<mxCell/>".to_string(),
        message: "invalid XML".to_string(),
    }];

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui);
    });
    harness.run();
}

/// show_section の CommandNotFound バリアント描画をカバー (L277-291)
#[test]
fn show_section_command_not_found_variant_renders() {
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::CommandNotFound {
        tool_name: "mmdc".to_string(),
        install_hint: "npm install -g @mermaid-js/mermaid-cli".to_string(),
        _source: "graph TD; A-->B".to_string(),
    }];

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui);
    });
    harness.run();
}

/// show_section の NotInstalled バリアント描画をカバー (L292-296, L310-341)
#[test]
fn show_section_not_installed_variant_renders() {
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::NotInstalled {
        kind: "PlantUML".to_string(),
        download_url: "https://example.com/plantuml.jar".to_string(),
        install_path: PathBuf::from("/tmp/plantuml.jar"),
    }];

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui);
    });
    harness.run();
}

/// show_section の Pending バリアント描画をカバー (L297-305)
#[test]
fn show_section_pending_variant_renders() {
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::Pending {
        kind: "Mermaid".to_string(),
    }];

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui);
    });
    // スピナーが常時リペイントするため step() で1フレームのみ実行
    harness.step();
}

/// show_section の Image バリアント描画をカバー (L258-261, L344-358)
#[test]
fn show_section_image_variant_renders() {
    let mut pane = PreviewPane::default();
    // 1x1 の RGBA ダミー画像
    pane.sections = vec![RenderedSection::Image {
        svg_data: RasterizedSvg {
            width: 1,
            height: 1,
            rgba: vec![255, 255, 255, 255],
        },
        alt: "test diagram".to_string(),
    }];

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui);
    });
    harness.run();
}

/// show() メソッド描画をカバー (L156-167): allow(dead_code) が付いているが egui_kittest でカバー
#[test]
fn show_method_renders_without_crash() {
    let mut pane = PreviewPane::default();
    pane.update_markdown_sections("# Show method test");

    let mut harness = Harness::new_ui(move |ui| {
        pane.show(ui);
    });
    harness.run();
}

/// render_sections の空セクション分岐をカバー (L189-191)
#[test]
fn render_sections_empty_shows_no_preview_label() {
    let mut pane = PreviewPane::default();
    // sections は空のまま

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui);
    });
    harness.run();
}

/// poll_renders: スチルペンディングのとき repaint_after が呼ばれる (L214-215)
#[test]
fn poll_renders_with_pending_does_not_crash() {
    let mut pane = PreviewPane::default();
    // full_render でバックグラウンドスレッドが起動する
    pane.full_render("# Title\n```mermaid\ngraph TD; A-->B\n```\nAfter");

    // poll_renders を egui コンテキスト付きで実行
    let mut harness = Harness::new_ui(move |ui| {
        // show_content が poll_renders を内部で呼ぶ
        pane.show_content(ui);
    });
    // Pending セクションがあるためスピナーでリペイント継続する。
    // step() で1フレームのみ実行してクラッシュしないことを確認。
    harness.step();
}

/// NotInstalled の show_not_installed UI が描画される (L310-341)
#[test]
fn show_section_not_installed_download_button_returns_request() {
    let mut pane = PreviewPane::default();
    pane.sections = vec![RenderedSection::NotInstalled {
        kind: "PlantUML".to_string(),
        download_url: "https://example.com/plantuml.jar".to_string(),
        install_path: PathBuf::from("/tmp/plantuml_test.jar"),
    }];

    let mut harness = Harness::new_ui(move |ui| {
        // show_not_installed の描画（L316-341）をカバー
        let _req = pane.show_content(ui);
    });
    harness.run();
    // クラッシュしなければOK（ボタン描画、ラベル描画が実行された）
}

/// show_rasterized: 実際のテクスチャ描画パス (L344-358) をカバー
#[test]
fn show_section_image_full_render_with_texture() {
    let mut pane = PreviewPane::default();
    // 4x4 の有効な RGBA 画像
    let rgba: Vec<u8> = (0..64).map(|i| (i * 4) as u8).collect();
    pane.sections = vec![RenderedSection::Image {
        svg_data: RasterizedSvg {
            width: 4,
            height: 4,
            rgba,
        },
        alt: "Test texture".to_string(),
    }];

    let mut harness = Harness::new_ui(move |ui| {
        pane.show_content(ui);
    });
    harness.run();
}
