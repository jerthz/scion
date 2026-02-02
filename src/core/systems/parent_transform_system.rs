use crate::core::components::Dirty;
use crate::core::components::maths::{
    transform::Transform,
};
use crate::core::world::{GameData, World};
use crate::graphics::components::tiles::tilemap::Tile;

pub(crate) fn dirty_transform_offset_system(data: &mut GameData) {
    for (_,(t,_, _)) in data.query_mut::<(&mut Transform, &Tile, &Dirty)>(){
        t.reset_dirty_offset();
    }
}
