use crate::window::{Window, WindowAttributes};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Anchor {
    pub x: f32,
    pub y: f32
}

impl Anchor{
    pub const TOP_LEFT: Self = Self{ x: 0.0, y: 0.0};
    pub const CENTER: Self = Self{ x: 0.5, y: 0.5};
    pub const TOP_RIGHT: Self = Self{ x: 1.0, y: 1.0};
    pub const STRETC: Self = Self{ x: 1.0, y: 1.0};

}

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct AnchorConstraints {
    // what to attach to
    pub target_anchor: Anchor,
    // what to attach from 
    pub pivot: Anchor,
    // extra shift (like a margin or smth)
    pub offset: Vec2,
}

// gets element pixel-pos
pub fn resolve_anchor(
    parent_rect: Rect,
    child_size: Vec2,
    constraint: &AnchorConstraints,
) -> Rect {
    let anchor_point = Vec2 {
	// find the point inside parent
        x: parent_rect.x + (parent_rect.width * constraint.target_anchor.x),
        y: parent_rect.y + (parent_rect.height * constraint.target_anchor.y),
    };
    // Line up the element's pivot with the parent's targets pivot, then add the offset
    let child_x = anchor_point.x - (child_size.x * constraint.pivot.x) + constraint.offset.x;
    let child_y = anchor_point.y - (child_size.y * constraint.pivot.y) + constraint.offset.y;

    Rect {
        x: child_x,
        y: child_y,
        width: child_size.x,
        height: child_size.y,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Anchors {
    pub min: Anchor, // Top-Left binding relative to parent [0..1]
    pub max: Anchor, // Bottom-Right binding relative to parent [0..1]
}

#[derive(Debug, Clone, Copy)]
pub struct Offsets {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

pub fn resolve_stretched_layout(
    parent: Rect,
    anchors: Anchors,
    offsets: Offsets,
) -> Rect {
    // calculates corners
    let min_x = parent.x + (parent.width * anchors.min.x) + offsets.left;
    let min_y = parent.y + (parent.height * anchors.min.y) + offsets.top;
    
    let max_x = parent.x + (parent.width * anchors.max.x) - offsets.right;
    let max_y = parent.y + (parent.height * anchors.max.y) - offsets.bottom;
    // > 0
    Rect {
        x: min_x,
        y: min_y,
        width: (max_x - min_x).max(0.0),
        height: (max_y - min_y).max(0.0),
    }
}
