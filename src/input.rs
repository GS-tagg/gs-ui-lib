use std::collections::HashMap;
use std::time::{Duration, Instant};

use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

pub struct InputState {
    pub mouse: MouseHandler,
    pub keyboard: KeyboardHandler,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            mouse: MouseHandler::new(),
            keyboard: KeyboardHandler::new(),
        }
    }

    pub fn update(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse.set_position([position.x, position.y]);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if crate::config::config().input_debug_enabled {
                    let pos = self.mouse.position();
                    println!("[input] mouse {button:?} {state:?} {pos:?}");
                }

                match state {
                    ElementState::Pressed => self.mouse.press(*button),
                    ElementState::Released => self.mouse.release(*button),
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if crate::config::config().input_debug_enabled {
                    println!("[input] key {:?} {:?}", event.physical_key, event.state);
                }

                if let PhysicalKey::Code(code) = event.physical_key {
                    self.keyboard.update(code, event.state);
                }
            }
            _ => {}
        }
    }

    pub fn end_frame(&mut self) {
        self.keyboard.clear_released();
        self.keyboard.clear_changes();
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MouseHandler {
    pos: [f64; 2],
    buttons: HashMap<MouseButton, Instant>,
}

impl MouseHandler {
    pub fn new() -> Self {
        Self {
            pos: [0.0, 0.0],
            buttons: HashMap::new(),
        }
    }

    pub fn position(&self) -> [f64; 2] {
        self.pos
    }

    fn set_position(&mut self, position: [f64; 2]) {
        self.pos = position;
    }

    pub fn press(&mut self, button: MouseButton) {
        self.buttons.entry(button).or_insert_with(Instant::now);
    }

    pub fn release(&mut self, button: MouseButton) {
        self.buttons.remove(&button);
    }

    pub fn held_duration(&self, button: MouseButton) -> Option<Duration> {
        self.buttons.get(&button).map(Instant::elapsed)
    }
}

impl Default for MouseHandler {
    fn default() -> Self {
        Self::new()
    }
}

pub struct KeyState {
    state: ElementState,
    press: Instant,
    release: Duration,
    just_changed: bool,
}

impl KeyState {
    pub fn new() -> Self {
        Self {
            state: ElementState::Released,
            press: Instant::now(),
            release: Duration::ZERO,
            just_changed: false,
        }
    }

    pub fn update(&mut self, state: ElementState) {
        self.just_changed = self.state != state;
        match (self.state, state) {
            (ElementState::Released, ElementState::Pressed) => {
                self.press = Instant::now();
            }
            (ElementState::Pressed, ElementState::Released) => {
                self.release = self.press.elapsed();
            }
            _ => {}
        }
        self.state = state;
    }

    pub fn is_pressed(&self) -> bool {
        self.state == ElementState::Pressed
    }

    pub fn just_pressed(&self) -> bool {
        self.just_changed && self.is_pressed()
    }

    pub fn just_released(&self) -> bool {
        self.just_changed && !self.is_pressed()
    }

    pub fn held_for(&self) -> Duration {
        if self.is_pressed() {
            self.press.elapsed()
        } else {
            self.release
        }
    }

    fn clear_change(&mut self) {
        self.just_changed = false;
    }
}

pub struct KeyboardHandler {
    keymap: HashMap<KeyCode, KeyState>,
}

impl KeyboardHandler {
    pub fn new() -> Self {
        Self {
            keymap: HashMap::new(),
        }
    }

    pub fn update(&mut self, code: KeyCode, state: ElementState) {
        self.keymap
            .entry(code)
            .or_insert_with(KeyState::new)
            .update(state);
    }

    pub fn is_pressed(&self, code: KeyCode) -> bool {
        self.keymap
            .get(&code)
            .map(KeyState::is_pressed)
            .unwrap_or(false)
    }

    pub fn held_for(&self, code: KeyCode) -> Duration {
        self.keymap
            .get(&code)
            .map(KeyState::held_for)
            .unwrap_or(Duration::ZERO)
    }

    pub fn just_released(&self, code: KeyCode) -> bool {
        self.keymap
            .get(&code)
            .map(KeyState::just_released)
            .unwrap_or(false)
    }

    pub fn just_pressed(&self, code: KeyCode) -> bool {
        self.keymap
            .get(&code)
            .map(KeyState::just_pressed)
            .unwrap_or(false)
    }

    pub fn pressed_keys(&self) -> impl Iterator<Item = &KeyCode> {
        self.keymap
            .iter()
            .filter(|(_, state)| state.is_pressed())
            .map(|(code, _)| code)
    }

    pub fn clear_released(&mut self) {
        self.keymap.retain(|_, state| state.is_pressed());
    }

    fn clear_changes(&mut self) {
        for state in self.keymap.values_mut() {
            state.clear_change();
        }
    }
}

impl Default for KeyboardHandler {
    fn default() -> Self {
        Self::new()
    }
}