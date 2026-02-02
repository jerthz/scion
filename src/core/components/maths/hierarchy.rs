use std::process::Child;
use hecs::Entity;
use crate::core::world::{ScionWorld, World};

/// A component creating a parent link to the wrapped entity
#[derive(Debug)]
pub struct Parent(Entity);

impl Parent {
    pub fn new(entity: Entity) -> Self {
        Self(entity)
    }

    pub fn entity(&self) -> Entity {
        self.0
    }
}

/// A component creating a link to the wrapped entities
/// This component will be automatically added to an entity by Scion
/// if a component references this entity with a [`Parent`] component
#[derive(Debug)]
pub struct Children(pub(crate) Vec<Entity>);


/// Adds a link to the children (current entity of the parent entity to the wrapped entity.
/// This function is called to handle the case where an entity is spawned with a [`Parent`] component.
pub(crate) fn init_parent_children_link(subworld: &mut hecs::World, potential_children: Entity) {
    let mut should_add_children = (false, None);
    if let Ok(entry) = subworld.query_one_mut::<Option<&Parent>>(potential_children) {
        if entry.is_some() {
            should_add_children = (true, Some(entry.unwrap().0));
        }
    }

    if should_add_children.0 {
        let parent_entity = should_add_children.1.expect("");

        if !subworld.contains(parent_entity) {
            panic!("Parent entity {:?} referenced by {:?} does not exist in the world", parent_entity, potential_children);
        }

        let mut ok = false;
        if let Ok(mut children) = subworld.get::<&mut Children>(parent_entity) {
            if !children.0.contains(&potential_children) {
            children.0.push(potential_children);
            }
            ok = true;
        }
        if !ok {
            let _ = subworld.insert(parent_entity, (Children(vec![potential_children]),));
        }
    }
}

pub(crate) fn retrieve_parent(subworld: &mut hecs::World, potential_children: Entity) -> Option<Entity>{
    let mut current_parent = None;
    if let Ok(entry) = subworld.query_one_mut::<Option<&Parent>>(potential_children) {
        if entry.is_some() {
            current_parent = Some(entry.unwrap().0);
        }
    }
    current_parent
}

pub(crate) fn retrieve_children(subworld: &mut hecs::World, potential_parent: Entity) ->  Option<Vec<Entity>>{
    let mut current_children = None;
    if let Ok(entry) = subworld.query_one_mut::<Option<&Children>>(potential_parent) {
        if entry.is_some() {
            current_children = Some(entry.unwrap().0.clone());
        }
    }
    current_children
}

pub(crate) fn update_parent_if_needed(subworld: &mut hecs::World,
                                      old_parent: Option<Entity>,
                                      old_child: Entity) {
   if let Some(parent_entity) = old_parent{
       if let Ok(mut children) = subworld.get::<&mut Children>(parent_entity) {
           if children.0.contains(&old_child) {
               children.0.retain(|x| x != &old_child);
           }
       }
   }
}

pub(crate) fn update_children_if_needed(subworld: &mut hecs::World, old_children: Option<Vec<Entity>>) {
    if let Some(children_entities) = old_children{
        children_entities.iter().for_each(|child| {
            let _r = subworld.remove_one::<Parent>(*child);
        });
    }
}