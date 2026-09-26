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

impl UiElement {
    pub fn bounds(&self) -> &Bounds {
        match self {
            UiElement::Button { bounds, .. } => bounds,
            UiElement::Checkbox { bounds, .. } => bounds,
            UiElement::Panel { bounds, .. } => bounds,
        }
    }

    pub fn contains_point(&self, px: f32, py: f32) -> bool {
        self.bounds().contains_point(px, py)
    }
}
