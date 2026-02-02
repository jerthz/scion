use hecs::Entity;
use log::info;
use scion::core::world::{GameData, World};
use scion::{
    graphics::components::{
        tiles::tileset::Tileset,
        ui::{font::Font, ui_text::UiText},
    },
    core::resources::time::TimerType,
    core::scene::Scene,
};
use scion::graphics::components::color::Color;
use scion::graphics::components::material::Material;
use scion::core::components::maths::padding::Padding;
use scion::core::components::maths::transform::{Transform, TransformBuilder};
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

    let mut input = UiInput::new(200, 200, font_asset.clone())
        .with_font_size(16)
        .with_tab_index(1)
        .with_font_color(Color::new_rgb(0, 0, 0));
    input.set_text("Coucou".to_string());

    data.push((
        input,
        Transform::from_xyz(394., 330., 1)
    ));

    let background_asset = data.assets_mut().register_material(Material::Diffuse(Color::new_rgb(200, 200, 200)));
    let hover_asset = data.assets_mut().register_material(Material::Diffuse(Color::new_rgb(160, 160, 160)));
    let clicked_asset = data.assets_mut().register_material(Material::Diffuse(Color::new_rgb(120, 120, 120)));

    let button = UiButton::new(70, 30, font_asset.clone())
        .with_font_size(16)
        .with_tab_index(13)
        .with_font_color(Color::new_rgb(0, 0, 0))
        .with_padding(Padding::new(Some(8.), Some(10.)))
        .with_background_material(background_asset)
        .with_clicked_material(clicked_asset)
        .with_hover_material(hover_asset)
        .with_text("Button")
        .with_on_click_action(|_res| { info!("Test click at {:?}", std::time::SystemTime::now()); });

    data.push((
        button,
        Transform::from_xyz(394., 430., 3)
    ));

    let font2 = Font::Bitmap {
        texture_path: asset_path().join("font.png").get(),
        chars: "0123456789ACEOPRSULI".to_string(),
        texture_columns: 20.,
        texture_lines: 1.,
        width: 21.,
        height: 27.,
    };
    let font_asset_2 = data.assets_mut().register_font(font2);

    let txt = UiText::new("SCORE".to_string(), font_asset_2.clone());
    let transform = Transform::from_xyz(394., 250., 2);

    data.push((txt, transform));

    let txt = UiText::new("".to_string(), font_asset.clone())
        .sync_value(|res| res.get_resource::<TetrisResource>().unwrap().get_score()).with_font_size(32);

    let transform = Transform::from_xyz(394., 290., 2);
    data.push((txt, transform))
}

fn add_main_ui_mask(data: &mut GameData) {
    let path = asset_path().join("ui.png").get();
    let image = Rectangle::new(544., 704., None);

    let t = TransformBuilder::default().with_z(10).with_screen_as_origin().build();Transform::default();
    data.push((image, t, Material::Texture(path)));
}

fn add_ui_top_overflow(data: &mut GameData) {
    let path = asset_path().join("ui_overflow_top.png").get();
    let image = Rectangle::new(324., 32., None);

    let t = TransformBuilder::default().with_z(0).with_translation(32.0, 0., 0).with_screen_as_origin().build();
    data.push((image, t, Material::Texture(path)));
}
