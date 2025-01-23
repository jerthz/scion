use hecs::Entity;
use std::time::Duration;
use log::{debug, info};

use scion::graphics::components::animations::{Animation, AnimationModifier, Animations};
use scion::graphics::components::color::Color;
use scion::graphics::components::material::Material;
use scion::core::components::maths::camera::Camera;
use scion::core::components::maths::hierarchy::Parent;
use scion::core::components::maths::transform::{Transform, TransformBuilder};
use scion::graphics::components::shapes::rectangle::Rectangle;
use scion::graphics::components::tiles::sprite::Sprite;
use scion::graphics::components::tiles::tilemap::{TileInfos, Tilemap, TilemapInfo, TilemapType};
use scion::graphics::components::tiles::tileset::Tileset;
use scion::core::resources::audio::PlayConfig;
use scion::core::resources::inputs::types::KeyCode;
use scion::core::resources::time::TimerType;
use scion::core::scene::Scene;
use scion::core::world::{GameData, Resources, World};
use scion::graphics::components::tiles::SPRITE_ANIMATION_PRELOAD;
use scion::utils::file::app_base_path;
use scion::utils::maths::{Dimensions, Position};

use crate::animations;
use crate::animations::switch_scene_animation;
use crate::level_reader::read_level;

pub struct GlobalResource {
    level: String,
    start_x: usize,
    start_y: usize,
    direction: String,
}

impl Default for GlobalResource {
    fn default() -> Self {
        Self {
            level: "main_house_bedroom".to_string(),
            start_x: 1,
            start_y: 5,
            direction: "TOP".to_string(),
        }
    }
}

#[derive(Default, Debug)]
pub struct MainCharacter {
    pub left: bool,
    pub right: bool,
    pub top: bool,
    pub bottom: bool,
}

#[derive(Default)]
pub struct MainScene {
    tilemap: Option<Entity>,
    player: Option<Entity>,
    fader: Option<Entity>,
    current_width: usize,
    current_height: usize,
    is_switching: bool,
}

impl Scene for MainScene {
    fn on_start(&mut self, data: &mut GameData) {
        if !data.contains_resource::<GlobalResource>() {
            data.insert_resource(GlobalResource::default());
            let _r = data.timers().add_timer("SceneSwitch", TimerType::Manual, 0.5);
            play_music(data);
        }
        self.fader = Some(data.push((
            Rectangle::new(384., 336., None),
            TransformBuilder::new().with_translation(0., 0., 0).with_screen_as_origin().build(),
            Material::Diffuse(Color::new(255, 255, 255, 1.)),
            Animations::new(switch_scene_animation()),
        )));

        let (level, start_x, start_y, direction) = {
            let global_resource = data.get_resource_mut::<GlobalResource>().unwrap();
            (
                format!(
                    "examples/new-bark-town/assets/scenes/{}.json",
                    global_resource.level
                ),
                global_resource.start_x,
                global_resource.start_y,
                global_resource.direction.to_string(),
            )
        };

        let tilemap = self.load_map(level, data);
        self.tilemap = Some(tilemap);
        let char = add_character(data, start_x, start_y, &direction);
        let camera_transform = Transform::from_xy(-192., -168.);
        data.push((Camera::new(384., 336.), camera_transform, Parent(char)));
        self.player = Some(char);
    }

    fn on_update(&mut self, data: &mut GameData) {
        if self.is_switching {
            let anim = data.entry_mut::<&mut Animations>(*self.fader.as_ref().unwrap()).unwrap();
            if !anim.any_animation_running() {
                let _r = data.remove(self.tilemap.unwrap());
                let _r = data.remove(self.player.unwrap());
                let _r = data.remove(*self.fader.as_ref().unwrap());
                data.scene_controller().switch::<MainScene>();
            }
            return;
        }

        if !data.timers().get_timer("SceneSwitch").unwrap().ended() {
            return;
        }

        let (pos_x, pos_y, delta_x, delta_y) = {
            let transform = data
                .entry_mut::<&mut Transform>(self.player.unwrap())
                .expect("Player is mandatory");
            (
                transform.translation().x() as usize / 48,
                transform.translation().y() as usize / 48,
                transform.translation().x() as usize % 48,
                transform.translation().y() as usize % 48,
            )
        };

        let (world, resources) = data.split();
        let (left, right, top, bottom) = {
            (
                pos_x > 0
                    && Tilemap::retrieve_pathing(
                        world,
                        self.tilemap.unwrap(),
                        &Position::new(pos_x - 1, pos_y, 0),
                        &resources.assets(),
                    )
                    .is_some(),
                pos_x < self.current_width - 1
                    && Tilemap::retrieve_pathing(
                        world,
                        self.tilemap.unwrap(),
                        &Position::new(pos_x + 1, pos_y, 0),
                        &resources.assets(),
                    )
                    .is_some(),
                pos_y > 0
                    && Tilemap::retrieve_pathing(
                        world,
                        self.tilemap.unwrap(),
                        &Position::new(pos_x, pos_y - 1, 0),
                        &resources.assets(),
                    )
                    .is_some(),
                pos_y < self.current_height - 1
                    && Tilemap::retrieve_pathing(
                        world,
                        self.tilemap.unwrap(),
                        &Position::new(pos_x, pos_y + 1, 0),
                        &resources.assets(),
                    )
                    .is_some(),
            )
        };

        {
            let player = world
                .entry_mut::<&mut MainCharacter>(self.player.unwrap())
                .expect("Player is mandatory");
            player.left = left;
            player.right = right;
            player.top = top;
            player.bottom = bottom;
        }

        {
            let tilemap = world.entry_mut::<&mut Tilemap>(self.tilemap.unwrap()).unwrap();
            let event = tilemap.retrieve_event(&Position::new(pos_x, pos_y, 0));
            if let Some(e) = event {
                if (e.event_type().eq("DOOR") && delta_x == 0 && delta_y == 0)
                    || e.event_type().eq("EXIT") && resources.inputs().key_pressed(&KeyCode::Down)
                {
                    let (level_name, target_x, target_y, direction) = (
                        e.properties().get("go_to").unwrap().to_string(),
                        e.properties().get("target_x").unwrap().parse::<usize>().unwrap(),
                        e.properties().get("target_y").unwrap().parse::<usize>().unwrap(),
                        e.properties().get("direction").unwrap().to_string(),
                    );
                    {
                        let mut level = resources
                            .get_resource_mut::<GlobalResource>()
                            .expect("Global resource is mandatory");
                        level.level = level_name;
                        level.start_x = target_x;
                        level.start_y = target_y;
                        level.direction = direction;
                    }

                    play_door_effect(resources);
                    self.is_switching = true;
                }
            }
        }
        if self.is_switching {
            let anim = world.entry_mut::<&mut Animations>(*self.fader.as_ref().unwrap()).unwrap();
            anim.run_animation("FADE_IN");

            resources.timers().get_timer("SceneSwitch").unwrap().reset();
        }
    }
}

impl MainScene {
    fn load_map(&mut self, level_name: String, data: &mut GameData) -> Entity {
        debug!("Starting to load new level named {}", level_name);
        let asset_ref = data.assets_mut().register_tileset(
            Tileset::from_atlas("examples/new-bark-town/assets/nbt_atlas.json", "examples/new-bark-town/assets/nbt.png").unwrap(),
        );
        let mut level = read_level(level_name.as_str());
        let mut scale = Transform::default();
        scale.set_scale(3.0);
        scale.set_z(5);
        let tilemap_infos = TilemapInfo::new(
            Dimensions::new(level.map.width, level.map.height, level.map.layers.len()),
            scale,
            asset_ref.clone(),
            TilemapType::Standard
        );

        self.current_width = level.map.width;
        self.current_height = level.map.height;
        let tilemap = Tilemap::create(tilemap_infos, data, |p| {
            TileInfos::new(Some(level.map.tile_at(p)))
                .with_animation(get_animation_for_tile(level.map.tile_at(p)))
                .with_event(level.event_at(p))
        });

        info!("Successfully loaded level {}", level_name);

        tilemap
    }
}

fn play_music(data: &mut GameData) {
    let _r = data
        .audio()
        .play(new_bark_town_theme(), PlayConfig { volume: 0.2, looped: true, category: None });
}

fn play_door_effect(resources: &mut Resources) {
    let _r = resources
        .audio()
        .play(door_effect(), PlayConfig { volume: 0.2, looped: false, category: None });
}

fn get_animation_for_tile(i: usize) -> Option<(String, Animation)> {
    if i == 73 {
        Some((SPRITE_ANIMATION_PRELOAD.to_string(), Animation::looping(
            Duration::from_millis(1500),
            vec![AnimationModifier::sprite(vec![74, 73, 74, 73, 74], 73)],
        )))
    } else if i == 85 {
        Some((SPRITE_ANIMATION_PRELOAD.to_string(), Animation::looping(
            Duration::from_millis(1500),
            vec![AnimationModifier::sprite(vec![86, 85, 86, 85, 86], 85)],
        )))
    } else if i == 96 {
        Some((SPRITE_ANIMATION_PRELOAD.to_string(), Animation::looping(
            Duration::from_millis(2000),
            vec![AnimationModifier::sprite(vec![51, 75, 99, 96, 51], 96)],
        )))
    } else if i == 108 {
        Some((SPRITE_ANIMATION_PRELOAD.to_string(), Animation::looping(
            Duration::from_millis(2000),
            vec![AnimationModifier::sprite(vec![63, 87, 111, 108, 63], 108)],
        )))
    } else if i == 97 {
        Some((SPRITE_ANIMATION_PRELOAD.to_string(), Animation::looping(
            Duration::from_millis(2000),
            vec![AnimationModifier::sprite(vec![52, 76, 100, 97, 52], 97)],
        )))
    } else if i == 109 {
        Some((SPRITE_ANIMATION_PRELOAD.to_string(), Animation::looping(
            Duration::from_millis(2000),
            vec![AnimationModifier::sprite(vec![64, 88, 112, 109, 64], 109)],
        )))
    } else {
        None
    }
}

fn add_character(
    data: &mut GameData,
    start_x: usize,
    start_y: usize,
    direction: &String,
) -> Entity {
    let asset_ref = data.assets_mut().register_tileset( Tileset::new(
        "character".to_string(),
        "examples/new-bark-town/assets/character.png".to_string(),
        10,
        8,
        16,
        16,
    ));
    let mut animations = Animations::new(animations::char_animations());
    if direction.eq("BOTTOM") {
        animations.run_animation("MOVE_BOTTOM");
    }
    data.push((
        TransformBuilder::new()
            .with_translation(start_x as f32 * 16. * 3., start_y as f32 * 16. * 3., 1)
            .with_scale(3.)
            .build(),
        Sprite::new(get_direction_sprite(direction)),
        asset_ref,
        animations,
        MainCharacter::default(),
    ))
}

fn get_direction_sprite(direction: &String) -> usize {
    if direction.eq("BOTTOM") {
        1
    } else {
        4
    }
}

fn new_bark_town_theme() -> String {
    app_base_path().join("examples/new-bark-town/assets/newbarktown.ogg").get()
}

fn door_effect() -> String {
    app_base_path().join("examples/new-bark-town/assets/door.ogg").get()
}
