use serde::{Deserialize, Serialize};

/// Convenience struct used in all `Scion` to specify any 2D position.
#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct Coordinates {
    pub(crate) x: f32,
    pub(crate) y: f32,
    #[serde(default)]
    pub(crate) z: usize,
}

impl Coordinates {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y, z: 0 }
    }

    pub fn new_with_z(x: f32, y: f32, layer: usize) -> Self {
        Self { x, y, z: layer }
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn z(&self) -> usize {
        self.z
    }

    pub fn set_x(&mut self, x: f32) {
        self.x = x
    }

    pub fn set_y(&mut self, y: f32) {
        self.y = y;
    }

    pub fn set_z(&mut self, z: usize) {
        self.z = z;
    }
}
