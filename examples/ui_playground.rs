use gs_ui_lib::config::{config, load_runtime_config};
use gs_ui_lib::debug::FpsTracker;
use gs_ui_lib::element::{UiElement, build_ui, hit_test};
use gs_ui_lib::gpu::{GpuState, Vertex};
use gs_ui_lib::input::InputState;

use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes};
// --- App init ---

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    gpu: Option<GpuState>,
    fps: Option<FpsTracker>,
    input: InputState,
    ui_elements: Vec<UiElement>,
    pressed_index: Option<usize>,
    dragging_index: Option<usize>,
    drag_offset: [f32; 2],
}

fn vertices_for_elements(size: winit::dpi::PhysicalSize<u32>, elements: &[UiElement]) -> Vec<Vertex> {
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
            WindowEvent::Resized(size) => {
                gpu.resize(size);
                let (verts, elements) = build_ui("examples/ui.xml", size);
                gpu.set_vertices(&verts);
                self.ui_elements = elements;
            }
            WindowEvent::CursorMoved { .. } => {
                if let Some(index) = self.dragging_index {
                    let [mx, my] = self.input.mouse.position();
                    let bounds = self.ui_elements[index].bounds_mut();
                    bounds.x = mx as f32 - self.drag_offset[0];
                    bounds.y = my as f32 - self.drag_offset[1];
                    let vertices = vertices_for_elements(gpu.size, &self.ui_elements);
                    gpu.set_vertices(&vertices);
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
        pressed_index: None,
        dragging_index: None,
        drag_offset: [0.0, 0.0],
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}
