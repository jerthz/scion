use crate::core::components::maths::coordinates::Coordinates;
use crate::core::components::maths::transform::Transform;
use crate::core::resources::font_atlas::CharacterPosition;
use crate::core::world::{GameData, World};
use crate::graphics::components::material::Material;
use crate::graphics::components::shapes::line::Line;
use crate::graphics::components::shapes::polygon::Polygon;
use crate::graphics::components::shapes::rectangle::Rectangle;
use crate::graphics::components::tiles::sprite::Sprite;
use crate::graphics::components::tiles::tilemap::{Tile, Tilemap};
use crate::graphics::components::ui::ui_image::UiImage;
use crate::graphics::components::ui::ui_text::UiText;
use crate::graphics::components::{Square, Triangle};
use crate::graphics::rendering::scion2d::pre_renderer::Scion2DPreRenderer;
use crate::graphics::rendering::shaders::gl_representations::TexturedGlVertexWithLayer;
use crate::graphics::rendering::{Renderable2D, RenderableUi, RenderingUpdate};
use hecs::{Component, Entity};
use log::info;
use wgpu::BufferUsages;

pub(crate) fn call(renderer: &mut Scion2DPreRenderer, data: &mut GameData) -> Vec<RenderingUpdate> {
    let mut updates = vec![];
    updates.append(&mut prepare_buffer_update_for_component::<Triangle>(renderer, data));
    updates.append(&mut prepare_buffer_update_for_component::<Square>(renderer, data));
    updates.append(&mut prepare_buffer_update_for_component::<Rectangle>(renderer, data));
    updates.append(&mut prepare_buffer_update_for_component::<Sprite>(renderer, data));
    updates.append(&mut prepare_buffer_update_for_component::<Line>(renderer, data));
    updates.append(&mut prepare_buffer_update_for_component::<Polygon>(renderer, data));
    updates.append(&mut prepare_buffer_update_for_ui_component::<UiImage>(renderer, data));
    updates.append(&mut prepare_buffer_update_for_ui_text(renderer, data));
    updates.append(&mut prepare_buffer_update_for_tilemap(renderer, data));
    updates
}

fn prepare_buffer_update_for_component<T: Component + Renderable2D>(
    renderer: &mut Scion2DPreRenderer,
    data: &mut GameData) -> Vec<RenderingUpdate> {
    let mut updates = vec![];
    for (entity, (component, material, _)) in data.query_mut::<(&mut T, &Material, &Transform)>() {
        if renderer.missing_vertex_buffer(&entity) || component.dirty() {
            let descriptor = component.vertex_buffer_descriptor(Some(material));
            updates.push(RenderingUpdate::VertexBuffer {
                entity,
                contents: descriptor.contents.to_vec(),
                usage: descriptor.usage,
            });
            renderer.upsert_vertex_buffer(entity);
        }

        if renderer.missing_indexes_buffer(&entity) || component.dirty() {
            let descriptor = component.indexes_buffer_descriptor();
            updates.push(RenderingUpdate::IndexBuffer {
                entity,
                contents: descriptor.contents.to_vec(),
                usage: descriptor.usage,
            });
            renderer.upsert_indexes_buffer(entity);
        }

        component.set_dirty(false);
    }
    updates
}

fn prepare_buffer_update_for_ui_component<T: Component + Renderable2D + RenderableUi>(
    renderer: &mut Scion2DPreRenderer,
    data: &mut GameData) -> Vec<RenderingUpdate> {
    let mut updates = vec![];
    for (entity, (component, _, m)) in data.query::<(&mut T, &Transform, Option<&Material>)>().iter() {
        if renderer.missing_vertex_buffer(&entity) || component.dirty() {
            let descriptor = component.vertex_buffer_descriptor(m);
            updates.push(RenderingUpdate::VertexBuffer {
                entity,
                contents: descriptor.contents.to_vec(),
                usage: descriptor.usage,
            });
            renderer.upsert_vertex_buffer(entity);
        }
        if renderer.missing_indexes_buffer(&entity) || component.dirty() {
            let descriptor = component.indexes_buffer_descriptor();

            updates.push(RenderingUpdate::IndexBuffer {
                entity,
                contents: descriptor.contents.to_vec(),
                usage: descriptor.usage,
            });
            renderer.upsert_indexes_buffer(entity);
        }
    }
    updates
}

fn prepare_buffer_update_for_tilemap(renderer: &mut Scion2DPreRenderer, data: &mut GameData) -> Vec<RenderingUpdate> {
    let mut updates = vec![];
    {
        let mut to_modify: Vec<(Entity, [TexturedGlVertexWithLayer; 4])> = Vec::new();
        for (entity, (t, material, _)) in data.query::<(&mut Tilemap, &Material, &Transform)>().iter() {
            let tile_size = Material::tile_size(material).expect("");
            let mut position = 0;
            let mut vertexes = Vec::new();
            let mut indexes = Vec::new();
            let isometric = t.is_isometric();
            let max_x = t.width();
            let depth = t.depth();

            let any_tile_modified = renderer.missing_vertex_buffer(&entity) || any_dirty_sprite(data, entity);
            if any_tile_modified {
                for (e, (tile, sprite)) in data.query::<(&Tile, &Sprite)>().iter() {
                    if tile.tilemap == entity {
                        let color_picking = renderer.color_picking_storage.create_picking(e);
                        let current_vertex = sprite.compute_content(Some(material));
                        to_modify.push((e, current_vertex));
                        let mut vec = current_vertex.to_vec();
                        let mut offset_x = 0.;
                        let mut offset_y = 0.;
                        let mut offset_z = 0;

                        /*
                            x 20
                            y 10
                         */
                        if isometric {
                            offset_x = -1. * tile.position.x() as f32 * t.offset_x_multiplier_x() + tile.position.y() as f32 * t.offset_x_multiplier_y() - (tile.position.z() as f32 * t.offset_x_multiplier_z());
                            offset_y = -1. * (tile.position.y() as f32 * t.offset_y_multiplier_y()  + tile.position.x() as f32 * t.offset_y_multiplier_x()) - (tile.position.z() as f32 * t.offset_y_multiplier_z());
                            offset_z = (max_x - tile.position.z()) * (max_x + 1) + tile.position.x() * (max_x + 1) + (max_x - tile.position.y())
                        } else {
                            offset_z = depth * 100 - tile.position.z() * 10;
                        }

                        vec.iter_mut().for_each(|gl_vertex| {
                            gl_vertex.position[0] = gl_vertex.position[0] + tile_size as f32 * tile.position.x() as f32 + offset_x;
                            gl_vertex.position[1] = gl_vertex.position[1] + tile_size as f32 * tile.position.y() as f32 + offset_y;
                            gl_vertex.position[2] = gl_vertex.position[2] + tile.position.z() as f32 / 100.;
                            gl_vertex.depth = gl_vertex.depth + offset_z as f32 * 0.00001;
                            gl_vertex.enable_color_picking_override = 1;
                            gl_vertex.color_picking_override = color_picking.as_f32_array();
                        });
                        let sprite_indexes = Sprite::indices();
                        let mut sprite_indexes: Vec<u16> = sprite_indexes
                            .iter()
                            .map(|indice| (*indice as usize + (position * 4)) as u16)
                            .collect();
                        position += 1;
                        vertexes.append(&mut vec);
                        indexes.append(&mut sprite_indexes);
                    }
                }

                let bytes_vertexes: &[u8] = bytemuck::cast_slice(vertexes.as_slice());
                updates.push(RenderingUpdate::VertexBuffer {
                    entity,
                    contents: bytes_vertexes.to_vec(),
                    usage: BufferUsages::VERTEX,
                });
                renderer.upsert_vertex_buffer(entity);

                let bytes_indexes: &[u8] = bytemuck::cast_slice(indexes.as_slice());
                updates.push(RenderingUpdate::IndexBuffer {
                    entity,
                    contents: bytes_indexes.to_vec(),
                    usage: BufferUsages::INDEX,
                });
                renderer.upsert_indexes_buffer(entity);
            }
        }

        for (e, vertexes) in to_modify.drain(0..) {
            let sprite = data.entry_mut::<&mut Sprite>(e).expect("");
            sprite.set_dirty(false);
            sprite.set_content(vertexes);
        }
    }
    updates
}

fn any_dirty_sprite(data: &GameData, entity: Entity) -> bool {
    data
        .query::<(&Tile, &Sprite)>()
        .iter()
        .filter(|(_, (tile, sprite))| tile.tilemap == entity && sprite.dirty())
        .count()
        > 0
}

fn prepare_buffer_update_for_ui_text(renderer: &mut Scion2DPreRenderer, data: &mut GameData) -> Vec<RenderingUpdate> {
    let mut updates = vec![];
    let (world, resources) = data.split();
    for (entity, (mut ui_text, _, material)) in world.query_mut::<(&mut UiText, &Transform, &Material)>() {
        let path = if let Material::Texture(e) = material {
            e.to_string()
        } else {
            "".to_string()
        };

        let mut indexes_accumulator = Vec::new();
        let mut vertexes_accumulator = Vec::new();
        let mut current_x = 0.;
        let current_y = 0.;
        if path != "" && ui_text.dirty() {
            let mut font_atlas = resources.font_atlas();
            let atlas = font_atlas.get_texture_from_path(&path).expect("Missing mandatory font atlas");
            let min_y = atlas.min_y();
            let texture_width = atlas.width as f32;
            let texture_height = atlas.height as f32;
            let mut space_nb = 0;
            for (pos, character) in ui_text.text().chars().enumerate() {
                if character.is_whitespace() {
                    current_x += 5.;
                    space_nb += 1;
                    continue;
                }
                let char = atlas.character_positions.get(&character).unwrap();
                let uvs = compute_char_uvs(texture_width, texture_height, char);
                let mut current_vertexes = ui_text.char_vertex(char.width(), char.height(), uvs);

                let offset_y = compute_offset(char, min_y, character);
                current_vertexes.iter_mut().for_each(|gl_vertex| {
                    gl_vertex.position[0] = gl_vertex.position[0] + current_x;
                    gl_vertex.position[1] = gl_vertex.position[1] + current_y + offset_y;
                });
                let char_indexes = UiText::char_indices();
                let mut char_indexes: Vec<u16> = char_indexes
                    .iter()
                    .map(|indice| (*indice as usize + ((pos - space_nb) * 4)) as u16)
                    .collect();

                vertexes_accumulator.append(&mut current_vertexes.to_vec());
                indexes_accumulator.append(&mut char_indexes);
                current_x = current_x + char.width() + 1.0; // TODO letter_spacing
                //TODO: Compute lines when handled
            }
            if vertexes_accumulator.is_empty(){
                vertexes_accumulator.append(&mut ui_text.char_vertex(0.,0.,empty_char_uvs()).to_vec())
            }
            if indexes_accumulator.is_empty(){
                indexes_accumulator.append(&mut UiText::char_indices().to_vec());
            }
            let bytes_vertexes: &[u8] = bytemuck::cast_slice(vertexes_accumulator.as_slice());
            updates.push(RenderingUpdate::VertexBuffer {
                entity,
                contents: bytes_vertexes.to_vec(),
                usage: BufferUsages::VERTEX,
            });
            renderer.upsert_vertex_buffer(entity);

            let bytes_indexes: &[u8] = bytemuck::cast_slice(indexes_accumulator.as_slice());
            updates.push(RenderingUpdate::IndexBuffer {
                entity,
                contents: bytes_indexes.to_vec(),
                usage: BufferUsages::INDEX,
            });
            renderer.upsert_indexes_buffer(entity);
            ui_text.set_dirty(false);
        }
    }
    updates
}

fn compute_offset(character_position: &CharacterPosition, min_y: f32, char: char) -> f32 {
    let current_start_y = character_position.start_y;
    if current_start_y > min_y {
        current_start_y - min_y
    }else{
        0.
    }
}

fn compute_char_uvs(texture_width: f32, texture_height: f32, char: &CharacterPosition) -> [Coordinates; 4] {
    [
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
    ]
}

fn empty_char_uvs() -> [Coordinates; 4] {
    [
        Coordinates::new(
            0.,
            0.,
        ),
        Coordinates::new(
            0.,
            0.,
        ),
        Coordinates::new(
            0.,
            0.,
        ),
        Coordinates::new(
            0.,
            0.,
        ),
    ]
}
