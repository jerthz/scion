use std::{collections::HashMap, ops::Range};

use hecs::Entity;
use serde::{Deserialize, Serialize};
use wgpu::{util::BufferInitDescriptor, PrimitiveTopology};

use crate::core::components::maths::hierarchy::Parent;
use crate::core::components::maths::transform::Transform;
use crate::core::resources::asset_manager::AssetManager;
use crate::core::world::{SubWorld, World};
use crate::{
    core::resources::asset_manager::AssetRef,
    graphics::rendering::Renderable2D,
    graphics::components::{
        animations::{Animation, Animations},
        material::Material,
        tiles::sprite::Sprite,
    },
    utils::maths::{Dimensions, Position},
};

#[derive(Debug)]
pub struct Pathing {
    pathing_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TileEvent {
    event_type: String,
    properties: HashMap<String, String>,
}

impl TileEvent {
    pub fn new(event_type: String, properties: HashMap<String, String>) -> Self {
        if event_type.as_str() == "" {
            panic!("An event must have a type");
        }
        Self { event_type, properties }
    }

    pub fn event_type(&self) -> String {
        self.event_type.to_string()
    }

    pub fn properties(&mut self) -> &mut HashMap<String, String> {
        &mut self.properties
    }
}


#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum TilemapType {
    Standard,
    Isometric{
        offset_x : OffsetMultiplier,
        offset_y : OffsetMultiplier,
        offset_z : OffsetMultiplier,
    },
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct OffsetMultiplier{
    pub x_multiplier: f32,
    pub y_multiplier: f32,
    pub z_multiplier: f32
}

pub struct Tile {
    pub(crate) position: Position,
    pub(crate) tilemap: Entity,
}

impl Tile{
    pub fn get_position(&self) -> &Position {
        &self.position
    }

    pub fn get_tilemap_entity(&self) -> Entity {
        self.tilemap
    }
}

/// Struct representing a single tile in a tilemap. Needs to be returned in the
/// tile resolver function when creating a tilemap.
pub struct TileInfos {
    tile_nb: Option<usize>,
    animation: Option<Animation>,
    event: Option<TileEvent>,
    pathing_type: Option<String>,
}

impl TileInfos {
    /// Creates a new TileInfos struct
    pub fn new(tile_nb: Option<usize>, animation: Option<Animation>) -> Self {
        Self { tile_nb, animation, event: None, pathing_type: None }
    }

    /// Adds an event to the current tile.
    pub fn with_event(mut self, event: Option<TileEvent>) -> Self {
        self.event = event;
        self
    }

    /// Force a pathing value for this position. If this exists, the engine won't check in the
    /// tileset atlas to retrive pathing value for this tile
    pub fn with_pathing(mut self, pathing: String) -> Self {
        self.pathing_type = Some(pathing);
        self
    }
}

/// `TilemapInfo` regroups all the needed informations that a Tilemap needs to be created
pub struct TilemapInfo {
    dimensions: Dimensions,
    transform: Transform,
    tileset_ref: AssetRef<Material>,
    tilemap_type: TilemapType
}

impl TilemapInfo {
    pub fn new(
        dimensions: Dimensions,
        transform: Transform,
        tileset_ref: AssetRef<Material>,
        tilemap_type: TilemapType
    ) -> Self {
        Self { dimensions, transform, tileset_ref, tilemap_type }
    }
}

/// `Tilemap` is `Scion` convenience component to create a full multi layered tilemap.
pub struct Tilemap {
    tile_entities: HashMap<Position, Entity>,
    events: HashMap<Position, TileEvent>,
    tileset_ref: AssetRef<Material>,
    tilemap_type: TilemapType,
    width: usize,
    height: usize,
    depth: usize,

}

impl Tilemap {
    pub(crate) fn new(tileset_ref: AssetRef<Material>, tilemap_type: TilemapType, dimensions: &Dimensions) -> Self {
        Self { tile_entities: Default::default(), events: HashMap::default(), tileset_ref, tilemap_type, width: dimensions.width(), height: dimensions.height(), depth: dimensions.depth() }
    }

    /// Convenience fn to create a tilemap and add it to the world.
    /// tile_resolver is a function taking a 3D position as parameter and a `TileInfos`
    /// as a return. This way, the tilemap knows exactly what to add at which coordinates.
    pub fn create<F>(infos: TilemapInfo, world: &mut impl World, mut tile_resolver: F) -> Entity
    where
        F: FnMut(&Position) -> TileInfos,
    {
        let self_entity = Tilemap::create_tilemap(world, infos.tileset_ref, infos.transform, infos.tilemap_type, &infos.dimensions);

        for x in 0..infos.dimensions.width() {
            for y in 0..infos.dimensions.height() {
                for z in 0..infos.dimensions.depth() {
                    let position = Position::new(x, y, z);
                    let tile_infos = tile_resolver(&position);

                    let entity = world.push((
                        Tile { position: position.clone(), tilemap: self_entity },
                        Parent(self_entity),
                    ));

                    if let Some(tile_nb) = tile_infos.tile_nb {
                        let _r = world.add_components(entity, (Sprite::new(tile_nb),));
                    }

                    if let Some(animation) = tile_infos.animation {
                        let _r = world.add_components(
                            entity,
                            (Animations::single("TileAnimation", animation),),
                        );
                    }

                    if let Some(pathing) = tile_infos.pathing_type {
                        let _r = world.add_components(entity, (Pathing { pathing_type: pathing },));
                    }

                    if let Some(event) = tile_infos.event {
                        world
                            .entry_mut::<&mut Tilemap>(self_entity)
                            .unwrap()
                            .events
                            .insert(position.clone(), event);
                    }

                    world
                        .entry_mut::<&mut Tilemap>(self_entity)
                        .unwrap()
                        .tile_entities
                        .insert(position, entity);
                }
            }
        }

        self_entity
    }

    /// Try to modify the sprite's tile at a given position
    pub fn modify_sprite_tile(
        world: &mut impl World,
        tilemap_entity: Entity,
        tile_position: Position,
        new_tile_nb: usize,
    ) {
        let tile = world
            .entry_mut::<&mut Tilemap>(tilemap_entity)
            .unwrap()
            .tile_entities
            .get(&tile_position)
            .as_ref()
            .map(|e| **e);
        if let Some(tile) = tile {
            let entry = world.entry_mut::<&mut Sprite>(tile);
            if let Ok(sprite) = entry {
                sprite.set_tile_nb(new_tile_nb);
            } else {
                let _r = world.add_components(tile, (Sprite::new(new_tile_nb),));
            }
        }
    }

    pub fn retrieve_sprite_tile(
        world: &mut impl World,
        entity: Entity,
        tile_position: &Position,
    ) -> Option<usize> {
        let tile = world
            .entry_mut::<&mut Tilemap>(entity)
            .unwrap()
            .tile_entities
            .get(tile_position)
            .as_ref()
            .map(|e| **e);
        if let Some(tile) = tile {
            return world.entry::<&Sprite>(tile).unwrap().get().map(|s| s.get_tile_nb());
        }
        None
    }

    /// Retrieves the pathing value associated with this position in the tilemap
    pub fn retrieve_pathing(
        world: &mut SubWorld,
        entity: Entity,
        tile_position: &Position,
        asset_manager: &AssetManager,
    ) -> Option<String> {
        let (tile, tileset_ref) = {
            let mut res = world.entry::<&Tilemap>(entity).unwrap();
            let tilemap = res.get();
            (
                tilemap
                    .as_ref()
                    .unwrap()
                    .tile_entities
                    .get(tile_position)
                    .as_ref()
                    .map(|e| **e),
                tilemap.as_ref().unwrap().tileset_ref.clone(),
            )
        };
        if let Some(tile) = tile {
            if let Ok(mut entry) = world.entry::<&Pathing>(tile) {
                if let Some(path_value) = entry.get() {
                    return Some(path_value.pathing_type.to_string());
                }
            }
        }

        if let Some(tileset) = asset_manager.retrieve_tileset(&tileset_ref) {
            if let Some(sprite) = Tilemap::retrieve_sprite_tile(world, entity, tile_position) {
                let val = tileset.pathing.iter().find(|(_k, v)| v.contains(&sprite));
                if let Some(entry) = val {
                    return Some(entry.0.to_string());
                }
            }
        }
        None
    }

    /// Retrieves the mutable tile event associated with this position in the tilemap
    pub fn retrieve_event(&mut self, tile_position: &Position) -> Option<&mut TileEvent> {
        self.events.get_mut(tile_position)
    }

    fn create_tilemap(
        world: &mut impl World,
        tileset_ref: AssetRef<Material>,
        transform: Transform,
        tilemap_type: TilemapType,
        dimensions: &Dimensions,
    ) -> Entity {
        world.push((Self::new(tileset_ref.clone(), tilemap_type, dimensions), tileset_ref, transform))
    }

    pub fn is_isometric(&self)-> bool{
        match self.tilemap_type {
            TilemapType::Standard => { false }
            TilemapType::Isometric { .. } => { true }
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn offset_x_multiplier_y(&self) -> f32 {
        if let TilemapType::Isometric { offset_x, .. } = self.tilemap_type{
            return offset_x.y_multiplier;
        }
        0.
    }

    pub fn offset_x_multiplier_x(&self) -> f32 {
        if let TilemapType::Isometric { offset_x, .. } = self.tilemap_type{
            return offset_x.x_multiplier;
        }
        0.
    }

    pub fn offset_x_multiplier_z(&self) -> f32 {
        if let TilemapType::Isometric { offset_x, .. } = self.tilemap_type{
            return offset_x.z_multiplier;
        }
        0.
    }

    pub fn offset_y_multiplier_y(&self) -> f32 {
        if let TilemapType::Isometric {offset_y, .. } = self.tilemap_type{
            return offset_y.y_multiplier;
        }
        0.
    }

    pub fn offset_y_multiplier_x(&self) -> f32 {
        if let TilemapType::Isometric { offset_y, .. } = self.tilemap_type{
            return offset_y.x_multiplier;
        }
        0.
    }

    pub fn offset_y_multiplier_z(&self) -> f32 {
        if let TilemapType::Isometric { offset_y, .. } = self.tilemap_type{
            return offset_y.z_multiplier;
        }
        0.
    }

}

impl Renderable2D for Tilemap {
    fn vertex_buffer_descriptor(&mut self, _material: Option<&Material>) -> BufferInitDescriptor {
        todo!()
    }

    fn indexes_buffer_descriptor(&self) -> BufferInitDescriptor {
        todo!()
    }

    fn range(&self) -> Range<u32> {
        todo!()
    }

    fn topology() -> PrimitiveTopology {
        PrimitiveTopology::TriangleList
    }

    fn dirty(&self) -> bool {
        todo!()
    }

    fn set_dirty(&mut self, _is_dirty: bool) {
        todo!()
    }
}