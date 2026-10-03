use crate::anchor::{resolve_anchor, Anchor, AnchorConstraints, Rect, Vec2};
use crate::gpu::{GpuState, Vertex};
use gs_xml_parser::{parse, tokenize, Node};
use std::collections::BTreeMap;
use winit::dpi::PhysicalSize;

pub fn build_ui(xml_path: &str, window_size: PhysicalSize<u32>) -> (Vec<Vertex>, Vec<UiElement>) {
    let xml = std::fs::read_to_string(xml_path).expect("failed to read ui xml");
    let tokens = tokenize(&xml).expect("tokenize failed");
    let nodes = parse(&tokens).expect("parse failed");

    if window_size.width == 0 || window_size.height == 0 {
        return (Vec::new(), Vec::new());
    }

    let parent = Rect {
        x: 0.0,
        y: 0.0,
        width: window_size.width as f32,
        height: window_size.height as f32,
    };
    let mut vertices = Vec::new();
    let mut elements = Vec::new();
    for node in &nodes {
        collect_layout(node, parent, window_size, &mut vertices, &mut elements);
    }
    (vertices, elements)
}

fn collect_layout(
    node: &Node,
    parent: Rect,
    window_size: PhysicalSize<u32>,
    vertices: &mut Vec<Vertex>,
    elements: &mut Vec<UiElement>,
) {
    if let Node::Element {
        tag,
        attributes,
        children,
        ..
    } = node
    {
        let element_rect = resolve_element_rect(attributes, parent);
        if let Some(rect) = element_rect {
            vertices.extend(GpuState::rect_vertices_for_window(
                window_size,
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                [1.0, 1.0, 1.0],
            ));

            let bounds = Bounds {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
            };
            match tag.as_str() {
                "button" => {
                    let label = attributes.get("label").cloned().unwrap_or_default();
                    elements.push(UiElement::Button { bounds, label });
                }
                "checkbox" => {
                    let checked = attributes
                        .get("checked")
                        .is_some_and(|value| value == "true");
                    elements.push(UiElement::Checkbox { bounds, checked });
                }
                "panel" => elements.push(UiElement::Panel { bounds }),
                _ => {}
            }
        }

        let child_parent = element_rect.unwrap_or(parent);
        for child in children {
            collect_layout(child, child_parent, window_size, vertices, elements);
        }
    }
}

fn resolve_element_rect(attributes: &BTreeMap<String, String>, parent: Rect) -> Option<Rect> {
    let value = |key: &str| attributes.get(key)?.parse::<f32>().ok();
    let width = value("width")?;
    let height = value("height")?;
    let anchor = parse_anchor(attributes.get("anchor"))?;
    let pivot = match attributes.get("pivot") {
        Some(value) => parse_anchor(Some(value))?,
        None => anchor,
    };

    Some(resolve_anchor(
        parent,
        Vec2 { x: width, y: height },
        &AnchorConstraints {
            target_anchor: anchor,
            pivot,
            offset: Vec2 {
                x: value("x").unwrap_or(0.0),
                y: value("y").unwrap_or(0.0),
            },
        },
    ))
}

fn parse_anchor(value: Option<&String>) -> Option<Anchor> {
    Some(match value.map(String::as_str).unwrap_or("top-left") {
        "top-left" => Anchor::TOP_LEFT,
        "top-center" => Anchor::TOP_CENTER,
        "top-right" => Anchor::TOP_RIGHT,
        "center-left" => Anchor::CENTER_LEFT,
        "center" => Anchor::CENTER,
        "center-right" => Anchor::CENTER_RIGHT,
        "bottom-left" => Anchor::BOTTOM_LEFT,
        "bottom-center" => Anchor::BOTTOM_CENTER,
        "bottom-right" => Anchor::BOTTOM_RIGHT,
        _ => return None,
    })
}

pub struct Bounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Bounds {
    pub fn contains_point(&self, px: f32, py: f32) -> bool {
        px >= self.x
            && px <= self.x + self.width
            && py >= self.y
            && py <= self.y + self.height
    }
}

pub enum UiElement {
    Button { bounds: Bounds, label: String },
    Checkbox { bounds: Bounds, checked: bool },
    Panel { bounds: Bounds },
}

impl UiElement {
    pub fn bounds(&self) -> &Bounds {
        match self {
            UiElement::Button { bounds, .. }
            | UiElement::Checkbox { bounds, .. }
            | UiElement::Panel { bounds } => bounds,
        }
    }
}

pub fn hit_test(elements: &[UiElement], x: f32, y: f32) -> Option<usize> {
    elements
        .iter()
        .enumerate()
        .rev()
        .find(|(_, element)| {
            !matches!(element, UiElement::Panel { .. })
                && element.bounds().contains_point(x, y)
        })
        .map(|(index, _)| index)
}

#[cfg(test)]
mod tests {
    use super::resolve_element_rect;
    use crate::anchor::Rect;
    use std::collections::BTreeMap;

    #[test]
    fn right_anchor_keeps_fixed_margin_on_resized_parent() {
        let attributes = BTreeMap::from([
            ("x".to_string(), "-80".to_string()),
            ("y".to_string(), "50".to_string()),
            ("width".to_string(), "180".to_string()),
            ("height".to_string(), "48".to_string()),
            ("anchor".to_string(), "top-right".to_string()),
        ]);

        let small = resolve_element_rect(
            &attributes,
            Rect { x: 0.0, y: 0.0, width: 800.0, height: 600.0 },
        )
        .unwrap();
        let large = resolve_element_rect(
            &attributes,
            Rect { x: 0.0, y: 0.0, width: 1920.0, height: 1080.0 },
        )
        .unwrap();

        assert_eq!(small.x + small.width, 720.0);
        assert_eq!(large.x + large.width, 1840.0);
        assert_eq!(small.y, 50.0);
        assert_eq!(large.y, 50.0);
    }
}