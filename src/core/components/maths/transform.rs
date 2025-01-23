use crate::{core::components::maths::coordinates::Coordinates, utils::maths::Vector};

/// represents the bounds for a Transoform with min and max values
#[derive(Default, Debug, Copy, Clone)]
struct Bounds {
    pub(crate) min_x: Option<f32>,
    pub(crate) max_x: Option<f32>,
    pub(crate) min_y: Option<f32>,
    pub(crate) max_y: Option<f32>,
}

/// Component used by the renderer to know where and how to represent an object.
/// Default is position 0;0 with a scale of 1.0 and no angle.
#[derive(Debug, Copy, Clone)]
pub struct Transform {
    pub(crate) local_translation: Coordinates,
    pub(crate) global_translation: Coordinates,
    pub(crate) scale: f32,
    pub(crate) local_angle: f32,
    pub(crate) global_angle: f32,
    pub(crate) dirty: bool,
    pub(crate) dirty_child: bool,
    pub(crate) use_screen_as_origin: bool,
    pub(crate) dirty_offset: bool,
    bounds: Bounds,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            local_translation: Default::default(),
            global_translation: Default::default(),
            scale: 1.0,
            local_angle: 0.0,
            global_angle: 0.0,
            dirty: false,
            dirty_child: true,
            use_screen_as_origin: false,
            dirty_offset: false,
            bounds: Default::default(),
        }
    }
}

impl Transform {
    /// Creates a new transform using provided values.
    pub fn new(translation: Coordinates, scale: f32, angle: f32) -> Self {
        Self {
            local_translation: translation,
            global_translation: translation,
            scale,
            local_angle: angle,
            global_angle: angle,
            dirty: false,
            dirty_child: true,
            use_screen_as_origin: false,
            dirty_offset: false,
            bounds: Default::default(),
        }
    }

    /// Creates a transform from x and y values. Will use z value 0, scale 1. and default angle.
    pub fn from_xy(x: f32, y: f32) -> Self {
        Self::new(Coordinates::new(x, y), 1., 0.)
    }

    /// Creates a transform from x, y and z values. Will use scale 1. and default angle.
    pub fn from_xyz(x: f32, y: f32, z: usize) -> Self {
        Self::new(Coordinates::new_with_z(x, y, z), 1., 0.)
    }

    /// Append a translation to this transform's position
    pub fn append_translation(&mut self, x: f32, y: f32) {
        self.local_translation.x += x;
        self.local_translation.y += y;
        self.global_translation.x += x;
        self.global_translation.y += y;
        self.dirty = true;
        self.dirty_offset = true;
        self.handle_bounds();
    }

    /// Append a translation to this transform's position
    pub fn append_vector(&mut self, vector: Vector) {
        self.append_translation(vector.x, vector.y);
    }

    /// Appends the x val to the translation's x value
    pub fn append_x(&mut self, x: f32) {
        self.local_translation.x += x;
        self.global_translation.x += x;
        self.dirty = true;
        self.dirty_offset = true;
        self.handle_bounds();
    }

    /// Appends the y val to the translation's y value
    pub fn append_y(&mut self, y: f32) {
        self.local_translation.y += y;
        self.global_translation.y += y;
        self.dirty = true;
        self.dirty_offset = true;
        self.handle_bounds();
    }

    /// Move this transform down
    pub fn move_down(&mut self, y: f32) {
        self.local_translation.y += y;
        self.global_translation.y += y;
        self.dirty = true;
        self.dirty_offset = true;
        self.handle_bounds();
    }

    /// Append an angle to this transform's angle
    pub fn append_angle(&mut self, angle: f32) {
        self.local_angle += angle;
        self.global_angle += angle;
        self.dirty = true;
        self.dirty_offset = true;
    }

    pub fn set_angle(&mut self, angle: f32) {
        self.local_angle = angle;
        self.global_angle = angle;
        self.dirty = true;
        self.dirty_offset = true;
    }

    pub fn global_angle(&self) -> f32 {
        self.global_angle
    }

    /// Get the transform's coordinates
    pub fn translation(&self) -> &Coordinates {
        &self.local_translation
    }

    /// Get the global transform's translation
    pub fn global_translation(&self) -> &Coordinates {
        &self.global_translation
    }

    /// Change the scale value to a new one.
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale
    }


    /// Change the scale value to a new one.
    pub fn get_scale(&self) -> f32{
        self.scale
    }

    /// Change the z value in the local translation coordinates.
    pub fn set_z(&mut self, z: usize) {
        let z_diff = self.global_translation.z - self.local_translation.z;
        self.local_translation.z = z;
        self.global_translation.z = z + z_diff;
        self.dirty = true;
        self.dirty_offset = true;
    }

    /// Change the x value in the local translation coordinates.
    pub fn set_x(&mut self, x: f32) {
        let x_diff = self.global_translation.x - self.local_translation.x;
        self.local_translation.x = x;
        self.global_translation.x = x + x_diff;
        self.dirty = true;
        self.dirty_offset = true;
    }

    /// Change the y value  in the local translation coordinates.
    pub fn set_y(&mut self, y: f32) {
        let y_diff = self.global_translation.y - self.local_translation.y;
        self.local_translation.y = y;
        self.global_translation.y = y + y_diff;
        self.dirty = true;
        self.dirty_offset = true;
    }

    /// Configure the minimum global x position for this transform to be min_x
    pub fn set_min_x(&mut self, min_x: Option<f32>) {
        self.bounds.min_x = min_x;
        self.handle_bounds();
    }

    /// Configure the maximum global x position for this transform to be max_x
    pub fn set_max_x(&mut self, max_x: Option<f32>) {
        self.bounds.max_x = max_x;
        self.handle_bounds();
    }

    /// Configure the minimum global y position for this transform to be min_x
    pub fn set_min_y(&mut self, min_y: Option<f32>) {
        self.bounds.min_y = min_y;
        self.handle_bounds();
    }

    /// Configure the maximum global y position for this transform to be max_x
    pub fn set_max_y(&mut self, max_y: Option<f32>) {
        self.bounds.max_y = max_y;
        self.handle_bounds();
    }

    pub fn set_use_screen_as_origin(&mut self, new_value: bool) {
        self.use_screen_as_origin = new_value;
        self.handle_bounds();
    }

    /// Configure the minimum and maximum global values of x and y
    pub fn set_global_translation_bounds(
        &mut self,
        min_x: Option<f32>,
        max_x: Option<f32>,
        min_y: Option<f32>,
        max_y: Option<f32>,
    ) {
        self.bounds.min_x = min_x;
        self.bounds.max_x = max_x;
        self.bounds.min_y = min_y;
        self.bounds.max_y = max_y;
        self.handle_bounds();
    }

    /// Computes the global_translation using the parent as origin
    pub(crate) fn compute_global_from_parent(&mut self, parent_translation: &Coordinates) {
        let mut new_global = Coordinates::new_with_z(parent_translation.x, parent_translation.y, parent_translation.z);
        new_global.x += self.local_translation.x;
        new_global.y += self.local_translation.y;
        new_global.z += self.local_translation.z;
        self.global_translation = new_global;
        self.dirty = true;
        self.dirty_offset = true;
        self.handle_bounds();
    }

    pub(crate) fn compute_global_angle_from_parent(&mut self, parent_angle: f32){
        self.global_angle = self.local_angle + parent_angle;
    }

    fn handle_bounds(&mut self) {
        if let Some(min_x) = self.bounds.min_x {
            if self.global_translation.x < min_x {
                self.global_translation.set_x(min_x);
            }
        }

        if let Some(max_x) = self.bounds.max_x {
            if self.global_translation.x > max_x {
                self.global_translation.set_x(max_x);
            }
        }

        if let Some(min_y) = self.bounds.min_y {
            if self.global_translation.y < min_y {
                self.global_translation.set_y(min_y);
            }
        }

        if let Some(max_y) = self.bounds.max_y {
            if self.global_translation.y > max_y {
                self.global_translation.set_y(max_y);
            }
        }
    }
}

pub struct TransformBuilder {
    transform: Transform,
}

impl Default for TransformBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TransformBuilder {
    pub fn new() -> Self {
        Self { transform: Transform::default() }
    }

    pub fn with_xy(mut self, x: f32, y: f32) -> Self {
        let translation = Coordinates::new(x, y);
        self.transform.local_translation = translation;
        self.transform.global_translation = translation;
        self
    }
    pub fn with_translation(mut self, x: f32, y: f32, z: usize) -> Self {
        let translation = Coordinates::new_with_z(x, y, z);
        self.transform.local_translation = translation;
        self.transform.global_translation = translation;
        self
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.transform.scale = scale;
        self
    }

    pub fn with_z(mut self, z: usize) -> Self {
        self.transform.local_translation.z = z;
        self.transform.global_translation.z = z;
        self
    }

    pub fn with_angle(mut self, angle: f32) -> Self {
        self.transform.local_angle = angle;
        self.transform.global_angle = angle;
        self
    }

    pub fn with_screen_as_origin(mut self) -> Self {
        self.transform.use_screen_as_origin = true;
        self
    }

    pub fn build(self) -> Transform {
        self.transform
    }
}

#[cfg(test)]
mod tests {
    use crate::core::components::maths::transform::{Coordinates, Transform};

    #[test]
    fn compute_global_from_parent_test() {
        let parent_translation = Coordinates::new(1., 2.);
        let mut child_transform = Transform::new(Coordinates::new(5., 3.), 1., 1.);

        assert_eq!(5., child_transform.global_translation.x);
        assert_eq!(3., child_transform.global_translation.y);

        child_transform.compute_global_from_parent(&parent_translation);

        assert_eq!(6., child_transform.global_translation.x);
        assert_eq!(5., child_transform.global_translation.y);
    }

    #[test]
    fn modify_transform_should_set_dirty_test() {
        let mut transform = Transform::new(Coordinates::new(5., 3.), 1., 1.);
        assert!(!transform.dirty);

        transform.append_translation(1., 1.);
        assert!(transform.dirty);

        transform.dirty = false;

        transform.move_down(1.);
        assert!(transform.dirty);

        transform.dirty = false;

        transform.append_x(1.);
        assert!(transform.dirty);

        transform.dirty = false;

        transform.append_y(1.);
        assert!(transform.dirty);

        transform.dirty = false;

        transform.compute_global_from_parent(&Coordinates::new(1., 2.));
        assert!(transform.dirty);
    }

    #[test]
    fn handle_bounds_test() {
        let mut t = Transform::default();
        t.set_min_x(Some(1.));
        assert_eq!(1., t.global_translation.x);
        t.append_x(-6.);
        assert_eq!(1., t.global_translation.x);
    }
}
