use std::collections::HashMap;

use log::trace;

use crate::core::components::maths::{
    hierarchy::{Children, Parent},
    transform::Transform,
};
use crate::core::world::{GameData, World};
use crate::graphics::components::tiles::tilemap::Tile;

/// System responsible of detecting when a transform's global coords should be computed again
/// based on the fact that the transform is flagged as dirty_child (IE when it's added to a parent)
pub(crate) fn dirty_child_system(data: &mut GameData) {
    let mut parent_to_check = Vec::new();
    for (child_entity, (t, p)) in data.query_mut::<(&mut Transform, Option<&Parent>)>() {
        if t.dirty_child {
            match p {
                None => t.dirty_child = false,
                Some(parent) => parent_to_check.push((child_entity, parent.entity())),
            }
        }
    }

    // GET all the parents that needs to be checked
    let mut parents_transform = HashMap::new();
    for (_, parent) in parent_to_check.iter() {
        if let Some(parent_transform) = data.entry::<&Transform>(*parent).unwrap().get() {
            parents_transform.insert(*parent, *parent_transform);
        }
    }

    while !parent_to_check.is_empty() {
        let mut tmp_vec = parent_to_check.clone();
        parent_to_check.clear();
        while let Some((child, parent)) = tmp_vec.pop() {
            match data.entry_mut::<&mut Transform>(child) {
                Ok(child_transform) => {
                    if parents_transform.contains_key(&parent) {
                        let parent_transform = parents_transform.get(&parent).unwrap();
                        // If the parent of the current entity is not a dirty child, then we can compute the current
                        // transform
                        if !parent_transform.dirty_child {
                            child_transform.dirty_child = false;
                            child_transform
                                .compute_global_from_parent(parent_transform.global_translation());
                            child_transform.compute_global_angle_from_parent(parent_transform.global_angle);
                            parents_transform.insert(child, *child_transform);
                        } else {
                            // Else we need to check the parent first, in the next iteration
                            parent_to_check.push((child, parent));
                        }
                    } else {
                        child_transform.dirty_child = false;
                        parents_transform.insert(child, *child_transform);
                    }
                }
                Err(_) => panic!("Error while retrieving child transform during internal system"),
            }
        }
    }
}

/// System responsible of detecting when a child transform should be computed again based on any parent
/// transform modification
pub(crate) fn dirty_transform_system(data: &mut GameData) {
    let mut first_iter = true;
    let mut transform_entities: Vec<(Transform, Vec<hecs::Entity>)> = Vec::new();
    while first_iter || !transform_entities.is_empty() {
        first_iter = false;
        while let Some((transform, entities)) = transform_entities.pop() {
            for entity in entities {
                if let Ok(child_transform) = data.entry_mut::<&mut Transform>(entity) {
                    trace!("Updating child Transform of entity {:?}, because parent was marked as dirty", entity);
                    child_transform.compute_global_from_parent(transform.global_translation());
                    child_transform.compute_global_angle_from_parent(transform.global_angle);
                }
            }
        }
        for (_, (parent_transform, children)) in
            data.query_mut::<(&mut Transform, Option<&Children>)>()
        {
            if let Some(children) = children {
                if parent_transform.dirty {
                    transform_entities.push((*parent_transform, children.0.clone()));
                }
            }
            parent_transform.dirty = false;
        }
    }
}

pub(crate) fn dirty_transform_offset_system(data: &mut GameData) {
    for (_,(t,_)) in data.query_mut::<(&mut Transform, &Tile)>(){
        t.reset_dirty_offset();
    }
}

#[cfg(test)]
mod tests {
    use crate::core::world::World;
    use crate::core::{components::maths::hierarchy::Parent};

    use super::*;

    #[test]
    fn dirty_parent_transform_test() {
        let mut world = GameData::default();

        let parent_transform = Transform::from_xyz(1., 1., 1);
        let child_transform = Transform::from_xyz(1., 1., 1);
        let child_of_child_transform = Transform::from_xyz(1., 1., 1);
        let parent = world.push((parent_transform,));
        let child = world.push((child_transform, Parent::new(parent)));
        let child_of_child = world.push((child_of_child_transform, Parent::new(child)));

        for (_, t) in world.query::<&Transform>().iter() {
            assert!(!t.dirty);
        }

        dirty_child_system(&mut world);
        dirty_transform_system(&mut world);

        assert_eq!(
            1.,
            world.entry::<&Transform>(parent).unwrap().get().unwrap().global_translation.x()
        );
        assert_eq!(
            2.,
            world.entry::<&Transform>(child).unwrap().get().unwrap().global_translation.x()
        );
        assert_eq!(
            3.,
            world
                .entry::<&Transform>(child_of_child)
                .unwrap()
                .get()
                .unwrap()
                .global_translation
                .x()
        );

        assert_eq!(
            1,
            world.entry::<&Transform>(parent).unwrap().get().unwrap().global_translation.z()
        );
        assert_eq!(
            2,
            world.entry::<&Transform>(child).unwrap().get().unwrap().global_translation.z()
        );
        assert_eq!(
            3,
            world
                .entry::<&Transform>(child_of_child)
                .unwrap()
                .get()
                .unwrap()
                .global_translation
                .z()
        );

        {
            let t = world.entry_mut::<&mut Transform>(parent).unwrap();
            t.append_translation(5.0, 1.0);
        }

        dirty_child_system(&mut world);
        dirty_transform_system(&mut world);

        assert_eq!(
            6.,
            world.entry::<&Transform>(parent).unwrap().get().unwrap().global_translation.x()
        );
        assert_eq!(
            7.,
            world.entry::<&Transform>(child).unwrap().get().unwrap().global_translation.x()
        );
        assert_eq!(
            8.,
            world
                .entry::<&Transform>(child_of_child)
                .unwrap()
                .get()
                .unwrap()
                .global_translation
                .x()
        );

        {
            let t = world.entry_mut::<&mut Transform>(parent).unwrap();
            t.set_z(5);
        }

        dirty_child_system(&mut world);
        dirty_transform_system(&mut world);

        assert_eq!(
            5,
            world.entry::<&Transform>(parent).unwrap().get().unwrap().global_translation.z()
        );
        assert_eq!(
            6,
            world.entry::<&Transform>(child).unwrap().get().unwrap().global_translation.z()
        );
        assert_eq!(
            7,
            world
                .entry::<&Transform>(child_of_child)
                .unwrap()
                .get()
                .unwrap()
                .global_translation
                .z()
        );

    }
}
