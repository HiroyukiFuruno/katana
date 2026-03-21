use crate::markdown::color_preset::DiagramColorPreset;
use crate::markdown::{render, DiagramRenderer, MarkdownError};

/// Exporter for generating standalone HTML documents.
pub struct HtmlExporter;

impl HtmlExporter {
    /// Exports Markdown as a standalone HTML document with embedded CSS matching the given preset.
    pub fn export<R: DiagramRenderer>(
        source: &str,
        renderer: &R,
        preset: &DiagramColorPreset,
    ) -> Result<String, MarkdownError> {
        let output = render(source, renderer)?;

        let bg_color = Self::get_bg_color(preset);

        let props = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif, 'Apple Color Emoji', 'Segoe UI Emoji'";
        let monos = "SFMono-Regular, Consolas, 'Liberation Mono', Menlo, monospace";

        let css = Self::generate_css(preset, bg_color, props, monos);

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Exported Document</title>
<style>
{}
</style>
</head>
<body>
{}
</body>
</html>"#,
            css, output.html
        );

        Ok(html)
    }

    fn get_bg_color(preset: &DiagramColorPreset) -> &str {
        if preset.background == "transparent" {
            if preset.text == "#E0E0E0" {
                "#1e1e1e"
            } else {
                "#ffffff"
            }
        } else {
            preset.background
        }
    }

    fn generate_css(
        preset: &DiagramColorPreset,
        bg_color: &str,
        props: &str,
        monos: &str,
    ) -> String {
        format!(
            r#"
body {{
    font-family: {props};
    background-color: {bg_color};
    color: {text};
    line-height: 1.6;
    max-width: 900px;
    margin: 0 auto;
    padding: 2rem;
}}
h1, h2, h3, h4, h5, h6 {{
    margin-top: 1.5em;
    margin-bottom: 0.5em;
    font-weight: 600;
}}
h1 {{ border-bottom: 1px solid {stroke}; padding-bottom: 0.3em; }}
h2 {{ border-bottom: 1px solid {stroke}; padding-bottom: 0.3em; }}
a {{ color: #0366d6; text-decoration: none; }}
pre {{
    background-color: {fill};
    border: 1px solid {stroke};
    border-radius: 6px;
    padding: 16px;
    overflow: auto;
}}
code {{
    font-family: {monos};
    background-color: {fill};
    border-radius: 3px;
    padding: 0.2em 0.4em;
    font-size: 85%;
}}
pre code {{ background-color: transparent; padding: 0; }}
blockquote {{ border-left: 0.25em solid {stroke}; color: {text}; opacity: 0.8; padding: 0 1em; margin: 0; }}
table {{ border-spacing: 0; border-collapse: collapse; margin-top: 0; margin-bottom: 16px; }}
table th, table td {{ padding: 6px 13px; border: 1px solid {stroke}; }}
img {{ max-width: 100%; box-sizing: content-box; background-color: #fff; }}
hr {{ height: 0.25em; padding: 0; margin: 24px 0; background-color: {stroke}; border: 0; }}
            "#,
            props = props,
            bg_color = bg_color,
            text = preset.text,
            stroke = preset.stroke,
            fill = preset.fill,
            monos = monos,
        )
    }
}

/// Exporter for generating PDF documents via external tools.
pub struct PdfExporter;

impl PdfExporter {
    /// Returns true if the required tool (`wkhtmltopdf`) is installed on the system.
    pub fn is_available() -> bool {
        std::process::Command::new("wkhtmltopdf")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|it| it.success())
            .unwrap_or(false)
    }

    /// Exports the given HTML content to a PDF file at the specified path.
    pub fn export(html: &str, output: &std::path::Path) -> Result<(), MarkdownError> {
        use std::io::Write;
        let mut temp = tempfile::Builder::new()
            .prefix("katana_pdf_src_")
            .suffix(".html")
            .tempfile()
            .map_err(|it| MarkdownError::ExportFailed(it.to_string()))?;

        temp.write_all(html.as_bytes())
            .map_err(|it| MarkdownError::ExportFailed(it.to_string()))?;

        let status = std::process::Command::new("wkhtmltopdf")
            .arg("-q")
            .arg(temp.path())
            .arg(output)
            .status()
            .map_err(|it| MarkdownError::ExportFailed(it.to_string()))?;

        if !status.success() {
            return Err(MarkdownError::ExportFailed(
                "wkhtmltopdf returned non-zero".to_string(),
            ));
        }

        Ok(())
    }
}

/// Exporter for generating image files (PNG/JPG) via external tools.
pub struct ImageExporter;

impl ImageExporter {
    /// Returns true if the required tool (`wkhtmltoimage`) is installed on the system.
    pub fn is_available() -> bool {
        std::process::Command::new("wkhtmltoimage")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|it| it.success())
            .unwrap_or(false)
    }

    /// Exports the given HTML content to an image file at the specified path.
    pub fn export(html: &str, output: &std::path::Path) -> Result<(), MarkdownError> {
        use std::io::Write;
        let mut temp = tempfile::Builder::new()
            .prefix("katana_img_src_")
            .suffix(".html")
            .tempfile()
            .map_err(|it| MarkdownError::ExportFailed(it.to_string()))?;

        temp.write_all(html.as_bytes())
            .map_err(|it| MarkdownError::ExportFailed(it.to_string()))?;

        let status = std::process::Command::new("wkhtmltoimage")
            .arg("-q")
            .arg(temp.path())
            .arg(output)
            .status()
            .map_err(|it| MarkdownError::ExportFailed(it.to_string()))?;

        if !status.success() {
            return Err(MarkdownError::ExportFailed(
                "wkhtmltoimage returned non-zero".to_string(),
            ));
        }

        Ok(())
    }
}
