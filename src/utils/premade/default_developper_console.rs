use hecs::Entity;
use log::info;
use crate::core::components::maths::hierarchy::Parent;
use crate::core::components::maths::transform::Transform;
use crate::core::package::Package;
use crate::core::resources::inputs::types::{Input, KeyCode};
use crate::core::world::{GameData, World};
use crate::graphics::components::color::Color;
use crate::graphics::components::material::Material;
use crate::graphics::components::tiles::atlas::data;
use crate::graphics::components::ui::font::Font;
use crate::graphics::components::ui::ui_image::UiImage;
use crate::graphics::components::ui::ui_input::UiInput;
use crate::ScionBuilder;
use crate::utils::file::app_base_path;

pub struct DummyDeveloperConsole;

pub(crate) struct ScionDeveloperConsole;
pub(crate) struct ScionDeveloperConsoleResource {
    pub(crate) currently_displayed: bool,
    pub(crate) current_entity: Option<Entity>,
}

impl Package for DummyDeveloperConsole {
    fn prepare(&self, data: &mut GameData) {
        data.resources.insert_resource(ScionDeveloperConsoleResource { currently_displayed: false, current_entity: None });
    }

    fn load(&self, builder: ScionBuilder) -> ScionBuilder {
        builder.with_system(dummy_developer_console_system)
    }
}

///
pub fn dummy_developer_console_system(data: &mut GameData) {
    let open = data.inputs().input_pressed_event(&Input::Key(KeyCode::F12));
    let close = data.inputs().input_pressed(&Input::Key(KeyCode::Escape));
    let currently_displayed = data.resources.get_resource::<ScionDeveloperConsoleResource>()
        .expect("Missing mandatory resource ScionDeveloperConsoleResource").currently_displayed;

    if (!currently_displayed && open && !close) {
        info!("pushing developer console");
        let current_window_width = data.resources.window().width();
        let current_window_height = data.resources.window().height();
        let parent = data.push((ScionDeveloperConsole,));
        let c = Color::new(50,50,50, 0.8);
        let material = Material::Diffuse(c);

        data.push((UiImage::new(current_window_width as f32, current_window_height as f32), Transform::from_xyz(0.,0.,3), material, Parent(parent)));

        let c2 = Color::new(10,10,10, 0.9);
        let material2 = Material::Diffuse(c2);


        data.push((UiImage::new(current_window_width as f32, 60.), Transform::from_xyz(0.,current_window_height as f32 -60.,2), material2, Parent(parent)));

        let font = Font::TrueType {
            font_path:  app_base_path().join("examples/tetris/assets/").join("Arial.ttf").get(),
        };
        let font_asset = data.assets_mut().register_font(font);

        let mut input = UiInput::new(current_window_width as usize, 60, font_asset.clone())
            .with_font_size(14)
            .with_tab_index(1)
            .with_font_color(Color::new_rgb(255, 255, 255));
        input.set_text("Coucou".to_string());

        data.push((
            input,
            Transform::from_xyz(15.,current_window_height as f32 -35.,0),
            Parent(parent)
        ));



        data.resources.get_resource_mut::<ScionDeveloperConsoleResource>().expect("Missing mandatory resource ScionDeveloperConsoleResource").currently_displayed = true;
        data.resources.get_resource_mut::<ScionDeveloperConsoleResource>().expect("Missing mandatory resource ScionDeveloperConsoleResource").current_entity = Some(parent);

    }else if (currently_displayed && !open && close) {
        let e = data.resources.get_resource_mut::<ScionDeveloperConsoleResource>().expect("Missing mandatory resource ScionDeveloperConsoleResource").current_entity.unwrap();
        data.resources.get_resource_mut::<ScionDeveloperConsoleResource>().expect("Missing mandatory resource ScionDeveloperConsoleResource").currently_displayed = false;
        data.resources.get_resource_mut::<ScionDeveloperConsoleResource>().expect("Missing mandatory resource ScionDeveloperConsoleResource").current_entity = None;
        let _r = data.remove(e);
    }
}