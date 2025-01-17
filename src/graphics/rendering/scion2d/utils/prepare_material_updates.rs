use std::path::Path;
use std::time::SystemTime;


use crate::core::world::{GameData, World};
use crate::graphics::components::color::Color;
use crate::graphics::components::material::{Material, Texture, TextureArray};
use crate::graphics::components::tiles::tileset::Tileset;
use crate::graphics::rendering::scion2d::pre_renderer::Scion2DPreRenderer;
use crate::graphics::rendering::{DiffuseBindGroupUpdate, RenderingUpdate};
use crate::utils::file::{read_file_modification_time, FileReaderError};

///
/// This function has the responsability to track material creation or updates
/// If hot reload is activated, it will check file timestamp to reinsert the material if needed
///
pub(crate) fn call(renderer: &mut Scion2DPreRenderer, data: &mut GameData) -> Vec<RenderingUpdate> {
    let mut updates = vec![];
    let hot_timer_cycle = should_try_to_hot_reload(data);

    for (_entity, material) in data.query::<&Material>().iter() {
        match material {
            Material::Diffuse(color) => {
                if let Some(update) = try_color_update(renderer, color) {
                    updates.push(update);
                }
            }
            Material::Texture(texture_path) => {
                if let Some(update) = try_texture_update(renderer, data, hot_timer_cycle, texture_path) {
                    updates.push(update);
                }
            }
            Material::Tileset(tileset) => {
                if let Some(update) = try_tileset_update(renderer, hot_timer_cycle, tileset){
                    updates.push(update);
                }
            }
        }
    }
    updates
}

fn try_color_update(renderer: &mut Scion2DPreRenderer, color: &Color) -> Option<RenderingUpdate> {
    let path = color.to_texture_path();
    if renderer.missing_texture(&path) {
        let update = RenderingUpdate::DiffuseBindGroup{ path: path.to_string(), data: DiffuseBindGroupUpdate::ColorBindGroup(Texture::from_color(color)) };
        renderer.upsert_texture_timestamps(&path, SystemTime::now());
        return Some(update);
    }
    None
}

fn try_texture_update(renderer: &mut Scion2DPreRenderer, data: &GameData, hot_timer_cycle: bool, texture_path: &str) -> Option<RenderingUpdate> {
    let new_timestamp = read_modification_timestamp(renderer, hot_timer_cycle, texture_path);
    if renderer.should_reload_texture(texture_path, &new_timestamp) {
        let path = Path::new(texture_path);
        let loaded_texture = match data.font_atlas().get_texture_from_path(texture_path) {
            Some(t) => t.take_texture(),
            None => Texture::from_png(path)
        };
        let update = RenderingUpdate::DiffuseBindGroup { path: texture_path.to_string(), data: DiffuseBindGroupUpdate::TextureBindGroup(loaded_texture) };
        let timestamp_to_use = if let Some(Ok(timestamp)) = new_timestamp {
            timestamp
        } else {
            SystemTime::now()
        };
        renderer.upsert_texture_timestamps(texture_path, timestamp_to_use);
        return Some(update);
    }
    None
}

fn try_tileset_update(renderer: &mut Scion2DPreRenderer, hot_timer_cycle: bool, tileset: &Tileset) -> Option<RenderingUpdate> {
    let new_timestamp = read_modification_timestamp(renderer, hot_timer_cycle, tileset.texture.as_str());
    if renderer.should_reload_texture(tileset.texture.as_str(), &new_timestamp) {
        let _path = Path::new(tileset.texture.as_str());
        let update = RenderingUpdate::DiffuseBindGroup{ path: tileset.texture.to_string(), data: DiffuseBindGroupUpdate::TilesetBindGroup(TextureArray::from_tileset(tileset))};
        let timestamp_to_use = if let Some(Ok(timestamp)) = new_timestamp {
            timestamp
        } else {
            SystemTime::now()
        };
        renderer.upsert_texture_timestamps(tileset.texture.as_str(), timestamp_to_use);
        return Some(update);
    }
    None
}


fn read_modification_timestamp(renderer: &mut Scion2DPreRenderer, hot_timer_cycle: bool, texture_path: &str) -> Option<Result<SystemTime, FileReaderError>> {
    if hot_timer_cycle || renderer.missing_texture(texture_path) {
        let path = Path::new(texture_path);
        Some(read_file_modification_time(path))
    } else {
        None
    }
}

fn should_try_to_hot_reload(data: &mut GameData) -> bool {
    if cfg!(feature = "hot-reload") {
        let mut timers = data.timers();
        let hot_reload_timer =
            timers.get_timer("hot-reload-timer").expect("Missing mandatory timer : hot_reload");
        hot_reload_timer.cycle() > 0
    } else {
        false
    }
}