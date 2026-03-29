use crate::markdown::color_preset::DiagramColorPreset;
use crate::markdown::{render, DiagramRenderer, MarkdownError};

/// Exporter for generating standalone HTML documents.
pub struct HtmlExporter;

impl HtmlExporter {
    /// Exports Markdown as a standalone HTML document with embedded CSS matching the given preset.
    ///
    /// When `base_dir` is provided, relative image paths in the rendered HTML are
    /// resolved to absolute `file://` URLs so that images display correctly even
    /// when the HTML is opened from a different directory (e.g. a temp file).
    pub fn export<R: DiagramRenderer>(
        source: &str,
        renderer: &R,
        preset: &DiagramColorPreset,
        base_dir: Option<&std::path::Path>,
    ) -> Result<String, MarkdownError> {
        let output = render(source, renderer)?;
        let bg_color = Self::get_bg_color(preset);
        let props = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif, 'Apple Color Emoji', 'Segoe UI Emoji'";
        let monos = "SFMono-Regular, Consolas, 'Liberation Mono', Menlo, monospace";
        
        let css = Self::generate_css(preset, bg_color, props, monos);
        let body = match base_dir {
            Some(dir) => Self::resolve_relative_paths(&output.html, dir),
            None => output.html,
        };

        Ok(Self::assemble_html_document(&css, &body))
    }

    fn assemble_html_document(css: &str, body: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Exported Document</title>
<style>
{css}
</style>
</head>
<body>
{body}
</body>
</html>"#
        )
    }

    /// Resolves relative `src` attributes in `<img>` tags to absolute `file://` URLs.
    fn resolve_relative_paths(html: &str, base_dir: &std::path::Path) -> String {
        // WHY: Match src="..." that don't start with http://, https://, data:, or file://
        let re = regex::Regex::new(r#"src="([^"]+)""#).unwrap();
        re.replace_all(html, |caps: &regex::Captures| {
            let src = &caps[1];
            if src.starts_with("http://")
                || src.starts_with("https://")
                || src.starts_with("data:")
                || src.starts_with("file://")
            {
                caps[0].to_string()
            } else {
                let abs = base_dir.join(src);
                format!("src=\"file://{}\"", abs.display())
            }
        })
        .to_string()
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

    fn generate_css(preset: &DiagramColorPreset, bg_color: &str, props: &str, monos: &str) -> String {
        let base = Self::generate_base_css(preset, bg_color, props, monos);
        let elems = Self::generate_elements_css(preset, bg_color);
        format!("{base}{elems}")
    }

    fn generate_base_css(preset: &DiagramColorPreset, bg_color: &str, props: &str, monos: &str) -> String {
        format!(
            r#"
body {{ font-family: {props}; background-color: {bg_color}; color: {text}; line-height: 1.6; max-width: 900px; margin: 0 auto; padding: 2rem; }}
h1, h2, h3, h4, h5, h6 {{ margin-top: 1.5em; margin-bottom: 0.5em; font-weight: 600; }}
h1 {{ border-bottom: 1px solid {stroke}; padding-bottom: 0.3em; }}
h2 {{ border-bottom: 1px solid {stroke}; padding-bottom: 0.3em; }}
a {{ color: #0366d6; text-decoration: none; }}
pre {{ background-color: {fill}; border: 1px solid {stroke}; border-radius: 6px; padding: 16px; overflow: auto; line-height: 1.5; }}
code {{ font-family: {monos}; background-color: {fill}; border-radius: 3px; padding: 0.2em 0.4em; font-size: 85%; }}
pre code {{ background-color: transparent; padding: 0; }}
"#,
            props = props, bg_color = bg_color, text = preset.text, stroke = preset.stroke, fill = preset.fill, monos = monos
        )
    }

    fn generate_elements_css(preset: &DiagramColorPreset, bg_color: &str) -> String {
        format!(
            r#"
blockquote {{ border-left: 0.25em solid {stroke}; color: {text}; opacity: 0.8; padding: 0 1em; margin: 0; }}
table {{ border-spacing: 0; border-collapse: collapse; margin-top: 0; margin-bottom: 16px; }}
table th, table td {{ padding: 6px 13px; border: 1px solid {stroke}; }}
img {{ max-width: 100%; box-sizing: content-box; background-color: {bg_color}; }}
.katana-diagram img {{ background-color: transparent; }}
hr {{ height: 0.25em; padding: 0; margin: 24px 0; background-color: {stroke}; border: 0; }}
"#,
            bg_color = bg_color, text = preset.text, stroke = preset.stroke
        )
    }
}
