use crate::config::{config, load_runtime_config, save_runtime_config};
use crate::debug::{FpsTracker, FramePacer};
use crate::gpu::{GpuState, Vertex};
use crate::input::InputState;
use std::sync::Arc;
use std::time::Duration;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};
use winit::window::{Fullscreen, Window, WindowAttributes};

pub fn create_window(event_loop: &ActiveEventLoop, attributes: WindowAttributes) -> Arc<Window> {
    Arc::new(
        event_loop
            .create_window(attributes)
            .expect("failed to create window"),
    )
}

pub struct WindowContext {
    window: Arc<Window>,
    gpu: GpuState,
    frame_pacer: FramePacer,
    modifiers: ModifiersState,
}

impl WindowContext {
    pub fn new(window: Arc<Window>, vertices: &[Vertex]) -> Self {
        let gpu = GpuState::new(window.clone(), vertices);
        let frame_pacer = FramePacer::new(config().target_fps);

        Self {
            window,
            gpu,
            frame_pacer,
            modifiers: ModifiersState::empty(),
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn gpu(&self) -> &GpuState {
        &self.gpu
    }

    pub fn gpu_mut(&mut self) -> &mut GpuState {
        &mut self.gpu
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state();
            }
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed && !event.repeat =>
            {
                let is_fullscreen_shortcut = match event.physical_key {
                    PhysicalKey::Code(KeyCode::F11) => true,
                    PhysicalKey::Code(KeyCode::Enter) => self.modifiers.alt_key(),
                    _ => false,
                };

                if is_fullscreen_shortcut {
                    let fullscreen = if self.window.fullscreen().is_some() {
                        None
                    } else {
                        Some(Fullscreen::Borderless(self.window.current_monitor()))
                    };
                    self.window.set_fullscreen(fullscreen);
                }
            }
            _ => {}
        }
    }

    pub fn begin_redraw(&mut self) -> bool {
        self.frame_pacer.begin_frame(std::time::Instant::now())
    }

    pub fn request_redraw(&mut self, event_loop: &ActiveEventLoop) {
        let cfg = *config();
        let target_duration = Duration::from_secs_f64(1.0 / cfg.target_fps.max(1) as f64);
        let frame_duration = if cfg.vsync_enabled {
            self.window
                .current_monitor()
                .and_then(|monitor| monitor.refresh_rate_millihertz())
                .filter(|refresh_rate| *refresh_rate > 0)
                .map(|refresh_rate| {
                    target_duration.max(Duration::from_secs_f64(1_000.0 / refresh_rate as f64))
                })
                .unwrap_or(target_duration)
        } else {
            target_duration
        };
        self.frame_pacer.set_frame_duration(frame_duration);

        if self
            .frame_pacer
            .should_request_redraw(std::time::Instant::now())
        {
            self.window.request_redraw();
            event_loop.set_control_flow(ControlFlow::Wait);
        } else {
            event_loop.set_control_flow(ControlFlow::WaitUntil(self.frame_pacer.next_frame_time()));
        }
    }
}

#[derive(Default)]

struct App {
    window: Option<WindowContext>,
    fps: Option<FpsTracker>,
    input: InputState,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let window_attrs = WindowAttributes::default()
                .with_title("space-game")
                .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
                .with_transparent(true);

            let window = create_window(event_loop, window_attrs);
            self.window = Some(WindowContext::new(window, &[] as &[Vertex]));
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

            WindowEvent::Resized(physical_size) => {
                window.gpu_mut().resize(physical_size);
            }

            WindowEvent::RedrawRequested => {
                if !window.begin_redraw() {
                    return;
                }
                let cfg = config();
                let start = fps.begin_render();
                if let Err(e) = window.gpu_mut().render() {
                    eprintln!("Render error: {e:?}");
                }

                fps.end_render(start);

                if cfg.fps_tracker_enabled {
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

pub fn init() -> Result<EventLoop<()>, Box<dyn std::error::Error>> {
    load_runtime_config()?;

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    Ok(event_loop)
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = init()?;
    let mut app = App {
        window: None,
        fps: None,
        input: InputState::new(),
    };
    event_loop.run_app(&mut app)?;

    save_runtime_config()?;
    Ok(())
}
