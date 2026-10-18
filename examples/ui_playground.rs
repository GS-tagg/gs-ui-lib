use gs_ui_lib::config::config;
use gs_ui_lib::debug::FpsTracker;
use gs_ui_lib::element::{UiElement, hit_test, layout_ui, load_ui_nodes};
use gs_ui_lib::gpu::{GpuState, Vertex};
use gs_ui_lib::input::InputState;
use gs_ui_lib::window::{self, WindowContext};
use gs_xml_parser::Node;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::WindowAttributes;
// --- App init ---

#[derive(Default)]
struct App {
    window: Option<WindowContext>,
    fps: Option<FpsTracker>,
    input: InputState,
    ui_nodes: Vec<Node>,
    ui_elements: Vec<UiElement>,
    resize_pending: bool,
    pressed_index: Option<usize>,
    dragging_index: Option<usize>,
    drag_offset: [f32; 2],
}

fn vertices_for_elements(
    size: winit::dpi::PhysicalSize<u32>,
    elements: &[UiElement],
) -> Vec<Vertex> {
    let mut vertices = Vec::new();
    for element in elements {
        let bounds = element.bounds();
        vertices.extend(GpuState::rect_vertices_for_window(
            size,
            bounds.x,
            bounds.y,
            bounds.width,
            bounds.height,
            element.color(),
        ));
    }
    vertices
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let window_attrs = WindowAttributes::default()
                .with_title("ui playground")
                .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
                .with_transparent(true);

            let native_window = window::create_window(event_loop, window_attrs);
            let window_size = native_window.inner_size();

            let ui_nodes = load_ui_nodes("examples/ui.xml");
            let (verts, ui_elements) = layout_ui(&ui_nodes, window_size);
            let window = WindowContext::new(native_window, &verts);

            self.window = Some(window);
            self.ui_nodes = ui_nodes;
            self.ui_elements = ui_elements;
            self.fps = Some(FpsTracker::new());
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let (Some(window), Some(fps)) = (self.window.as_mut(), self.fps.as_mut()) else {
            return;
        };

        self.input.update(&event);
        window.handle_window_event(&event);

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(_) => self.resize_pending = true,
            WindowEvent::CursorMoved { .. } => {
                if let Some(index) = self.dragging_index {
                    let [mx, my] = self.input.mouse.position();
                    let bounds = self.ui_elements[index].bounds_mut();
                    bounds.x = mx as f32 - self.drag_offset[0];
                    bounds.y = my as f32 - self.drag_offset[1];
                    let vertices = vertices_for_elements(window.gpu().size, &self.ui_elements);
                    window.gpu_mut().set_vertices(&vertices);
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                //button press (to be moved)
                if button == MouseButton::Left {
                    let [mx, my] = self.input.mouse.position();
                    let hit = hit_test(&self.ui_elements, mx as f32, my as f32);

                    match state {
                        ElementState::Pressed => {
                            self.pressed_index = hit;
                            self.dragging_index =
                                hit.filter(|index| self.ui_elements[*index].is_movable());
                            if let Some(index) = self.dragging_index {
                                let bounds = self.ui_elements[index].bounds();
                                self.drag_offset = [mx as f32 - bounds.x, my as f32 - bounds.y];
                            }
                        }
                        ElementState::Released => {
                            self.dragging_index = None;
                            let started = self.pressed_index.take();
                            if let (Some(start), Some(end)) = (started, hit) {
                                if start == end {
                                    if let UiElement::Button { label, .. } = &self.ui_elements[end]
                                    {
                                        println!("[ui] button clicked: {label}");
                                    }
                                }
                            }
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if self.resize_pending {
                    self.resize_pending = false;
                    let size = window.window().inner_size();
                    window.gpu_mut().resize(size);
                    let (vertices, elements) = layout_ui(&self.ui_nodes, size);
                    window.gpu_mut().set_vertices(&vertices);
                    self.ui_elements = elements;
                }

                let start = fps.begin_render();
                if let Err(e) = window.gpu_mut().render() {
                    eprintln!("Render error: {e:?}");
                }
                fps.end_render(start);
                if config().fps_tracker_enabled {
                    fps.tick();
                }
                self.input.end_frame();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let Some(window) = self.window.as_mut() else {
            event_loop.set_control_flow(ControlFlow::Wait);
            return;
        };

        window.request_redraw(event_loop);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = window::init()?;
    print!("Loaded runtime config: {}\n", *config());

    let mut app = App {
        window: None,
        fps: None,
        input: InputState::new(),
        ui_nodes: Vec::new(),
        ui_elements: Vec::new(),
        resize_pending: false,
        pressed_index: None,
        dragging_index: None,
        drag_offset: [0.0, 0.0],
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}
