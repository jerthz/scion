pub mod camera;
pub mod collider;
pub mod coordinates;
pub mod hierarchy;
pub mod transform;
pub mod padding;
mod color_picking;

/// `Pivot` tells where the pivot point of a component is
#[derive(Debug, Copy, Clone)]
pub enum Pivot {
    /// Pivot is on the top left corner of the shape
    TopLeft,
    /// Pivot is on the center of the shape
    Center,
    Custom(f32, f32)
}





