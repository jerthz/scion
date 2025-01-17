use hecs::Entity;
use log::info;
use scion::core::world::{GameData, World};
use scion::{
    graphics::components::{
        tiles::tileset::Tileset,
        ui::{font::Font, ui_image::UiImage, ui_text::UiText},
    },
    core::resources::time::TimerType,
    core::scene::Scene,
};
use scion::graphics::components::color::Color;
use scion::graphics::components::material::Material;
use scion::core::components::maths::padding::Padding;
use scion::core::components::maths::transform::Transform;
use scion::graphics::components::ui::ui_button::UiButton;
use scion::graphics::components::ui::ui_input::UiInput;
use scion::core::resources::inputs::types::{Input, KeyCode};
use scion::graphics::components::shapes::rectangle::Rectangle;
use crate::{asset_path, resources::TetrisResource};

#[derive(Default)]
pub struct MainScene {
    score: Option<Entity>,
}

impl Scene for MainScene {
    fn on_start(&mut self, data: &mut GameData) {
        add_main_ui_mask(data);
        add_ui_top_overflow(data);

        self.score = Some(add_score_ui(data));
        data.add_default_camera();
        let _r = data.timers().add_timer("piece", TimerType::Cyclic, 0.5);
        let _r = data.timers().add_timer("action_reset_timer", TimerType::Manual, 0.2);
        let mut tetris = TetrisResource::default();
        tetris.asset = Some(data.assets_mut().register_tileset(Tileset::new(
            "tetris_asset".to_string(),
            asset_path().join("blocs.png").get(),
            8,
            1,
            32,
            32,
        )));
        data.insert_resource(tetris);
    }

    fn on_update(&mut self, data: &mut GameData) {
        let pause_click = data.inputs().input_pressed_event(&Input::Key(KeyCode::P));
        let current_pause_state = data.game_state().get_bool("pause");

        if pause_click {
            data.game_state_mut().set_bool("pause", !current_pause_state);
        }
    }
}

fn add_score_ui(data: &mut GameData) -> Entity {
    // First we add an UiText to the world
    let font = Font::TrueType {
        font_path: asset_path().join("Arial.ttf").get(),
    };
    let font_asset = data.assets_mut().register_font(font);

    let font2 = Font::Bitmap {
        texture_path: asset_path().join("font.png").get(),
        chars: "0123456789ACEOPRSULI".to_string(),
        texture_columns: 20.,
        texture_lines: 1.,
        width: 21.,
        height: 27.,
    };
    let font_asset_2 = data.assets_mut().register_font(font2);

    let txt = UiText::new("abcdefghijklmnop".to_string(), font_asset.clone()).with_font_size(12);
    let transform = Transform::from_xyz(394., 250., 2);

    data.push((txt, transform))

}

fn add_main_ui_mask(data: &mut GameData) {
    let path = asset_path().join("ui.png").get();
    let image = Rectangle::new(544., 704., None);

    let mut t = Transform::default();
    t.set_z(10);
    t.set_use_screen_as_origin(true);
    data.push((image, t, Material::Texture(path)));
}

fn add_ui_top_overflow(data: &mut GameData) {
    let path = asset_path().join("ui_overflow_top.png").get();
    let image = Rectangle::new(324., 32., None);

    let mut t = Transform::default();
    t.set_z(0);
    t.append_translation(32., 0.);
    t.set_use_screen_as_origin(true);
    data.push((image, t, Material::Texture(path)));
}
