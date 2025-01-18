use crate::graphics::components::color::Color;
use hecs::Entity;
use std::collections::{HashMap, VecDeque};

#[derive(Default)]
pub(crate) struct ColorPickingStorage{
    pub(crate) entity_to_color : HashMap<Entity, u32>,
    pub(crate) color_to_entity : HashMap<u32, Entity>,
    pub(crate) available_colors_indexes : VecDeque<u32>,
    pub(crate) next_color_index : u32,
}

impl ColorPickingStorage {
    pub fn entity_registered(&self, entity: Entity) -> bool {
        self.entity_to_color.contains_key(&entity)
    }

    pub fn color_registered(&self, color: &Color) -> bool {
        self.color_to_entity.contains_key(&color.as_u32())
    }

    pub fn color(&self, entity: Entity) -> Option<Color> {
        if !self.entity_to_color.contains_key(&entity){
            None
        } else {
            let index = self.entity_to_color.get(&entity).expect("Unreachable color u32 despite check");
            Some(Color::color_from_u32(*index))
        }
    }

    pub fn create_picking(&mut self, entity: Entity) -> Color{
        if self.entity_registered(entity) {
            return Color::color_from_u32(*self.entity_to_color.get(&entity).expect("Unreachable color u32 despite check"));
        }
        let next_index = self.available_colors_indexes.pop_front().unwrap_or(self.next_index());
        self.entity_to_color.insert(entity, next_index);
        self.color_to_entity.insert(next_index, entity);
        Color::color_from_u32(next_index)
    }

    pub fn get_entity_from_color(&mut self, color: &Color) -> Option<Entity> {
        if self.color_registered(color) {
            Some(*self.color_to_entity.get(&color.as_u32()).expect("Unreachable entity from color u32 despite check"))
        }else{
            None
        }
    }

    fn next_index(&mut self) -> u32 {
        let mut next = self.next_color_index;
        if next == 0 {
            next = next + 1;
        }
        self.next_color_index = next +1;
        next
    }
}