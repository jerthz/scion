use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::core::resources::inputs::types::{Input, InputState, MouseButton};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MouseEvent {
    pub button: MouseButton,
    pub state: InputState,
}

impl From<winit::event::MouseButton> for MouseButton {
    fn from(button: winit::event::MouseButton) -> Self {
        match button {
            winit::event::MouseButton::Left => MouseButton::Left,
            winit::event::MouseButton::Right => MouseButton::Right,
            winit::event::MouseButton::Middle => MouseButton::Middle,
            winit::event::MouseButton::Other(v) => MouseButton::Other(v),
            winit::event::MouseButton::Back => MouseButton::Back,
            winit::event::MouseButton::Forward => MouseButton::Forward
        }
    }
}

/// Contains some data about the mouse, updated at each event received from the window.
/// Can be used in any system.
#[derive(Default, Debug)]
pub struct Mouse {
    x: f64,
    y: f64,
    buttons_pressed: HashSet<MouseButton>,
    click_events: Vec<MouseEvent>,
}

impl Mouse {
    pub(crate) fn set_position(&mut self, x: f64, y: f64) {
        self.x = x;
        self.y = y;
    }

    pub(crate) fn add_click_event(&mut self, event: MouseEvent) {
        if event.state == InputState::Pressed {
            self.buttons_pressed.insert(event.button);
        } else if self.buttons_pressed.contains(&event.button) {
            self.buttons_pressed.remove(&event.button);
        }
        self.click_events.push(event);
    }

    pub(crate) fn reset_mouse_wheel(&mut self) {
        if self.buttons_pressed.contains(&MouseButton::WheelUp) {
            self.buttons_pressed.remove(&MouseButton::WheelUp);
        }
        if self.buttons_pressed.contains(&MouseButton::WheelDown) {
            self.buttons_pressed.remove(&MouseButton::WheelDown);
        }
    }

    pub(crate) fn clear_events(&mut self) {
        self.click_events.clear();
    }

    /// Execute the action `action` if the left mouse button is clicked, actions params are mouse position x;y
    pub(crate) fn on_left_click_pressed<Body>(&self, action: Body)
    where
        Body: FnMut(f64, f64),
    {
        self.on_mouse_event(MouseButton::Left, InputState::Pressed, action);
    }

    /// Execute the action `action` if the right mouse button is clicked, actions params are mouse position x;y
    pub(crate) fn on_right_click_pressed<Body>(&self, action: Body)
    where
        Body: FnMut(f64, f64),
    {
        self.on_mouse_event(MouseButton::Right, InputState::Pressed, action);
    }

    /// Execute the action `action` if the middle mouse button is clicked, actions params are mouse position x;y
    pub(crate) fn on_middle_click_pressed<Body>(&self, action: Body)
    where
        Body: FnMut(f64, f64),
    {
        self.on_mouse_event(MouseButton::Middle, InputState::Pressed, action);
    }

    /// Execute the action `action` if the left mouse button is released, actions params are mouse position x;y
    pub(crate) fn on_left_click_released<Body>(&self, action: Body)
    where
        Body: FnMut(f64, f64),
    {
        self.on_mouse_event(MouseButton::Left, InputState::Released, action);
    }

    /// Execute the action `action` if the right mouse button is released, actions params are mouse position x;y
    pub(crate) fn on_right_click_released<Body>(&self, action: Body)
    where
        Body: FnMut(f64, f64),
    {
        self.on_mouse_event(MouseButton::Right, InputState::Pressed, action);
    }

    /// Execute the action `action` if the right mouse button is released, actions params are mouse position x;y
    pub(crate) fn on_middle_click_released<Body>(&self, action: Body)
    where
        Body: FnMut(f64, f64),
    {
        self.on_mouse_event(MouseButton::Middle, InputState::Pressed, action);
    }

    fn on_mouse_event<Body>(
        &self,
        target_button: MouseButton,
        target_state: InputState,
        mut action: Body,
    ) where
        Body: FnMut(f64, f64),
    {
        for event in self.click_events.iter() {
            if event.button == target_button && event.state == target_state {
                return action(self.x, self.y);
            }
        }
    }

    pub(crate) fn all_click_at_state(&self, state: InputState) -> Vec<Input> {
        self.click_events
            .iter()
            .filter(|input| input.state == state)
            .map(|input| Input::Mouse(input.button))
            .collect()
    }

    pub(crate) fn all_pressed(&self) -> Vec<Input> {
        self.buttons_pressed.iter().map(|input| Input::Mouse(*input)).collect()
    }

    /// Returns the current x and y value of the cursor
    pub(crate) fn xy(&self) -> (f64, f64) {
        (self.x, self.y)
    }
    pub(crate) fn button_pressed(&self, button: &MouseButton) -> bool {
        self.buttons_pressed.contains(button)
    }
}
