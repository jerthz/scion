use std::collections::HashSet;

use crate::core::resources::inputs::types::{Input, InputState, KeyCode, KeyboardEvent};

#[derive(Default)]
/// Convenience resource used to keep track of keyboard inputs
pub struct Keyboard {
    pub(crate) pressed_keys: HashSet<KeyCode>,
    pub(crate) keyboard_events: Vec<KeyboardEvent>,
}

impl Keyboard {
    /// Whether or not `key` is currently pressed
    pub fn key_pressed(&self, key: &KeyCode) -> bool {
        self.pressed_keys.contains(key)
    }

    /// Keyboard events of the current frame
    pub fn keyboard_events(&self) -> &Vec<KeyboardEvent> {
        &self.keyboard_events
    }

    /// convenient function to run `action` if `key` is pressed during the current frame
    pub fn on_key_pressed<Body>(&self, key: KeyCode, mut action: Body)
    where
        Body: FnMut(),
    {
        if self
            .keyboard_events
            .iter()
            .filter(|e| e.keycode == key && e.state == InputState::Pressed)
            .count()
            > 0
        {
            action()
        }
    }

    /// convenient function to run `action` if `key` is released during the current frame
    pub fn on_key_released<Body>(&self, key: KeyCode, mut action: Body)
    where
        Body: FnMut(),
    {
        if self
            .keyboard_events
            .iter()
            .filter(|e| e.keycode == key && e.state == InputState::Released)
            .count()
            > 0
        {
            action()
        }
    }

    pub(crate) fn all_keys_at_state(&self, state: InputState) -> Vec<Input> {
        self.keyboard_events
            .iter()
            .filter(|input| input.state == state)
            .map(|input| Input::Key(input.keycode))
            .collect()
    }

    pub(crate) fn all_pressed(&self) -> Vec<Input> {
        self.pressed_keys.iter().map(|input| Input::Key(*input)).collect()
    }

    pub(crate) fn clear_events(&mut self) {
        self.keyboard_events.clear();
    }

    pub(crate) fn add_keyboard_event(&mut self, keyboard_event: KeyboardEvent) {
        if match &keyboard_event {
            KeyboardEvent { keycode, state } => match state {
                InputState::Pressed => self.press(keycode),
                InputState::Released => self.release(keycode),
            },
        } {
            self.keyboard_events.push(keyboard_event);
        }
    }

    /// Set the given key to pressed status if it is not present in the registered pressed key. Return true if it was updated
    pub(crate) fn press(&mut self, key: &KeyCode) -> bool {
        if self.pressed_keys.contains(key) {
            false
        } else {
            self.pressed_keys.insert(*key);
            true
        }
    }

    /// Set the given key to released status if it present in the registered pressed key. Return true if it was updated
    pub(crate) fn release(&mut self, key: &KeyCode) -> bool {
        if self.pressed_keys.contains(key) {
            self.pressed_keys.remove(key);
            true
        } else {
            false
        }
    }
}
