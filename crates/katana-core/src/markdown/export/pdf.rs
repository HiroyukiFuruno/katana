use crate::markdown::MarkdownError;

/// Exporter for generating PDF documents via Headless Chrome.
pub struct PdfExporter;

impl PdfExporter {
    /// Returns true if Headless Chrome can be initialized.
    pub fn is_available() -> bool {
        // WHY: We assume it's available as headless_chrome can download a browser.
        true
    }

    /// Exports the given HTML content to a PDF file at the specified path.
    pub fn export(html: &str, output: &std::path::Path) -> Result<(), MarkdownError> {
        use headless_chrome::{Browser, LaunchOptions};

        let browser = Browser::new(LaunchOptions::default()).map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;
        let tab = browser.new_tab().map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;

        let temp = Self::write_temp_html(html)?;
        Self::navigate_to_file(&tab, temp.path())?;

        let pdf_options = headless_chrome::types::PrintToPdfOptions {
            print_background: Some(true),
            ..Default::default()
        };

        let pdf_data = tab.print_to_pdf(Some(pdf_options)).map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;
        std::fs::write(output, pdf_data).map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;

        Ok(())
    }

    fn write_temp_html(html: &str) -> Result<tempfile::NamedTempFile, MarkdownError> {
        use std::io::Write;
        let mut temp = tempfile::Builder::new().prefix("katana_export_src_").suffix(".html").tempfile()
            .map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;
        temp.write_all(html.as_bytes()).map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;
        Ok(temp)
    }

    fn navigate_to_file(tab: &headless_chrome::Tab, path: &std::path::Path) -> Result<(), MarkdownError> {
        let url = format!("file://{}", path.display());
        tab.navigate_to(&url).map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;
        tab.wait_until_navigated().map_err(|e| MarkdownError::ExportFailed(e.to_string()))?;
        Ok(())
    }
}
