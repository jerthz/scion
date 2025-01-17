use std::collections::HashSet;

use crate::core::components::maths::coordinates::Coordinates;
use crate::core::components::maths::hierarchy::Parent;
use crate::core::components::maths::transform::Transform;
use atomic_refcell::AtomicRefMut;
use hecs::Entity;
use log::{debug, info};

use crate::core::resources::font_atlas::FontAtlas;
use crate::core::world::{GameData, World};
use crate::graphics::components::color::Color;
use crate::graphics::components::material::Material;
use crate::graphics::components::ui::{
    font::Font,
    ui_image::UiImage,
    ui_text::{UiText, UiTextImage},
    UiComponent,
};

pub(crate) fn sync_text_value_system(data: &mut GameData) {
    let (world, resources) = data.split();
    for (_e, ui_text) in world.query_mut::<&mut UiText>() {
        if let Some(function) = ui_text.sync_fn {
            ui_text.set_text(function(resources));
        }
    }
}


/// This system is responsible to resolve missing material on ui_text
pub(crate) fn ui_text_material_resolver(data: &mut GameData) {
    let (world, resources) = data.split();
    let mut to_add = Vec::new();
    let default_color = Color::new_rgb(255, 255, 255);
    for (e, ui_text) in world.query::<&UiText>().without::<&Material>().iter() {
        let font = resources.assets_mut().get_font_for_ref(ui_text.font_ref());
        to_add.push((e, match font {
            Font::Bitmap { texture_path, .. } => { Material::Texture(texture_path.to_string()) }
            Font::TrueType { font_path } => {
                Material::Texture(FontAtlas::true_type_path(&font_path, ui_text.font_size(), if ui_text.font_color().is_some() {
                    ui_text.font_color().as_ref().unwrap()
                } else {
                    &default_color
                }))
            }
        }));
    }
    to_add.drain(0..to_add.len()).for_each(|(e, material)| {
        world.add_components(e, (material,)).expect("Failed to add component");
    })
}

/// This system is responsible to check missing font atlas in the registry
/// In the end, it generates a bitmap for True type, and use existing bitmap for bitmap font
/// Each are inserted in the Font atlas, int later used in rendering.
pub(crate) fn ui_text_atlas_system(data: &mut GameData) {
    let (world, resources) = data.split();
    let mut font_atlas = resources.font_atlas();
    for (_, ui_text) in world.query::<&UiText>().iter() {
        let font = resources.assets_mut().get_font_for_ref(ui_text.font_ref());
        match font {
            Font::Bitmap { texture_path, chars, width, height, texture_columns, texture_lines } => {
                add_bitmap_to_atlas_if_missing(texture_path, chars, width, height, texture_columns, texture_lines, &mut font_atlas);
            }
            Font::TrueType { font_path } => {
                let color = if ui_text.font_color().is_some() {
                    ui_text.font_color().as_ref().unwrap().clone()
                } else {
                    Color::new_rgb(255, 255, 255)
                };
                add_true_type_to_atlas_if_missing(ui_text.font_size(), &color, &font_path, &mut font_atlas);
            }
        }
    }
}

fn add_true_type_to_atlas_if_missing(size: usize, color: &Color, font_path: &str, font_atlas: &mut AtomicRefMut<FontAtlas>) {
    if font_atlas.get_texture(font_path, size, color).is_none() {
        debug!("Adding true type font to atlas: [path: {}; size:{}; color:{:?}]", font_path, size, color);
        let res = crate::core::resources::font_atlas::convert_true_type(font_path.to_string(), size, color);
        if let Ok(texture) = res {
            font_atlas.add_true_type(font_path.to_string(), size, color, texture);
        }
    }
}

fn add_bitmap_to_atlas_if_missing(texture_path: String,
                                  chars: String,
                                  width: f32,
                                  height: f32,
                                  texture_columns: f32,
                                  texture_lines: f32,
                                  font_atlas: &mut AtomicRefMut<FontAtlas>) {
    if font_atlas.get_texture_from_path(&texture_path).is_none() {
        debug!("Adding bitmap font to atlas: [path: {}]", texture_path);
        let res = crate::core::resources::font_atlas::convert_bitmap(texture_path.to_string(), chars, width, height, texture_columns, texture_lines);
        if let Ok(texture) = res {
            font_atlas.add_bitmap(texture_path.to_string(), texture);
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::core::resources::asset_manager::AssetManager;
    use crate::core::world::World;
    use crate::graphics::components::ui::{
        font::Font,
        ui_text::{UiText, UiTextImage},
    };

    use super::*;

    fn get_test_ui_text(assets: &mut AssetManager) -> UiText {
        // First we add an UiText to the world
        let font = Font::Bitmap {
            texture_path: "test".to_string(),
            chars: "abcdefg".to_string(),
            texture_columns: 7.,
            texture_lines: 1.,
            width: 5.,
            height: 5.,
        };

        let asset = assets.register_font(font);

        UiText::new("abf".to_string(), asset)
    }

    #[test]
    fn ui_text_without_transform_should_not_generate_ui_image() {
        let mut world = GameData::default();
        let mut manager = AssetManager::default();
        let _entity = world.push((get_test_ui_text(&mut manager),));
        world.insert_resource(manager);

        ui_text_bitmap_update_system(&mut world);

        let cpt = world.query::<&UiTextImage>().iter().count();
        assert_eq!(0, cpt);
    }

    #[test]
    fn ui_text_with_transform_should_generate_ui_image() {
        let mut world = GameData::default();

        let mut manager = AssetManager::default();
        let _entity = world.push((get_test_ui_text(&mut manager), Transform::default()));
        world.insert_resource(manager);

        ui_text_bitmap_update_system(&mut world);

        let cpt = world.query::<&UiTextImage>().iter().count();
        assert_eq!(3, cpt);
    }

    struct Test {
        pub score: usize,
    }

    #[test]
    fn ui_text_synchronized() {
        let mut world = GameData::default();
        world.insert_resource(Test { score: 5 });

        let mut manager = AssetManager::default();
        let text_synced = get_test_ui_text(&mut manager)
            .sync_value(|g| g.get_resource::<Test>().unwrap().score.to_string());
        let _entity = world.push((text_synced, Transform::default()));
        world.insert_resource(manager);

        let txt = world.query::<&UiText>().iter().next().unwrap().1.text().to_string();
        assert_eq!("abf".to_string(), txt);

        sync_text_value_system(&mut world);

        let txt = world.query::<&UiText>().iter().next().unwrap().1.text().to_string();
        assert_eq!("5".to_string(), txt);
    }
}
