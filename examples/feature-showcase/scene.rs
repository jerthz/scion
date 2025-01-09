use std::collections::HashMap;
use std::time::Duration;
use hecs::Entity;
use log::info;
use scion::core::components::maths::transform::{Transform, TransformBuilder};
use scion::graphics::components::tiles::atlas::importer::load_tilemap;
use scion::core::resources::asset_manager::AssetType;

use scion::core::scene::Scene;
use scion::core::world::{GameData, World};
use scion::graphics::components::animations::{Animation, AnimationModifier, Animations};
use scion::graphics::components::tiles::atlas::data;
use scion::utils::file::app_base_path_join;
use scion::utils::maths::Vector;

#[derive(Default)]
pub struct DemoScene {
    entity: Option<Entity>,
    last: usize,
}

pub struct Tm;

impl Scene for DemoScene {
    fn on_start(&mut self, data: &mut GameData) {
        data.add_default_camera();

        data.assets_mut().register_tileset_atlas_and_texture(
            "demo-ply", &app_base_path_join("examples/feature-showcase/assets/demo-ply.json"),
            &app_base_path_join("examples/feature-showcase/assets/Isometric-SpriteSheet.png"),
        );
        data.assets_mut().register_tileset_atlas_and_texture(
            "demo-ply2", &app_base_path_join("examples/feature-showcase/assets/demo-ply2.json"),
            &app_base_path_join("examples/feature-showcase/assets/Isometric-SpriteSheet-green.png"),
        );
        data.assets_mut().register_tileset_atlas_and_texture(
            "demo-ply3", &app_base_path_join("examples/feature-showcase/assets/demo-ply3.json"),
            &app_base_path_join("examples/feature-showcase/assets/Isometric-SpriteSheet-orange.png"),
        );
        data.assets_mut().register_tileset_atlas_and_texture(
            "demo-ply4", &app_base_path_join("examples/feature-showcase/assets/demo-ply4.json"),
            &app_base_path_join("examples/feature-showcase/assets/Isometric-SpriteSheet-purple.png"),
        );
        data.assets_mut().register_tileset_atlas_and_texture(
            "demo-ply5", &app_base_path_join("examples/feature-showcase/assets/demo-ply5.json"),
            &app_base_path_join("examples/feature-showcase/assets/Isometric-SpriteSheet-red.png"),
        );

        data.assets_mut().register_atlas_path(AssetType::Tilemap("test-lvl".to_string()), &app_base_path_join("examples/feature-showcase/assets/test-lvl.json"));
        data.assets_mut().register_atlas_path(AssetType::Tilemap("test-lvl2".to_string()), &app_base_path_join("examples/feature-showcase/assets/test-lvl2.json"));
        data.assets_mut().register_atlas_path(AssetType::Tilemap("test-lvl3".to_string()), &app_base_path_join("examples/feature-showcase/assets/test-lvl3.json"));
        data.assets_mut().register_atlas_path(AssetType::Tilemap("test-lvl4".to_string()), &app_base_path_join("examples/feature-showcase/assets/test-lvl4.json"));
        data.assets_mut().register_atlas_path(AssetType::Tilemap("test-lvl5".to_string()), &app_base_path_join("examples/feature-showcase/assets/test-lvl5.json"));
        let (_, entity) = load_tilemap(data, "test-lvl", TransformBuilder::new().with_xy(260., 560.).with_z(1).with_scale(2.0).build());
        load_tilemap(data, "test-lvl2", TransformBuilder::new().with_xy(420., 180.).with_z(2).with_scale(2.0).build());
        load_tilemap(data, "test-lvl3", TransformBuilder::new().with_xy(100., 180.).with_z(2).with_scale(2.0).build());
        load_tilemap(data, "test-lvl4", TransformBuilder::new().with_xy(420., 340.).with_z(0).with_scale(2.0).build());
        load_tilemap(data, "test-lvl5", TransformBuilder::new().with_xy(100., 340.).with_z(0).with_scale(2.0).build());
        let mut animations = HashMap::new();

        animations.insert("up".to_string(), Animation::new(Duration::from_millis(1500),
                                               vec![AnimationModifier::transform(40, Some(Vector::new(0., -400.)), None, None)]));

        animations.insert("down".to_string(), Animation::new(Duration::from_millis(1500),
                                                 vec![AnimationModifier::transform(40, Some(Vector::new(0., 400.)), None, None)]));

        data.add_components(entity, (Animations::new(animations), Tm {}));
        self.entity = Some(entity);
    }

    fn on_update(&mut self, data: &mut GameData) {
        let animations = data.entry_mut::<&mut Animations>(self.entity.unwrap()).expect("");
        if !animations.any_animation_running() {
            if self.last == 0 {
                animations.run_animation("up");
                self.last = 1;
            } else {
                animations.run_animation("down");
                self.last = 0;
            }
        }
    }
}