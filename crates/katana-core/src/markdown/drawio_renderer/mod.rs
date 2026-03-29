use xmltree::Element;

use super::diagram::{DiagramBlock, DiagramResult};

pub mod edge;
pub mod parse;
pub mod svg;
pub mod utils;
pub mod vertex;

pub fn render_drawio(block: &DiagramBlock) -> DiagramResult {
    match convert_xml_to_svg(&block.source) {
        Ok(svg) => DiagramResult::Ok(format!(r#"<div class="katana-diagram drawio">{svg}</div>"#)),
        Err(e) => DiagramResult::Err {
            source: block.source.clone(),
            error: e,
        },
    }
}

fn convert_xml_to_svg(xml: &str) -> Result<String, String> {
    let root = Element::parse(xml.as_bytes()).map_err(|e| format!("XML parse error: {e}"))?;
    let model = parse::extract_graph_model(&root)?;
    let cells = parse::collect_cells(model);
    let (w, h) = parse::estimate_canvas_size(&cells);
    Ok(svg::build_svg(&cells, w, h))
}
