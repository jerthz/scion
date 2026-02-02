use std::time::Duration;
use hecs::Entity;


use scion::graphics::components::animations::{Animation, AnimationModifier, Animations};
use scion::graphics::components::color::Color;
use scion::core::components::maths::coordinates::Coordinates;
use scion::core::components::maths::transform::Transform;
use scion::graphics::components::tiles::sprite::Sprite;
use scion::graphics::components::tiles::tileset::Tileset;
use scion::graphics::components::ui::font::Font;
use scion::graphics::components::ui::ui_text::UiText;
use scion::core::resources::audio::PlayConfig;
use scion::core::resources::inputs::types::{KeyCode};
use scion::core::resources::inputs::types::Input::Key;
use scion::core::scene::Scene;
use scion::core::world::{GameData, World};
use scion::utils::file::app_base_path_join;

use crate::scripts::background_parallax::{add_parallax_backgrounds};
use crate::level_scene::{CURRENT_LEVEL, LevelScene};

pub const ACTIVE_MENU_FLAG: &str = "current_scene_is_menu";

#[derive(Default)]
pub struct Menu {
    ship: Option<Entity>,
}

struct Ship {
    dir: bool,
}

impl Scene for Menu {
    fn on_start(&mut self, data: &mut GameData) {
        data.game_state_mut().set_bool(ACTIVE_MENU_FLAG, true);
        data.add_default_camera();
        add_parallax_backgrounds(data);

        let font_asset = data.assets_mut().register_font(Font::TrueType {
            font_path: app_base_path_join("examples/starlight-1961/assets/pixel.ttf")
        });

        // Menu
        {
            data.push((
                UiText::new("NEW  GAME".to_string(), font_asset).with_font_size(45).with_font_color(Color::new(255, 255, 255, 0.5)),
                Transform::new(Coordinates::new(225., 250.), 1.0, 0.),
                Animations::single("blink", Animation::looping(Duration::from_millis(800), vec![AnimationModifier::blink(1)]))
            ));

            data.push((
                UiText::new("LOAD  GAME".to_string(), font_asset).with_font_size(45).with_font_color(Color::new(255, 255, 255, 0.5)),
                Transform::new(Coordinates::new(225., 325.), 1.0, 0.),
                Animations::single("blink", Animation::new(Duration::from_millis(800), vec![AnimationModifier::blink(1)]))
            ));
            data.push((
                UiText::new("QUIT  GAME".to_string(), font_asset).with_font_size(45).with_font_color(Color::new(255, 255, 255, 0.5)),
                Transform::new(Coordinates::new(225., 400.), 1.0, 0.),
                Animations::single("blink", Animation::new(Duration::from_millis(800), vec![AnimationModifier::blink(1)]))
            ));

        }

        // Animated ship on menu
        {
            let ship_asset = data.assets_mut().register_tileset(Tileset::new("ship".to_string(),
                                                                             app_base_path_join("examples/starlight-1961/assets/space_ship.png"),
                                                                             5, 1, 32, 64));
            self.ship = Some(data.push((
                Ship { dir: true },
                Sprite::new(3),
                ship_asset,
                Transform::from_xy(330., 500.),
                Animations::single("float", Animation::looping(Duration::from_millis(300),
                                                               vec![AnimationModifier::sprite(vec![1, 2, 3, 2], 0)]))
            )
            ));
        }

        // Ambient music
        {
            let _r = data
                .audio()
                .play(app_base_path_join("examples/starlight-1961/assets/menu_music.ogg"),
                      PlayConfig { volume: 0.1, looped: true, category: None });
        }
    }

    fn on_update(&mut self, data: &mut GameData) {
        self.animate_main_menu_ship(data);
        if data.inputs().input_pressed(&Key(KeyCode::Enter)) {
            data.game_state_mut().set_text(CURRENT_LEVEL, "lvl1");
            data.scene_controller().switch::<LevelScene>();
        };
    }

    fn on_stop(&mut self, data: &mut GameData) {
        data.game_state_mut().set_bool(ACTIVE_MENU_FLAG, false);
        data.clear();
    }
}

impl Menu {
    fn animate_main_menu_ship(&mut self, data: &mut GameData) {
        let dir = {
            let (transform, ship) = data.entry_mut::<(&mut Transform, &mut Ship)>(self.ship.unwrap())
                .expect("Ship with transform must be mandatory here");

            if ship.dir && transform.global_translation().y() <= 495. {
                ship.dir = false;
            } else if !ship.dir && transform.global_translation().y() >= 505. {
                ship.dir = true;
            }
            ship.dir
        };
        data.commands().transform_commands.append_y(self.ship.unwrap(), if dir { -0.4 } else { 0.2 });
    }
}
