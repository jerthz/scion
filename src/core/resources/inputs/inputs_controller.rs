//! Everything that is relatives to the core.resources.inputs.

use crate::core::resources::inputs::{
    keyboard::Keyboard,
    mouse::{Mouse, MouseEvent},
    types::{Input, InputState, KeyCode, KeyboardEvent, Shortcut},
};

/// A resource updated by `Scion` to keep track of the core.resources.inputs
/// Can be used in any system.
#[derive(Default)]
pub struct InputsController {
    mouse: Mouse,
    keyboard: Keyboard,
}

impl InputsController {
    /// Whether or not `key` is currently pressed
    pub fn key_pressed(&self, key: &KeyCode) -> bool {
        self.keyboard.pressed_keys.contains(key)
    }

    /// convenient function to run `action` if `key` is pressed during the current frame
    pub fn on_key_pressed<Body>(&self, key: KeyCode, action: Body)
    where
        Body: FnMut(),
    {
        self.keyboard.on_key_pressed(key, action);
    }

    /// convenient function to run `action` if `key` is released during the current frame
    pub fn on_key_released<Body>(&self, key: KeyCode, action: Body)
    where
        Body: FnMut(),
    {
        self.keyboard.on_key_released(key, action);
    }

    /// Execute the action `action` if the left mouse button is clicked, actions params are mouse position x;y
    pub fn on_left_click_pressed<Body>(&self, action: Body)
    where
        Body: FnMut(f64, f64),
    {
        self.mouse.on_left_click_pressed(action);
    }

    /// Execute the action `action` if the right mouse button is clicked, actions params are mouse position x;y
    pub fn on_right_click_pressed<Body>(&self, action: Body)
    where
        Body: FnMut(f64, f64),
    {
        self.mouse.on_right_click_pressed(action);
    }

    /// Execute the action `action` if the middle mouse button is clicked, actions params are mouse position x;y
    pub fn on_middle_click_pressed<Body>(&self, action: Body)
    where
        Body: FnMut(f64, f64),
    {
        self.mouse.on_middle_click_pressed(action);
    }

    /// Execute the action `action` if the left mouse button is released, actions params are mouse position x;y
    pub fn on_left_click_released<Body>(&self, action: Body)
    where
        Body: FnMut(f64, f64),
    {
        self.mouse.on_left_click_released(action);
    }

    /// Execute the action `action` if the right mouse button is released, actions params are mouse position x;y
    pub fn on_right_click_released<Body>(&self, action: Body)
    where
        Body: FnMut(f64, f64),
    {
        self.mouse.on_right_click_released(action);
    }

    /// Execute the action `action` if the right mouse button is released, actions params are mouse position x;y
    pub fn on_middle_click_released<Body>(&self, action: Body)
    where
        Body: FnMut(f64, f64),
    {
        self.mouse.on_middle_click_released(action);
    }

    /// Retrieve all the inputs pressed or clicked during last frame
    pub fn all_pressed_events(&self) -> Vec<Input> {
        self.all_events_for_state(InputState::Pressed)
    }

    /// Retrieve all the inputs released or clicked during last frame
    pub fn all_released_events(&self) -> Vec<Input> {
        self.all_events_for_state(InputState::Released)
    }

    /// Retrieve all the inputs that are currently hold pressed or clicked
    pub fn all_pressed(&self) -> Vec<Input> {
        let mut pressed = self.keyboard.all_pressed();
        let mut mouse_pressed = self.mouse.all_pressed();
        pressed.append(&mut mouse_pressed);
        pressed
    }

    /// Retrieve the mouse x and y position
    pub fn mouse_xy(&self) -> (f64, f64) {
        self.mouse.xy()
    }

    /// Whether or not `shortcut` is currently pressed
    pub fn shortcut_pressed(&self, shortcut: &Shortcut) -> bool {
        shortcut.iter().all(|input| self.input_pressed(input))
    }

    /// Whether or not `shortcut` has just been pressed
    pub fn shortcut_pressed_event(&self, shortcut: &Shortcut) -> bool {
        shortcut.iter().all(|input| self.input_pressed(input))
            && shortcut.iter().any(|input| self.all_pressed_events().contains(input))
    }

    /// Whether or not `shortcut` has just been released
    pub fn shortcut_released_event(&self, shortcut: &Shortcut) -> bool {
        shortcut.iter().any(|input| self.all_released_events().contains(input))
            && shortcut.iter().all(|input| {
                self.input_pressed(input) || self.all_released_events().contains(input)
            })
    }

    fn all_events_for_state(&self, input_state: InputState) -> Vec<Input> {
        let mut inputs = self.keyboard.all_keys_at_state(input_state);
        let mut mouse_inputs = self.mouse.all_click_at_state(input_state);
        inputs.append(&mut mouse_inputs);
        inputs
    }

    pub fn input_pressed(&self, input: &Input) -> bool {
        match input {
            Input::Key(keycode) => self.key_pressed(keycode),
            Input::Mouse(mouse_button) => self.mouse.button_pressed(mouse_button),
        }
    }

    pub fn input_pressed_event(&self, input: &Input) -> bool {
        self.all_pressed_events().contains(input)
    }

    pub(crate) fn reset_inputs(&mut self) {
        self.mouse.clear_events();
        self.keyboard.clear_events();
    }

    pub(crate) fn set_mouse_position(&mut self, x: f64, y: f64) {
        self.mouse.set_position(x, y);
    }

    pub(crate) fn add_click_event(&mut self, event: MouseEvent) {
        self.mouse.add_click_event(event);
    }

    pub(crate) fn add_keyboard_event(&mut self, event: KeyboardEvent) {
        self.keyboard.add_keyboard_event(event);
    }
}
