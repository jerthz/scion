use std::collections::HashSet;

use atomic_refcell::AtomicRefMut;
use hecs::Entity;
use log::debug;
use crate::core::components::maths::coordinates::Coordinates;
use crate::core::components::maths::hierarchy::Parent;
use crate::core::components::maths::transform::Transform;

use crate::graphics::components::{
    ui::{
        font::Font,
        ui_image::UiImage,
        ui_text::{UiText, UiTextImage},
        UiComponent,
    },
};
use crate::graphics::components::color::Color;
use crate::graphics::components::material::Material;
use crate::core::resources::font_atlas::FontAtlas;
use crate::core::world::{GameData, World};

pub(crate) fn sync_text_value_system(data: &mut GameData) {
    let (world, resources) = data.split();
    for(_e, ui_text) in world.query_mut::<&mut UiText>(){
        if let Some(function) = ui_text.sync_fn {
            ui_text.set_text(function(resources));
        }
    }
}

pub(crate) fn ui_text_bitmap_update_system(data: &mut GameData) {
    let mut parent_to_remove: HashSet<Entity> = HashSet::new();
    let mut to_add: Vec<(UiTextImage, Material, UiComponent, Transform, Parent)> = Vec::new();
    let (world, resources) = data.split();

    for (e, (ui_text, transform)) in world.query_mut::<(&mut UiText, &Transform)>() {
        if ui_text.dirty {
            parent_to_remove.insert(e);
            let font = resources.assets_mut().get_font_for_ref(ui_text.font_ref());

            match font {
                Font::Bitmap { texture_path, chars, width, height, texture_columns, texture_lines } => {
                    to_add.append(&mut update_bitmap(texture_path, chars, width, height, texture_columns, texture_lines, ui_text, transform, e));
                }
                Font::TrueType { font_path } => {
                    let mut font_atlas = resources.font_atlas();
                    let color = ui_text.font_color();
                    let color = if color.is_some(){
                        color.as_ref().unwrap().clone()
                    }else{
                        Color::new_rgb(255,255,255)
                    };
                    add_font_to_atlas_if_missing(ui_text.font_size(), &color, &font_path, &mut font_atlas);
                    let texture_path = format!("{:?}_{:?}_{:?}", &font_path, ui_text.font_size(), color.to_string());
                    let true_type_data = font_atlas.get_texture(&font_path, ui_text.font_size(), &color).expect("Missing data from atlas after insert");


                    let mut to_add_secondary: Vec<(UiTextImage, Material, UiComponent, Transform, Parent)> = Vec::new();
                    let texture_width = true_type_data.width as f32;
                    let texture_height = true_type_data.height  as f32;
                    let mut current_pos = 0.;
                    for character in ui_text.text().chars() {
                        if character.is_whitespace() {
                            current_pos += 5.;
                            continue;
                        }
                        let char = true_type_data.character_positions.get(&character).unwrap();
                        let uvs = [
                            Coordinates::new(
                                char.start_x / texture_width,
                                char.start_y / texture_height,
                            ),
                            Coordinates::new(
                                char.start_x / texture_width,
                                char.end_y / texture_height,
                            ),
                            Coordinates::new(
                                char.end_x / texture_width,
                                char.end_y / texture_height,
                            ),
                            Coordinates::new(
                                char.end_x / texture_width,
                                char.start_y / texture_height,
                            ),
                        ];
                        let mut char_transform = Transform::from_xy(current_pos, true_type_data.compute_vertical_offset(char.start_y));
                        current_pos = current_pos + (char.end_x-char.start_x) + 2.;
                        char_transform.set_z(transform.translation().z());
                        char_transform.append_x(ui_text.padding().left_or_zero());
                        char_transform.append_y(ui_text.padding().top_or_zero());
                        to_add_secondary.push((
                            UiTextImage(UiImage::new_with_uv_map_and_depth(
                                char.end_x-char.start_x,
                                char.end_y-char.start_y,
                                uvs,
                            )),
                            Material::Texture(texture_path.clone()),
                            UiComponent,
                            char_transform,
                            Parent(e),
                        ));
                    }
                    to_add.append(&mut to_add_secondary);
                }
            };
            ui_text.dirty = false;
        }
    }
    let entities_to_remove = data
        .query::<( &UiTextImage, &Parent) > ()
        .iter()
        .filter( | (_e, (_, p)) | parent_to_remove.contains( & p.0))
        .map( | (e, _) | e)
        .collect::<Vec<_ > > ();

    entities_to_remove.iter().for_each( | e| {
        let _r = data.remove( * e);
    });

    to_add.drain(0..).for_each( | c| {
        data.push(c);
    });
}

fn add_font_to_atlas_if_missing(size: usize, color: &Color, font_path: &str,  font_atlas: &mut AtomicRefMut<FontAtlas>) {
    if font_atlas.get_texture(font_path, size, color).is_none() {
        debug!("Adding font to atlas: [path: {}; size:{}; color:{:?}", font_path, size, color);
        let res = crate::core::resources::font_atlas::generate_bitmap(Font::TrueType { font_path: font_path.to_string() },
                                                                      size,
                                                                      color);
        if let Ok(texture) = res {
            font_atlas.add_texture(font_path.to_string(), size, color, texture);
        }
    }
}


fn update_bitmap(texture_path: String,
                 chars: String,
                 width: f32,
                 height: f32,
                 texture_columns: f32,
                 texture_lines: f32,
                 ui_text: &mut UiText,
                 transform: &Transform,
                 e: Entity
) -> Vec<(UiTextImage, Material, UiComponent, Transform, Parent)> {
    let mut to_add: Vec<(UiTextImage, Material, UiComponent, Transform, Parent)> = Vec::new();
    let texture_width = texture_columns * width;
    let texture_height = texture_lines * height;

    for (index, character) in ui_text.text().chars().enumerate() {
        let (line, column) =
            Font::find_line_and_column(&chars, texture_columns, character);

        let uvs = [
            Coordinates::new(
                (column * width) / texture_width,
                (line * height) / texture_height,
            ),
            Coordinates::new(
                (column * width) / texture_width,
                (line * height + height) / texture_height,
            ),
            Coordinates::new(
                (column * width + width) / texture_width,
                (line * height + height) / texture_height,
            ),
            Coordinates::new(
                (column * width + width) / texture_width,
                (line * height) / texture_height,
            ),
        ];

        let mut char_transform = Transform::from_xy(index as f32 * (width + 1.), 0.);
        char_transform.set_z(transform.translation().z());
        to_add.push((
            UiTextImage(UiImage::new_with_uv_map_and_depth(
                width,
                height,
                uvs,
            )),
            Material::Texture(texture_path.clone()),
            UiComponent,
            char_transform,
            Parent(e),
        ));
    }
    to_add
}


    #[cfg(test)]
    mod tests {
        use crate::graphics::components::{
            ui::{
                font::Font,
                ui_text::{UiText, UiTextImage},
            },
        };
        use crate::core::resources::asset_manager::AssetManager;
        use crate::core::world::World;

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
            let _entity = world.push((get_test_ui_text(&mut manager), ));
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
