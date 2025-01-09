use std::path::Path;
use std::str::from_utf8;
use std::time::{Duration};
use std::vec;

use hecs::Entity;
use log::info;


use scion::graphics::components::animations::{Animation, AnimationModifier, Animations};
use scion::graphics::components::material::Material;
use scion::core::components::maths::camera::Camera;
use scion::core::components::maths::collider::{Collider, ColliderMask, ColliderType};
use scion::core::components::maths::hierarchy::Parent;

use scion::core::components::maths::transform::{Transform, TransformBuilder};
use scion::graphics::components::tiles::sprite::Sprite;
use scion::graphics::components::tiles::tilemap::{TileInfos, Tilemap, TilemapInfo, TilemapType};
use scion::graphics::components::tiles::tileset::Tileset;
use scion::core::resources::inputs::types::{Input, KeyCode};
use scion::core::scene::Scene;
use scion::core::world::{GameData, World};
use scion::utils::file::{app_base_path, read_file};
use scion::utils::maths::{Dimensions, Position, Vector};

use crate::character::{Character, get_animations_character};

#[derive(PartialEq, Default, Copy, Clone)]
pub enum Direction {
    LEFT,
    #[default]
    RIGHT,
    BOTTOM,
}

#[derive(Default)]
pub struct MainScene {
    pub char_entity: Option<Entity>,
    pub direction: Direction,
    pub idle: bool,
    pub running: bool,
    pub jumping: bool,
    pub jumping_released: bool,
    pub vertical_force: f32,
    pub jump_counter: usize,
}

impl Scene for MainScene {
    fn on_start(&mut self, data: &mut GameData) {
        add_background(data);
        add_structures(data);
        add_cherries(data);
        add_bananas(data);
        add_colliders(data);
        self.char_entity = Some(add_character(data));

        let mut camera_transform = Transform::from_xy(-202., -320.);
        camera_transform.set_global_translation_bounds(Some(0.), Some(512.), Some(0.), Some(0.));
        data.push((
            Camera::new(1024., 768.),
            camera_transform,
            Parent(self.char_entity.expect("Hero is mandatory")),
        ));

        self.idle = true;
    }

    fn on_update(&mut self, data: &mut GameData) {
        let collisions = self.compute_collision_directions(data, self.char_entity.unwrap());
        let left = data.inputs().input_pressed(&Input::Key(KeyCode::Left));
        let right = data.inputs().input_pressed(&Input::Key(KeyCode::Right));
        let running = left || right;
        let jumping = data.inputs().input_pressed(&Input::Key(KeyCode::Up)) && self.jumping_released;
        if jumping && !self.jumping && collisions.contains(&Direction::BOTTOM) {
            self.jumping_released = false;
            self.jumping = true;
            self.jump_counter = 1;
            self.vertical_force = 15.;
        };

        if !data.inputs().input_pressed(&Input::Key(KeyCode::Up)) {
            self.jumping_released = true;
        }

        let direction = if left { Some(Direction::LEFT) } else if right { Some(Direction::RIGHT) } else { None };
        let direction = if direction.is_some() { direction.expect("") } else { self.direction };
        let (world, resources) = data.split();
        if (!self.running || self.direction != direction) && running {
            for (_, (character, material, _transform, animations)) in world.query_mut::<(&Character, &mut Material, &mut Transform, &mut Animations)>() {
                if direction == Direction::LEFT {
                    *material = Material::Tileset(resources.assets_mut().retrieve_tileset(&character.running_left_asset_ref).expect("").clone());
                    animations.stop_all_animation(true);
                    animations.loop_animation("run left");
                } else if direction == Direction::RIGHT {
                    *material = Material::Tileset(resources.assets_mut().retrieve_tileset(&character.running_right_asset_ref).expect("").clone());
                    animations.stop_all_animation(true);
                    animations.loop_animation("run right");
                }
            }
            self.running = true;
            self.idle = false;
        } else if !running && (!self.idle || self.direction != direction) {
            for (_, (character, material, _transform, animations)) in world.query_mut::<(&Character, &mut Material, &mut Transform, &mut Animations)>() {
                if direction == Direction::LEFT {
                    *material = Material::Tileset(resources.assets_mut().retrieve_tileset(&character.idle_left_asset_ref).expect("").clone());
                    animations.stop_all_animation(true);
                    animations.loop_animation("idle left");
                } else if direction == Direction::RIGHT {
                    *material = Material::Tileset(resources.assets_mut().retrieve_tileset(&character.idle_right_asset_ref).expect("").clone());
                    animations.stop_all_animation(true);
                    animations.loop_animation("idle right");
                }
            }
            self.running = false;
            self.idle = true;
        }
        if right {
            for (_, (_character, _material, transform, _animations)) in world.query_mut::<(&Character, &mut Material, &mut Transform, &mut Animations)>() {
                if !collisions.contains(&Direction::RIGHT) {
                    transform.append_x(4.5);
                }
            }
        }
        if left {
            for (_, (_character, _material, transform, _animations)) in world.query_mut::<(&Character, &mut Material, &mut Transform, &mut Animations)>() {
                if !collisions.contains(&Direction::LEFT) {
                    transform.append_x(-4.5);
                }
            }
        }
        if self.vertical_force != 0.0 && (self.jumping || !collisions.contains(&Direction::BOTTOM)) {
            for (_, (character, material, transform, animations)) in world.query_mut::<(&Character, &mut Material, &mut Transform, &mut Animations)>() {
                *material = Material::Tileset(resources.assets_mut().retrieve_tileset(&character.jump_asset_ref).expect("").clone());
                animations.stop_all_animation(true);
                animations.loop_animation("jump");
                transform.append_y(-self.vertical_force);
            }
            self.running = false;
            self.idle = false;
        }
        self.direction = direction;
        if !self.jumping && !collisions.contains(&Direction::BOTTOM) {
            self.vertical_force -= 1.0;
        } else if !self.jumping && collisions.contains(&Direction::BOTTOM) {
            self.vertical_force = 0.0;
        }
        if self.jumping {
            self.vertical_force -= 1.0;
            if self.vertical_force <= 1.5 {
                self.vertical_force = 0.;
                self.jumping = false;
            }
        }
    }
}

impl MainScene {
    fn compute_collision_directions(&mut self, data: &mut GameData, char_entity: Entity) -> Vec<Direction> {
        let current_pos = *data.entry_mut::<&Transform>(char_entity).expect("").global_translation();
        let current_collisions = data.entry_mut::<&Collider>(char_entity).expect("").collisions();
        let mut res = Vec::new();
        let mut y_bottom = None;
        current_collisions.iter().for_each(|col| {
            if col.area().min_y() >= (current_pos.y() + 12.) {
                y_bottom = Some(col.area().min_y());
                res.push(Direction::BOTTOM);
            } else if col.area().max_x() >= current_pos.x() + 8.
                && col.area().max_x() < current_pos.x() + 32.
                && col.area().min_y() <= (current_pos.y() + 8. + 47.) {
                res.push(Direction::LEFT);
            } else if col.area().min_x() <= (current_pos.x() + 47.) && col.area().min_y() <= (current_pos.y() + 8. + 47.) {
                res.push(Direction::RIGHT);
            }
        });

        if y_bottom.is_some() {
            data.entry_mut::<&mut Transform>(char_entity).expect("").set_y(y_bottom.unwrap() - 9. - 45.);
            self.vertical_force = 0.;
        }
        res
    }
}

fn add_background(data: &mut GameData) {
    let tileset = Tileset::new("background".to_string(), app_base_path().join("examples/pixel-adventures/assets/background.png").get(), 32, 1, 64, 64);
    let tileset_ref = data.assets_mut().register_tileset(tileset);
    let tilemap_infos = TilemapInfo::new(
        Dimensions::new(12, 6, 1),
        TransformBuilder::new().with_scale(2.0).with_z(10).build(),
        tileset_ref,
        TilemapType::Standard
    );

    Tilemap::create(tilemap_infos, data, |_p| {
        TileInfos::new(Some(0), Some(Animation::looping(Duration::from_millis(2560), vec![AnimationModifier::sprite(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31], 1)])))
    });
}

fn add_structures(data: &mut GameData) {
    let file = read_file(Path::new(&app_base_path().join("examples/pixel-adventures/assets/map1_structures.csv").get()))
        .unwrap_or_default();
    let csv = from_utf8(file.as_slice()).expect("no");
    let tiles: Vec<usize> = csv.split(',').map(|v| v.parse::<usize>().expect("")).collect();

    let tileset = Tileset::new("terrain".to_string(), app_base_path().join("examples/pixel-adventures/assets/Terrain/Terrain.png").get(), 22, 11, 16, 16);
    let tileset_ref = data.assets_mut().register_tileset(tileset);
    let tilemap_infos = TilemapInfo::new(
        Dimensions::new(48, 24, 1),
        TransformBuilder::new().with_scale(2.0).with_z(2).build(),
        tileset_ref,
        TilemapType::Standard
    );

    Tilemap::create(tilemap_infos, data, |p| {
        TileInfos::new(compute_tile_nb(p, &tiles), None)
    });
}

fn compute_tile_nb(pos: &Position, tiles: &Vec<usize>) -> Option<usize> {
    // 48 x 24
    let index = pos.y() * 48 + pos.x();
    let tile = *tiles.get(index).expect("");
    if tile == 0 {
        None
    } else {
        Some(tile - 1)
    }
}

fn add_cherries(data: &mut GameData) {
    let tileset =
        Tileset::new("cherries".to_string(), app_base_path().join("examples/pixel-adventures/assets/Items/Fruits/Cherries.png").get(), 17, 1, 32, 32);
    let tileset_ref = data.assets_mut().register_tileset(tileset);

    let cherries_pos = vec![(484., 432.), (548., 432.), (612., 432.)];

    for (x, y) in cherries_pos {
        data.push((
            TransformBuilder::new().with_xy(x, y).with_z(3).with_scale(1.7).build(),
            Sprite::new(0),
            tileset_ref.clone(),
            Collider::new(
                ColliderMask::Custom("Item".to_string()),
                vec![ColliderMask::None],
                ColliderType::RectangleCollider(28, 28),
            ).with_offset(Vector::new(13., 13.)),
            Animations::single("loop", Animation::looping(Duration::from_millis(900), vec![AnimationModifier::sprite(vec![0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16], 0)]))
        ));
    }
}

fn add_bananas(data: &mut GameData) {
    let tileset =
        Tileset::new("Bananas".to_string(), app_base_path().join("examples/pixel-adventures/assets/Items/Fruits/Bananas.png").get(), 17, 1, 32, 32);
    let tileset_ref = data.assets_mut().register_tileset(tileset);

    let cherries_pos = vec![(245., 202.), (470., 202.), (695., 202.)];

    for (x, y) in cherries_pos {
        data.push((
            TransformBuilder::new().with_xy(x, y).with_z(3).with_scale(1.7).build(),
            Sprite::new(0),
            tileset_ref.clone(),
            Collider::new(
                ColliderMask::Custom("Item".to_string()),
                vec![ColliderMask::None],
                ColliderType::RectangleCollider(28, 28),
            ).with_offset(Vector::new(13., 11.)),
            Animations::single("loop", Animation::looping(Duration::from_millis(900), vec![AnimationModifier::sprite(vec![0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16], 0)]))
        ));
    }
}

fn add_character(data: &mut GameData) -> Entity {
    let tileset_idle_right =
        Tileset::new("Idle".to_string(), app_base_path().join("examples/pixel-adventures/assets/Main Characters/Ninja Frog/Idle.png").get(), 11, 1, 32, 32);
    let tileset_ref_idle_right = data.assets_mut().register_tileset(tileset_idle_right);

    let tileset_idle_left =
        Tileset::new("Idle2".to_string(), app_base_path().join("examples/pixel-adventures/assets/Main Characters/Ninja Frog/Idle2.png").get(), 11, 1, 32, 32);
    let tileset_ref_idle_left = data.assets_mut().register_tileset(tileset_idle_left);

    let tileset_run_right =
        Tileset::new("Run".to_string(), app_base_path().join("examples/pixel-adventures/assets/Main Characters/Ninja Frog/Run.png").get(), 12, 1, 32, 32);
    let tileset_ref_run_right = data.assets_mut().register_tileset(tileset_run_right);

    let tileset_run_left =
        Tileset::new("Run2".to_string(), app_base_path().join("examples/pixel-adventures/assets/Main Characters/Ninja Frog/Run2.png").get(), 12, 1, 32, 32);
    let tileset_ref_run_left = data.assets_mut().register_tileset(tileset_run_left);

    let tileset_jump =
        Tileset::new("Jump".to_string(), app_base_path().join("examples/pixel-adventures/assets/Main Characters/Ninja Frog/Jump.png").get(), 1, 1, 32, 32);
    let tileset_ref_jump = data.assets_mut().register_tileset(tileset_jump);

    data.push((
        TransformBuilder::new().with_xy(96., 681.).with_z(3).with_scale(1.7).build(),
        Sprite::new(0),
        tileset_ref_idle_right.clone(),
        Character {
            idle_right_asset_ref: tileset_ref_idle_right,
            idle_left_asset_ref: tileset_ref_idle_left,
            running_right_asset_ref: tileset_ref_run_right,
            running_left_asset_ref: tileset_ref_run_left,
            jump_asset_ref: tileset_ref_jump,
        },
        Collider::new(
            ColliderMask::Character,
            vec![ColliderMask::Landscape],
            ColliderType::RectangleCollider(39, 47),
        ).with_offset(Vector::new(8., 8.)),
        Animations::new(get_animations_character())
    ))
}

fn add_colliders(data: &mut GameData) {
    let colliders = vec![
        (0., 0., 1536, 32),
        (0., 0., 32, 768),
        (0., 736., 1536, 32),
        (1504., 0., 32, 768),
        (224., 288., 96, 32),
        (448., 288., 96, 32),
        (672., 288., 96, 32),
        (896., 672., 192, 64),
        (864., 480., 96, 32),
        (1024., 384., 64, 32),
        (1152., 320., 32, 32),
        (1248., 288., 256, 32),
        (32., 608., 96, 10),
        (32., 448., 96, 10),
        (32., 288., 96, 10),
        (1184., 672., 320, 64),
        (1248., 608., 256, 64),
        (1280., 544., 192, 64),
        (1344., 480., 64, 64),
        (224., 672., 96, 64),
        (320., 640., 160, 32),
        (480., 512., 192, 128),
        (672., 640., 32, 32),
        (704., 672., 96, 64),
    ];
    for (x, y, w, h) in colliders {
        data.push((
            Transform::from_xy(x, y),
            Collider::new(
                ColliderMask::Landscape,
                vec![ColliderMask::None],
                ColliderType::RectangleCollider(w, h),
            ),
        ));
    }
}