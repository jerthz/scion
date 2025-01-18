use hecs::Entity;
use std::collections::HashMap;

/// `GameState` is a convenience Resource created to keep track of
/// diverse thing internally. It's also the resource used to create
/// pausable systems.
#[derive(Debug, Default)]
pub struct GameState {
    flags: HashMap<String, bool>,
    text: HashMap<String, String>,
    color_picked_entity: Option<Entity>,
    color_picked_status_update: Option<bool>,
}

impl GameState {
    pub fn get_bool(&self, key: &str) -> bool {
        if self.flags.contains_key(key) {
            return *self.flags.get(key).unwrap();
        } else {
            false
        }
    }

    pub fn set_bool(&mut self, key: &str, val: bool) {
        self.flags.insert(key.to_string(), val);
    }

    pub fn get_text(&self, key: &str) -> Option<String> {
        if self.text.contains_key(key) {
            return Some(self.text.get(key).unwrap().to_string());
        } else {
            None
        }
    }

    pub fn set_text(&mut self, key: &str, val: &str) {
        self.text.insert(key.to_string(), val.to_string());
    }

    pub(crate) fn set_color_picked_entity(&mut self, e: Option<Entity>) {
        self.color_picked_entity = e;
    }

    pub fn get_color_picked_entity(&self) -> Option<Entity> {
        self.color_picked_entity.clone()
    }

    pub fn update_color_picking_status(&mut self, status: bool) {
        self.color_picked_status_update = Some(status);
    }

    pub(crate) fn take_picking_update(&mut self) -> Option<bool> {
        self.color_picked_status_update.take()
    }
}
