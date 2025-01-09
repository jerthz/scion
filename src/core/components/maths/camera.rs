pub(crate) struct DefaultCamera;

/// Mandatory component to add to the World to have anything rendered.
#[derive(Clone)]
pub struct Camera {
    pub(crate) left: f32,
    pub(crate) right: f32,
    pub(crate) top: f32,
    pub(crate) bottom: f32,
    pub(crate) near: f32,
    pub(crate) far: f32,
    pub(crate) dpi: f64,
}

impl Camera {
    /// Creates a camera with a viewport of size (width;height;depth). In general the same width and height of the window.
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            left: 0.,
            right: width,
            top: 0.,
            bottom: -1. * height,
            near: 0.0,
            far: 1000.,
            dpi: 1.0,
        }
    }
}
