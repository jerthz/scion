use std::{collections::HashMap, marker::PhantomData};

use log::debug;

use crate::graphics::components::ui::font::Font;
use crate::graphics::components::{material::Material, tiles::tileset::Tileset};

/// `AssetManager` is resource that will link assets to an asset ref to allow reusability of assets
#[derive(Default)]
pub struct AssetManager {
    /// A registry to keep track of where is located a tileset's texture. Key is a unique identifier for the tileset
    texture_path_registry: HashMap<String, String>,
    /// A registry to keep track of where is located an asset's atlas. Key is a unique identifier for the asset
    atlas_path_registry: HashMap<AssetType, String>,
    /// A registry to keep track of Asset ref already loaded in the engine. Key is a unique identifier for the asset
    asset_ref_registry: HashMap<AssetType, usize>,
    materials: HashMap<usize, Material>,
    fonts: HashMap<usize, Font>,
}

impl AssetManager {
    pub(crate) fn get_material_for_ref(&self, asset_ref: &AssetRef<Material>) -> Material {
        self.materials
            .get(&asset_ref.0)
            .expect("An asset has been requested but does not exist")
            .clone()
    }

    pub(crate) fn get_font_for_ref(&self, asset_ref: &AssetRef<Font>) -> Font {
        self.fonts
            .get(&asset_ref.0)
            .expect("An asset has been requested but does not exist")
            .clone()
    }

    pub(crate) fn get_atlas_path_for_asset_type(&self, asset_type: AssetType) -> String {
        self.atlas_path_registry
            .get(&asset_type)
            .expect("An asset path has been requested but does not exist")
            .to_string()
    }

    pub(crate) fn get_texture_for_tileset(&self, tileset_name: &str) -> String {
        self.texture_path_registry
            .get(tileset_name)
            .expect("A tileset texture path has been requested but does not exist")
            .to_string()
    }

    pub fn register_material(&mut self, material: Material) -> AssetRef<Material> {
        let next_ref = AssetRef(self.materials.keys().count(), PhantomData);
        self.materials.insert(next_ref.0, material);
        next_ref
    }

    /// Register a tileset as an asset into the assets registry and returns a reference to this asset.
    /// If a tileset already exists for this name, returns the existing asset ref instead
    pub fn register_tileset(&mut self, tileset: Tileset) -> AssetRef<Material> {
        let existing_ref = self.retrieve_asset_ref_for_tileset_name(&tileset.name);
        match existing_ref {
            Some(tileset_ref) => {
                debug!("Reusing existing tileset '{}'", tileset.name);
                tileset_ref
            }
            None => {
                debug!("Registering new tileset '{}' into the registry", tileset.name);
                let next_ref = AssetRef(self.materials.keys().count(), PhantomData);
                self.asset_ref_registry.insert(AssetType::Tileset(tileset.name.to_string()), next_ref.0);
                self.materials.insert(next_ref.0, Material::Tileset(tileset));
                next_ref
            }
        }
    }

    /// Register a path to find a tileset texture
    pub fn register_tileset_texture_path(&mut self, tileset_name: &str, path: &str) {
        self.texture_path_registry.insert(tileset_name.to_string(), path.to_string());
    }

    /// Register a path to load an asset's atlas. Usefull to let scion load/unload assets atlases when
    /// needed
    pub fn register_atlas_path(&mut self, identifier: AssetType, path: &str) {
        self.atlas_path_registry.insert(identifier, path.to_string());
    }

    pub fn register_atlases_path(&mut self, mut data: Vec<(AssetType, String)>) {
        data.drain(0..data.len()).for_each(|(asset_type, path)| self.register_atlas_path(asset_type, &path));
    }

    pub fn register_tileset_atlas_and_texture(&mut self, tileset_name: &str, atlas_path: &str, texture_path: &str) {
        self.register_atlas_path(AssetType::Tileset(tileset_name.to_string()), atlas_path);
        self.register_tileset_texture_path(tileset_name, texture_path);
    }

    pub fn register_tilesets_atlas_and_textures(&mut self, data: Vec<(&str, &str, &str)>) { // FIXME improve signature
        data.iter().for_each(|(tileset_name, atlas_path, texture_path)|
            self.register_tileset_atlas_and_texture(tileset_name, atlas_path, texture_path));
    }


    pub(crate) fn retrieve_asset_ref_for_tileset_name(&self, tileset_name: &str) -> Option<AssetRef<Material>>{
        self.asset_ref_registry.get(&AssetType::Tileset(tileset_name.to_string())).map(|id| AssetRef(*id, PhantomData))
    }

    pub fn register_font(&mut self, font: Font) -> AssetRef<Font> {
        let next_ref = AssetRef(self.fonts.keys().count(), PhantomData);
        self.fonts.insert(next_ref.0, font);
        next_ref
    }

    pub fn retrieve_tileset(&self, asset_ref: &AssetRef<Material>) -> Option<&Tileset> {
        match self.materials.get(&asset_ref.0) {
            None => None,
            Some(material) => match material {
                Material::Tileset(tileset) => Some(tileset),
                _ => None,
            },
        }
    }
}

#[derive(Clone, Eq, Copy, PartialEq, Hash)]
pub struct AssetRef<T: Send + Sync>(pub(crate) usize, pub(crate) PhantomData<T>);

impl Copy for AssetRef<Material>{}
impl Copy for AssetRef<Font>{}

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum AssetType {
    Tileset(String),
    Tilemap(String),
}

#[cfg(test)]
mod tests {
    use crate::core::resources::asset_manager::AssetManager;
    use crate::graphics::components::color::Color;
    use crate::graphics::components::material::Material;
    use crate::graphics::components::tiles::tileset::Tileset;

    #[test]
    fn register_material_test() {
        let mut manager = AssetManager::default();
        let asset_ref = manager.register_material(Material::Diffuse(Color::new(1, 1, 1, 1.)));
        assert_eq!(0, asset_ref.0);
        assert_eq!(1, manager.materials.len());

        let asset_ref = manager.register_material(Material::Diffuse(Color::new(2, 2, 2, 1.)));
        assert_eq!(1, asset_ref.0);
        assert_eq!(2, manager.materials.len());
    }

    #[test]
    fn register_tileset_test() {
        let mut manager = AssetManager::default();
        let asset_ref = manager.register_tileset(Tileset::new("test".to_string(),"test".to_string(), 1, 1, 1, 1));
        assert_eq!(0, asset_ref.0);
        assert_eq!(1, manager.materials.len());
    }
}
