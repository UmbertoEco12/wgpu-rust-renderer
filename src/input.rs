use std::collections::HashSet;

pub use winit::event::VirtualKeyCode as KeyCode;
use winit::{
    event::{KeyboardInput, VirtualKeyCode},
    window::WindowId,
};
pub struct Input {
    pressed_keys: HashSet<VirtualKeyCode>,
    just_pressed_keys: HashSet<VirtualKeyCode>,
    just_released_keys: HashSet<VirtualKeyCode>,
}
pub static mut INPUT: Option<Input> = None;

impl Input {
    pub fn is_key_pressed(&self, key: VirtualKeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }
    pub fn is_key_just_pressed(&self, key: VirtualKeyCode) -> bool {
        self.just_pressed_keys.contains(&key)
    }
    pub fn is_key_just_released(&self, key: VirtualKeyCode) -> bool {
        self.just_released_keys.contains(&key)
    }
    pub fn handle_keyboard_event(&mut self, event: &KeyboardInput, _window_id: WindowId) {
        if let Some(key) = event.virtual_keycode {
            match event.state {
                winit::event::ElementState::Pressed => {
                    if !self.pressed_keys.contains(&key) {
                        self.just_pressed_keys.insert(key);
                    }
                    // Key pressed, add it to the list
                    self.pressed_keys.insert(key);
                }
                winit::event::ElementState::Released => {
                    if self.pressed_keys.contains(&key) {
                        self.just_released_keys.insert(key);
                    }
                    // Key released, remove it from the list
                    self.pressed_keys.remove(&key);
                }
            }
        }
    }

    pub fn update_input(&mut self) {
        self.just_released_keys.clear();
        self.just_pressed_keys.clear();
    }
}

pub fn init() {
    unsafe {
        INPUT = Some(Input {
            pressed_keys: HashSet::new(),
            just_pressed_keys: HashSet::new(),
            just_released_keys: HashSet::new(),
        });
    }
}

pub fn is_key_pressed(key: KeyCode) -> bool {
    let mut pressed: bool = false;
    unsafe {
        if let Some(input) = INPUT.as_ref() {
            pressed = input.is_key_pressed(key);
        }
    }
    pressed
}

pub fn is_key_just_pressed(key: KeyCode) -> bool {
    let mut pressed: bool = false;
    unsafe {
        if let Some(input) = INPUT.as_ref() {
            pressed = input.is_key_just_pressed(key);
        }
    }
    pressed
}

pub fn is_key_just_released(key: KeyCode) -> bool {
    let mut pressed: bool = false;
    unsafe {
        if let Some(input) = INPUT.as_ref() {
            pressed = input.is_key_just_released(key);
        }
    }
    pressed
}
