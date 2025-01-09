


pub mod importer {
    use std::path::Path;
    use std::time::Duration;

    use base64::Engine;
    use base64::prelude::BASE64_STANDARD;
    use hecs::Entity;
    use log::{debug, error};

    use crate::graphics::components::animations::{Animation, AnimationModifier};
    use crate::graphics::components::material::Material;
    use crate::core::components::maths::transform::{Transform};
    use crate::graphics::components::tiles::atlas::data::{TilemapAtlas, TilesetAtlas};
    use crate::graphics::components::tiles::tilemap::{TileInfos, Tilemap, TilemapInfo, TilemapType};
    use crate::graphics::components::tiles::tileset::Tileset;
    use crate::core::resources::asset_manager::{AssetRef, AssetType};
    use crate::core::world::{GameData, Resources};
    use crate::utils::maths::Dimensions;

    /// Import a tilemap from a .scion format located at `path`, into a TilemapAtlas
    pub fn import_tilemap(path: &str) -> TilemapAtlas {
        match crate::utils::file::read_file(Path::new(path)) {
            Ok(file) => {
                let mut tilemap: TilemapAtlas = serde_json::from_slice(file.as_slice()).expect(format!("Error while reading file at path {:?}", path).as_str());
                tilemap.layers.iter_mut().for_each(|l| {
                    let tiles: Vec<Vec<isize>> = serde_json::from_slice(BASE64_STANDARD.decode(l.tiles_encoded.as_ref().unwrap()).expect("").as_slice()).expect("");
                    l.tiles = tiles;
                    l.tiles_encoded = None;
                });
                debug!("Tilemap at path {} has been loaded", path);
                tilemap
            }
            Err(e) => panic!("{:?}", e)
        }
    }

    /// Import a tileset from a .scion format located at `path`, into a TilesetAtlas
    pub fn import_tileset(path: &str) -> TilesetAtlas {
        match crate::utils::file::read_file(Path::new(path)) {
            Ok(file) => {
                let tileset: TilesetAtlas = serde_json::from_slice(file.as_slice()).expect(format!("Error while reading file at path {:?}", path).as_str());
                debug!("Tileset at path {} has been loaded", path);
                tileset
            }
            Err(e) => {
                error!("{:?}", e);
                std::panic::panic_any(e)
            }
        }
    }


    /// Load a tilemap into the world.
    /// To use this function, you need to have an entry into the registry for the `AssetType::Tilemap(name)` (see `AssetManager`)
    /// You also need to have an entry in the registry for each AssetType::Tileset used in the tilemap
    /// Scion will load the tilesets into the asset manager or reuse them if they exist
    pub fn load_tilemap(data: &mut GameData, name: &str, tilemap_transform: Transform) -> (TilemapAtlas, Entity) {
        let (subworld, resources) = data.split();

        let tilemap_path = resources.assets().get_atlas_path_for_asset_type(AssetType::Tilemap(name.to_string()));
        let tilemap = import_tilemap(&tilemap_path);

        if tilemap.tilesets.len() > 1 {
            panic!("Multiple tilesets not yet implemented !");
        }

        let tileset_refs: Vec<AssetRef<Material>> = tilemap.tilesets.iter().map(|t| {
            load_tileset(resources, &t.name)
        }).collect();

        let tileset_ref = tileset_refs.first().unwrap();
        let asset_manager = resources.assets();
        let opt_tileset = asset_manager.retrieve_tileset(tileset_ref);
        let tileset = opt_tileset.unwrap();

        let tilemap_info = create_tilemap_info(&tilemap, tileset_refs, tilemap_transform);
        let entity = Tilemap::create(tilemap_info, subworld, |p| {
            let tile = tilemap.tile_at(p);
            let animation = compute_animation(&tile, tileset);
            TileInfos::new(tile, animation)
        });
        (tilemap, entity)
    }

    fn compute_animation(tile: &Option<usize>, tileset: &Tileset) -> Option<Animation> {
        match tile {
            None => None,
            Some(tile) => {
                let tile_config = tileset.tiles.get(tile);
                match tile_config {
                    None => None,
                    Some(config) => {
                        if let Some(vec_anim) = &config.animation {
                            let time: usize = vec_anim.iter().map(|a| a.duration).sum();
                            let frames: Vec<usize> = vec_anim.iter().map(|a| a.tile_id).collect();
                            let end_tile = *frames.last().unwrap();
                            debug!("Duration of animation : {:?}", time);
                            Some(Animation::looping(Duration::from_millis(time as u64), vec![AnimationModifier::sprite(frames, end_tile)]))
                        } else {
                            None
                        }
                    }
                }
            }
        }
    }

    fn load_tileset(resources: &mut Resources, name: &str) -> AssetRef<Material> {
        let existing_ref = resources.assets().retrieve_asset_ref_for_tileset_name(name);
        match existing_ref {
            None => {
                let tileset_path = resources.assets().get_atlas_path_for_asset_type(AssetType::Tileset(name.to_string()));
                let tileset_atlas = import_tileset(&tileset_path);
                let texture_path = resources.assets().get_texture_for_tileset(name);
                resources.assets_mut().register_tileset(tileset_atlas.into_tileset(texture_path))
            }
            Some(tileset_ref) => {
                debug!("Reusing existing tileset '{}'", name);
                tileset_ref
            }
        }
    }

    fn create_tilemap_info(tilemap: &TilemapAtlas, mut tileset_refs: Vec<AssetRef<Material>>, tilemap_transform: Transform) -> TilemapInfo {
        TilemapInfo::new(Dimensions::new(tilemap.width, tilemap.height, tilemap.layers.len()),
                         tilemap_transform,
                         tileset_refs.remove(0),
                         tilemap.tilemap_type.as_ref().unwrap_or(&TilemapType::Standard).clone()) // TODO : When support for multi tileset is developped
    }
}

pub mod data {
    use std::collections::{HashMap, HashSet};

    use serde::{Deserialize, Serialize};

    use crate::core::components::maths::coordinates::Coordinates;
    use crate::graphics::components::tiles::tilemap::TilemapType;
    use crate::graphics::components::tiles::tileset::Tileset;
    use crate::utils::maths::Position;

    #[derive(Serialize, Deserialize)]
    pub struct TilesetAtlas {
        pub(crate) name: String,
        pub(crate) total_tiles: usize,
        pub(crate) width: usize,
        pub(crate) height: usize,
        pub(crate) tile_width: usize,
        pub(crate) tile_height: usize,
        pub(crate) pathing: Option<HashMap<String, HashSet<usize>>>,
        pub(crate) tiles: HashMap<usize, TileConfig>,
    }

    impl TilesetAtlas {
        pub fn into_tileset(self, texture_path: String) -> Tileset {
            Tileset {
                name: self.name,
                width: self.width,
                height: self.height,
                tile_width: self.tile_width,
                tile_height: self.tile_height,
                texture: texture_path,
                pathing: self.pathing.unwrap_or_default(),
                tiles: self.tiles,
            }
        }

        pub fn tile_config_for(&self, tile_id: usize) -> &TileConfig{
            self.tiles.get(&tile_id).as_ref().unwrap()
        }

    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TileConfig {
        pub(crate) animation: Option<Vec<TileAnimationFrame>>,
        pub(crate) objects: Vec<TileObject>,
    }

    impl TileConfig{
        pub fn objects(&self) -> &Vec<TileObject>{
            &self.objects
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TileAnimationFrame {
        pub(crate) tile_id: usize,
        pub(crate) duration: usize,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TileObject {
        class: TileObjectClass,
        shape_type: TileObjectShapeType,
        position: Coordinates,
        polygon: Option<Vec<Coordinates>>,
        rectangle: Option<TileRectangle>,
    }

    impl TileObject{
        pub fn get_class(&self) -> &TileObjectClass{
            &self.class
        }
        pub fn get_position(&self) -> &Coordinates{
            &self.position
        }
        pub fn is_rect(&self) -> bool{
            self.rectangle.is_some()
        }
        pub fn get_rect(&self) -> &TileRectangle{
            self.rectangle.as_ref().unwrap()
        }
        pub fn get_polygon(&self) -> &Vec<Coordinates>{
            self.polygon.as_ref().unwrap()
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TileRectangle {
        width: f32,
        height: f32,
    }

    impl TileRectangle {
        pub fn width(&self) -> f32{
            self.width
        }

        pub fn height(&self) -> f32{
            self.height
        }
    }


    #[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
    #[serde(tag = "tag", content = "content")]
    pub enum TileObjectClass {
        CollisionArea,
        Item,
        Door,
        Trigger,
        Custom(String),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum TileObjectShapeType {
        Polygon,
        Rectangle,
        Point,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct TilemapAtlas {
        pub(crate) width: usize,
        pub(crate) height: usize,
        pub(crate) tile_width: usize,
        pub(crate) tile_height: usize,
        pub(crate) properties: HashMap<String, String>,
        pub(crate) layers: Vec<TilemapLayer>,
        pub(crate) objects: Vec<TileObject>,
        pub(crate) tilesets: Vec<TiledTilemapTileset>,
        pub(crate) tilemap_type: Option<TilemapType>,
    }

    impl TilemapAtlas{
        pub fn get_objects(&self) -> &Vec<TileObject>{
            &self.objects
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct TilemapLayer {
        pub(crate) name: String,
        pub(crate) tiles_encoded: Option<String>,
        #[serde(skip_serializing)]
        #[serde(skip_deserializing)]
        pub(crate) tiles: Vec<Vec<isize>>,
        pub(crate) properties: HashMap<String, String>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct TiledTilemapTileset {
        pub(crate) index: usize,
        pub(crate) total_tiles: usize,
        pub(crate) name: String,
    }

    impl TilemapAtlas {
        pub(crate) fn tile_at(&self, position: &Position) -> Option<usize> {
            if position.x() > self.width || position.y() > self.height || position.z() > self.layers.len() {
                panic!("Position of requested tile is not coherent with tilemap informations");
            }
            let tile = *self.layers.get(position.z()).unwrap().tiles.get(position.y()).unwrap().get(position.x()).unwrap();
            if tile >= 0 {
                Some(tile as usize)
            } else {
                None
            }
        }
    }
}