use xmltree::Element;

use super::utils::{attr_f64, CANVAS_EDGE_MARGIN, CANVAS_MIN_HEIGHT, CANVAS_MIN_WIDTH};

pub fn extract_graph_model(root: &Element) -> Result<&Element, String> {
    if root.name == "mxGraphModel" {
        return Ok(root);
    }
    if root.name == "mxfile" {
        let diagram = root
            .get_child("diagram")
            .ok_or("<diagram> element not found")?;
        return diagram
            .get_child("mxGraphModel")
            .ok_or("<mxGraphModel> element not found".to_string());
    }
    Err(format!("Unsupported root element: {}", root.name))
}

pub fn collect_cells(model: &Element) -> Vec<&Element> {
    let root = match model.get_child("root") {
        Some(r) => r,
        None => return Vec::new(),
    };
    root.children
        .iter()
        .filter_map(|node| node.as_element())
        .filter(|el| el.name == "mxCell")
        .collect()
}

pub fn estimate_canvas_size(cells: &[&Element]) -> (f64, f64) {
    let (mut max_x, mut max_y) = (CANVAS_MIN_WIDTH, CANVAS_MIN_HEIGHT);
    for cell in cells {
        if let Some(geo) = cell.get_child("mxGeometry") {
            let x: f64 = attr_f64(geo, "x");
            let y: f64 = attr_f64(geo, "y");
            let w: f64 = attr_f64(geo, "width");
            let h: f64 = attr_f64(geo, "height");
            max_x = max_x.max(x + w + CANVAS_EDGE_MARGIN);
            max_y = max_y.max(y + h + CANVAS_EDGE_MARGIN);
        }
    }
    (max_x, max_y)
}
