use crate::anchor::{Anchor, AnchorConstraints, Rect, Vec2, resolve_anchor};
use crate::gpu::{GpuState, Vertex};
use gs_xml_parser::{Node, parse, tokenize};
use std::collections::BTreeMap;
use winit::dpi::PhysicalSize;

pub fn build_ui(xml_path: &str, window_size: PhysicalSize<u32>) -> (Vec<Vertex>, Vec<UiElement>) {
    let nodes = load_ui_nodes(xml_path);
    layout_ui(&nodes, window_size)
}

pub fn load_ui_nodes(xml_path: &str) -> Vec<Node> {
    let xml = std::fs::read_to_string(xml_path).expect("failed to read ui xml");
    let tokens = tokenize(&xml).expect("tokenize failed");
    parse(&tokens).expect("parse failed")
}

pub fn layout_ui(nodes: &[Node], window_size: PhysicalSize<u32>) -> (Vec<Vertex>, Vec<UiElement>) {
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
    for node in nodes {
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
            let color =
                parse_color(attributes.get("color").map(String::as_str)).unwrap_or([1.0, 1.0, 1.0]);
            let opacity = attributes
                .get("opacity")
                .and_then(|value| value.parse::<f32>().ok())
                .filter(|value| value.is_finite())
                .unwrap_or(1.0)
                .clamp(0.0, 1.0);
            let rgba = [color[0], color[1], color[2], opacity];
            vertices.extend(GpuState::rect_vertices_for_window(
                window_size,
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                rgba,
            ));

            let bounds = Bounds {
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height,
            };
            let movable = attributes
                .get("movable")
                .is_some_and(|value| value == "true");
            match tag.as_str() {
                "button" => {
                    let label = attributes.get("label").cloned().unwrap_or_default();
                    elements.push(UiElement::Button {
                        bounds,
                        label,
                        movable,
                        color: rgba,
                    });
                }
                "checkbox" => {
                    let checked = attributes
                        .get("checked")
                        .is_some_and(|value| value == "true");
                    elements.push(UiElement::Checkbox {
                        bounds,
                        checked,
                        movable,
                        color: rgba,
                    });
                }
                "panel" => elements.push(UiElement::Panel {
                    bounds,
                    movable,
                    color: rgba,
                }),
                _ => {}
            }
        }

        let child_parent = element_rect.unwrap_or(parent);
        for child in children {
            collect_layout(child, child_parent, window_size, vertices, elements);
        }
    }
}

fn parse_color(value: Option<&str>) -> Option<[f32; 3]> {
    let value = value?.strip_prefix('#')?;
    if value.len() != 6 {
        return None;
    }
    let red = u8::from_str_radix(&value[0..2], 16).ok()?;
    let green = u8::from_str_radix(&value[2..4], 16).ok()?;
    let blue = u8::from_str_radix(&value[4..6], 16).ok()?;
    Some([
        red as f32 / 255.0,
        green as f32 / 255.0,
        blue as f32 / 255.0,
    ])
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

    let mut rect = resolve_anchor(
        parent,
        Vec2 {
            x: width,
            y: height,
        },
        &AnchorConstraints {
            target_anchor: anchor,
            pivot,
            offset: Vec2 {
                x: value("x").unwrap_or(0.0),
                y: value("y").unwrap_or(0.0),
            },
        },
    );
    rect.width = rect.width.max(0.0).min(parent.width.max(0.0));
    rect.height = rect.height.max(0.0).min(parent.height.max(0.0));
    rect.x = rect.x.clamp(
        parent.x,
        (parent.x + parent.width - rect.width).max(parent.x),
    );
    rect.y = rect.y.clamp(
        parent.y,
        (parent.y + parent.height - rect.height).max(parent.y),
    );

    Some(rect)
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
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height
    }
}

pub enum UiElement {
    Button {
        bounds: Bounds,
        label: String,
        movable: bool,
        color: [f32; 4],
    },
    Checkbox {
        bounds: Bounds,
        checked: bool,
        movable: bool,
        color: [f32; 4],
    },
    Panel {
        bounds: Bounds,
        movable: bool,
        color: [f32; 4],
    },
}

impl UiElement {
    pub fn bounds(&self) -> &Bounds {
        match self {
            UiElement::Button { bounds, .. }
            | UiElement::Checkbox { bounds, .. }
            | UiElement::Panel { bounds, .. } => bounds,
        }
    }

    pub fn color(&self) -> [f32; 4] {
        match self {
            UiElement::Button { color, .. }
            | UiElement::Checkbox { color, .. }
            | UiElement::Panel { color, .. } => *color,
        }
    }

    pub fn bounds_mut(&mut self) -> &mut Bounds {
        match self {
            UiElement::Button { bounds, .. }
            | UiElement::Checkbox { bounds, .. }
            | UiElement::Panel { bounds, .. } => bounds,
        }
    }

    pub fn is_movable(&self) -> bool {
        match self {
            UiElement::Button { movable, .. }
            | UiElement::Checkbox { movable, .. }
            | UiElement::Panel { movable, .. } => *movable,
        }
    }
}

pub fn hit_test(elements: &[UiElement], x: f32, y: f32) -> Option<usize> {
    elements
        .iter()
        .enumerate()
        .rev()
        .find(|(_, element)| {
            !matches!(element, UiElement::Panel { .. }) && element.bounds().contains_point(x, y)
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
            Rect {
                x: 0.0,
                y: 0.0,
                width: 800.0,
                height: 600.0,
            },
        )
        .unwrap();
        let large = resolve_element_rect(
            &attributes,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1920.0,
                height: 1080.0,
            },
        )
        .unwrap();

        assert_eq!(small.x + small.width, 720.0);
        assert_eq!(large.x + large.width, 1840.0);
        assert_eq!(small.y, 50.0);
        assert_eq!(large.y, 50.0);
    }

    #[test]
    fn bottom_anchor_stays_inside_a_short_parent() {
        let attributes = BTreeMap::from([
            ("x".to_string(), "-80".to_string()),
            ("y".to_string(), "-170".to_string()),
            ("width".to_string(), "180".to_string()),
            ("height".to_string(), "48".to_string()),
            ("anchor".to_string(), "bottom-right".to_string()),
        ]);

        let rect = resolve_element_rect(
            &attributes,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 300.0,
                height: 120.0,
            },
        )
        .unwrap();

        assert!(rect.x >= 0.0);
        assert!(rect.y >= 0.0);
        assert!(rect.x + rect.width <= 300.0);
        assert!(rect.y + rect.height <= 120.0);
    }
}
