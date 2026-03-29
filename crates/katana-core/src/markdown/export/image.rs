use crate::markdown::MarkdownError;

const DEFAULT_VIEWPORT_WIDTH: f64 = 1280.0;
const DEFAULT_VIEWPORT_HEIGHT: f64 = 800.0;
const SCREENSHOT_SCALE: f64 = 2.0;

pub struct ImageExporter;

impl ImageExporter {
    pub fn export(html: &str, output: &std::path::Path) -> Result<(), MarkdownError> {
        use headless_chrome::{Browser, LaunchOptions};

        let browser = Browser::new(LaunchOptions::default())
            .map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;
        let tab = browser
            .new_tab()
            .map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;

        Self::set_viewport(&tab, DEFAULT_VIEWPORT_HEIGHT as u32)?;
        let temp = Self::write_temp_html(html)?;
        Self::navigate_to_file(&tab, temp.path())?;

        let format = Self::determine_format(output);
        let height = Self::get_document_height(&tab).unwrap_or(DEFAULT_VIEWPORT_HEIGHT as u32);
        Self::set_viewport(&tab, height)?;

        let img_data = tab
            .capture_screenshot(format, None, None, true)
            .map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;

        std::fs::write(output, img_data).map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;
        Ok(())
    }

    fn set_viewport(tab: &headless_chrome::Tab, height: u32) -> Result<(), MarkdownError> {
        use headless_chrome::protocol::cdp::Emulation;
        tab.call_method(Emulation::SetDeviceMetricsOverride {
            width: DEFAULT_VIEWPORT_WIDTH as u32,
            height,
            device_scale_factor: SCREENSHOT_SCALE,
            mobile: false,
            scale: None,
            screen_width: None,
            screen_height: None,
            position_x: None,
            position_y: None,
            dont_set_visible_size: None,
            screen_orientation: None,
            viewport: None,
            display_feature: None,
            device_posture: None,
        })
        .map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;
        Ok(())
    }

    fn write_temp_html(html: &str) -> Result<tempfile::NamedTempFile, MarkdownError> {
        use std::io::Write;
        let mut temp = tempfile::Builder::new()
            .prefix("katana_export_src_")
            .suffix(".html")
            .tempfile()
            .map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;
        temp.write_all(html.as_bytes())
            .map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;
        Ok(temp)
    }

    fn navigate_to_file(
        tab: &headless_chrome::Tab,
        path: &std::path::Path,
    ) -> Result<(), MarkdownError> {
        let url = format!("file://{}", path.display());
        tab.navigate_to(&url)
            .map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;
        tab.wait_until_navigated()
            .map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;
        Ok(())
    }

    fn determine_format(
        output: &std::path::Path,
    ) -> headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption {
        use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
        let ext = output
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();
        if ext == "jpg" || ext == "jpeg" {
            CaptureScreenshotFormatOption::Jpeg
        } else {
            CaptureScreenshotFormatOption::Png
        }
    }

    fn get_document_height(tab: &headless_chrome::Tab) -> Option<u32> {
        let dimensions = tab.evaluate(
            "JSON.stringify({width: Math.max(document.body.scrollWidth, document.documentElement.scrollWidth), height: Math.max(document.body.scrollHeight, document.documentElement.scrollHeight)})",
            false,
        ).ok()?;

        if let Some(serde_json::Value::String(json_str)) = dimensions.value {
            let Ok(dims) = serde_json::from_str::<serde_json::Value>(&json_str) else {
                return None;
            };
            return Some(dims["height"].as_f64()? as u32);
        }
        None
    }
}
