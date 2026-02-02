use std::collections::{HashMap, HashSet};
use std::ops::AddAssign;
use hecs::Entity;
use crate::core::components::maths::transform::{Transform, TransformOperation};
use crate::utils::maths::Vector;

#[derive(Default)]
pub struct CommandBuffer {
    pub transform_commands: TransformCommand
}

impl CommandBuffer {
    pub(crate) fn drain(&mut self) -> (HashMap<Entity, TransformOperation>,){
        (std::mem::take(&mut self.transform_commands.transforms),)
    }
    pub(crate) fn drain_parent_modified(&mut self) -> (HashMap<Entity, (Vec<Entity>, Transform)>,){
        (std::mem::take(&mut self.transform_commands.parent_modified),)
    }
}

#[derive(Default)]
pub struct TransformCommand {
    pub(crate) transforms: HashMap<Entity, TransformOperation>,
    pub(crate) parent_modified: HashMap<Entity, (Vec<Entity>, Transform)>,
}

impl TransformCommand {
    pub fn set_x(&mut self, entity: Entity, x: f32) {
        self.transforms
            .entry(entity)
            .or_insert_with(TransformOperation::default)
            .x = Some(x);
    }

    pub fn set_y(&mut self, entity: Entity, y: f32) {
        self.transforms
            .entry(entity)
            .or_insert_with(TransformOperation::default)
            .y = Some(y);
    }

    pub fn set_z(&mut self, entity: Entity, z: usize) {
        self.transforms
            .entry(entity)
            .or_insert_with(TransformOperation::default)
            .z = Some(z);
    }

    pub fn append_x(&mut self, entity: Entity, x: f32) {
        self.transforms
            .entry(entity)
            .or_insert_with(TransformOperation::default)
            .delta_x.get_or_insert(0.0).add_assign(x);
    }
    pub fn append_y(&mut self, entity: Entity, y: f32) {
        self.transforms
            .entry(entity)
            .or_insert_with(TransformOperation::default)
            .delta_y.get_or_insert(0.0).add_assign(y);
    }

    pub fn append_translation(&mut self, entity: Entity, x: f32, y: f32) {
        let transformation = self.transforms
            .entry(entity)
            .or_insert_with(TransformOperation::default);
        transformation.delta_x.get_or_insert(0.0).add_assign(x);
        transformation.delta_y.get_or_insert(0.0).add_assign(y);
    }
    pub fn append_vector(&mut self, entity: Entity, vector: Vector) {
        self.append_translation(entity, vector.x, vector.y);
    }

    pub fn append_angle(&mut self, entity: Entity, angle: f32) {
        self.transforms
            .entry(entity)
            .or_insert_with(TransformOperation::default)
            .delta_angle.get_or_insert(0.0).add_assign(angle);
    }
    pub fn set_angle(&mut self, entity: Entity, angle: f32) {
        self.transforms
            .entry(entity)
            .or_insert_with(TransformOperation::default)
            .angle = Some(angle);
    }

    pub fn set_scale(&mut self, entity: Entity, scale: f32) {
        self.transforms
            .entry(entity)
            .or_insert_with(TransformOperation::default)
            .scale = Some(scale);
    }

}
