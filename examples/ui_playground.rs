use gs_ui_lib::gpu::{GpuState, Vertex};
use gs_ui_lib::input::InputState;
use gs_ui_lib::debug::FpsTracker;
use gs_ui_lib::config::{config, load_runtime_config};
use gs_xml_parser::{tokenize, parse, Node};

use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{WindowEvent, MouseButton, ElementState};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes};
use winit::dpi::PhysicalSize;

// --- UI types ---

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
            UiElement::Button { bounds, .. } => bounds,
            UiElement::Checkbox { bounds, .. } => bounds,
            UiElement::Panel { bounds, .. } => bounds,
        }
    }

    pub fn contains_point(&self, px: f32, py: f32) -> bool {
        self.bounds().contains_point(px, py)
    }
}

// --- XML -> GPU + UI element builders ---

fn build_ui(xml_path: &str, window_size: PhysicalSize<u32>) -> (Vec<Vertex>, Vec<UiElement>) {
    let xml = std::fs::read_to_string(xml_path).expect("failed to read ui xml");
    let tokens = tokenize(&xml).expect("tokenize failed");
    let nodes = parse(&tokens).expect("parse failed");

    let mut verts = Vec::new();
    let mut elements = Vec::new();
    for node in &nodes {
        collect_vertices(node, window_size, &mut verts);
        collect_elements(node, &mut elements);
    }
    (verts, elements)
}

fn collect_vertices(node: &Node, window_size: PhysicalSize<u32>, out: &mut Vec<Vertex>) {
    if let Node::Element { attributes, children, .. } = node {
        let get = |k: &str| attributes.get(k).and_then(|v| v.parse::<f32>().ok());

        if let (Some(x), Some(y), Some(w), Some(h)) =
            (get("x"), get("y"), get("width"), get("height"))
        {
            let color = [1.0, 1.0, 1.0];
            out.extend(GpuState::rect_vertices_for_window(window_size, x, y, w, h, color));
        }

        for child in children {
            collect_vertices(child, window_size, out);
        }
    }
}

fn collect_elements(node: &Node, out: &mut Vec<UiElement>) {
    if let Node::Element { tag, attributes, children, .. } = node {
        let get = |k: &str| attributes.get(k).and_then(|v| v.parse::<f32>().ok());

        if let (Some(x), Some(y), Some(width), Some(height)) =
            (get("x"), get("y"), get("width"), get("height"))
        {
            let bounds = Bounds { x, y, width, height };
            match tag.as_str() {
                "button" => {
                    let label = attributes.get("label").cloned().unwrap_or_default();
                    out.push(UiElement::Button { bounds, label });
                }
                "checkbox" => {
                    let checked = attributes.get("checked")
                        .map(|v| v == "true")
                        .unwrap_or(false);
                    out.push(UiElement::Checkbox { bounds, checked });
                }
                "panel" => {
                    out.push(UiElement::Panel { bounds });
                }
                _ => {}
            }
        }

        for child in children {
            collect_elements(child, out);
        }
    }
}

// --- App ---

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    gpu: Option<GpuState>,
    fps: Option<FpsTracker>,
    input: InputState,
    ui_elements: Vec<UiElement>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let window_attrs = WindowAttributes::default()
                .with_title("ui playground")
                .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0));

            let window = Arc::new(event_loop.create_window(window_attrs).unwrap());
            let window_size = window.inner_size();

            let (verts, ui_elements) = build_ui("examples/ui.xml", window_size);
            let gpu = GpuState::new(window.clone(), &verts);

            self.window = Some(window);
            self.gpu = Some(gpu);
            self.ui_elements = ui_elements;
            let cfg = config();
            self.fps = Some(FpsTracker::new(cfg.target_fps as f32));
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let (Some(gpu), Some(window), Some(fps)) =
            (self.gpu.as_mut(), self.window.as_ref(), self.fps.as_mut())
        else {
            return;
        };

        self.input.update(&event);

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => gpu.resize(size),
            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left && state == ElementState::Pressed {
                    let [mx, my] = self.input.mouse.position();
                    for element in &self.ui_elements {
                        if element.contains_point(mx as f32, my as f32) {
                            if let UiElement::Button { label, .. } = element {
                                println!("[ui] button pressed: {label}");
                            }
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                let start = fps.begin_render();
                if let Err(e) = gpu.render() {
                    eprintln!("Render error: {e:?}");
                }
                fps.end_render(start);
                self.input.end_frame();
                window.request_redraw();
            }
            _ => {}
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_runtime_config()?;

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App {
        window: None,
        gpu: None,
        fps: None,
        input: InputState::new(),
        ui_elements: Vec::new(),
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}