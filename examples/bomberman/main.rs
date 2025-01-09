use bomb_system::explosion_system;
use character_control_system::controller_system;
use hecs::Entity;
use scion::core::world::{GameData, World};
use scion::{
    config::{scion_config::ScionConfigBuilder, window_config::WindowConfigBuilder},
    {
        graphics::components::{
            animations::Animations,
            material::Material,
            tiles::{
                sprite::Sprite,
                tilemap::{TileInfos, Tilemap, TilemapInfo},
                tileset::Tileset,
            },
        },
        core::resources::asset_manager::AssetRef,
        core::scene::Scene,
    },
    utils::{file::app_base_path, maths::Dimensions},
    Scion,
};
use scion::core::components::maths::transform::Transform;
use scion::graphics::components::tiles::tilemap::TilemapType;
use crate::level_reader::Level;

mod bomb_animations;
mod bomb_system;
mod char_animations;
mod character_control_system;
mod level_reader;

#[derive(Default)]
struct MainScene {
    character: Option<Entity>,
}

#[derive(Default)]
pub struct BombermanRefs {
    tileset: Option<AssetRef<Material>>,
    tilemap_entity: Option<Entity>,
}

pub struct Bomb {
    pub pos_x: usize,
    pub pos_y: usize,
}

#[derive(Default)]
pub struct BombermanInfos {
    pub pos_x: usize,
    pub pos_y: usize,
}

impl Scene for MainScene {
    fn on_start(&mut self, data: &mut GameData) {
        let asset_ref = data.assets_mut().register_tileset(Tileset::new(
            "sokoban_tilesheet".to_string(),
            app_base_path().join("examples/bomberman/assets/sokoban_tilesheet.png").get(),
            13,
            9,
            64,
            64,
        ));

        let level = level_reader::read_level("examples/bomberman/assets/test_map.json");

        let tilemap_infos = TilemapInfo::new(
            Dimensions::new(level.width, level.height, level.tilemap.len()),
            Transform::from_xyz(0.,0.,10),
            asset_ref.clone(),
            TilemapType::Standard
        );

        let tilemap = Tilemap::create(tilemap_infos, data, |p| {
            TileInfos::new(
                Some(
                    *level
                        .tilemap
                        .get(p.z())
                        .unwrap()
                        .values
                        .get(p.y())
                        .unwrap()
                        .get(p.x())
                        .unwrap(),
                ),
                None,
            )
        });

        self.character = Some(data.push(create_char(asset_ref.clone(), &level)));

        data.add_default_camera();

        data.insert_resource(level);
        data.insert_resource(BombermanRefs {
            tileset: Some(asset_ref),
            tilemap_entity: Some(tilemap),
        });
    }
}

fn create_char(
    asset_ref: AssetRef<Material>,
    level: &Level,
) -> (Transform, Sprite, AssetRef<Material>, Animations, BombermanInfos) {
    (
        Transform::from_xyz(
            (level.character_x * 64) as f32,
            (level.character_y * 64) as f32,
            level.tilemap.len() + 2,
        ),
        Sprite::new(52),
        asset_ref,
        Animations::new(char_animations::get_animations()),
        BombermanInfos { pos_x: level.character_x, pos_y: level.character_y },
    )
}

fn main() {
    Scion::app_with_config(
        ScionConfigBuilder::new()
            .with_app_name("Scion's Bomberman".to_string())
            .with_window_config(
                WindowConfigBuilder::new().with_resizable(true).with_dimensions((640, 640)).get(),
            )
            .get(),
    )
    .with_scene::<MainScene>()
    .with_system(controller_system)
    .with_system(explosion_system)
    .run();
}
