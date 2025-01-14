use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

use hecs::Entity;
use crate::core::resources::color_picking::ColorPickingStorage;
use crate::graphics::components::{Square, Triangle};
use crate::graphics::components::shapes::line::Line;
use crate::graphics::components::shapes::polygon::Polygon;
use crate::graphics::components::shapes::rectangle::Rectangle;
use crate::graphics::components::tiles::sprite::Sprite;
use crate::graphics::components::ui::ui_image::UiImage;
use crate::graphics::components::ui::ui_text::UiTextImage;
use crate::core::world::{GameData, World};
use crate::graphics::rendering::{RenderingInfos, RenderingUpdate};
use crate::graphics::rendering::scion2d::utils::pre_render_components::{pre_render_component, pre_render_tilemaps, pre_render_ui_component};
use crate::graphics::rendering::scion2d::utils::{prepare_component_buffer_updates};
use crate::graphics::rendering::scion2d::utils::prepare_material_updates;
use crate::graphics::rendering::scion2d::utils::prepare_transform_updates;
use crate::utils::file::FileReaderError;

#[derive(Default)]
pub(crate) struct Scion2DPreRenderer {
    textures_timestamps: HashMap<String, SystemTime>,
    transform_uniform: HashSet<Entity>,
    vertex_buffer: HashSet<Entity>,
    indexes_buffer: HashSet<Entity>,
    pub(crate) color_picking_storage: ColorPickingStorage
}

impl Scion2DPreRenderer {

    pub(crate) fn prepare_update(&mut self, data: &mut GameData) -> Vec<RenderingUpdate> {
        let mut updates = vec![];
        if data.has_camera() {
            updates.append(&mut prepare_material_updates::call(self, data));
            updates.append(&mut prepare_transform_updates::call(self, data));
            updates.append(&mut prepare_component_buffer_updates::call(self, data));
        }
        self.clean_buffers(data);
        updates
    }

    pub(crate) fn prepare_rendering(data: &mut GameData) -> Vec<RenderingInfos> {
        if data.has_camera() {
            let mut rendering_infos = Vec::new();
            rendering_infos.append(&mut pre_render_component::<Triangle>(data));
            rendering_infos.append(&mut pre_render_component::<Square>(data));
            rendering_infos.append(&mut pre_render_component::<Rectangle>(data));
            rendering_infos.append(&mut pre_render_component::<Sprite>(data));
            rendering_infos.append(&mut pre_render_component::<Line>(data));
            rendering_infos.append(&mut pre_render_component::<Polygon>(data));
            rendering_infos.append(&mut pre_render_ui_component::<UiImage>(data));
            rendering_infos.append(&mut pre_render_ui_component::<UiTextImage>(data));
            rendering_infos.append(&mut pre_render_tilemaps(data));
            rendering_infos.sort_by(|a, b| b.layer.cmp(&a.layer));
            return rendering_infos;
        }
        vec![]
    }
    pub(crate) fn missing_texture(&self, str: &str) -> bool {
        !self.textures_timestamps.contains_key(str)
    }

    pub(crate) fn upsert_texture_timestamps(&mut self, str: &str, timestamp: SystemTime) {
        self.textures_timestamps.insert(str.to_string(), timestamp);
    }

    pub(crate) fn should_reload_texture(&self, path: &str, new_timestamp: &Option<Result<SystemTime, FileReaderError>>) -> bool {
        self.missing_texture(path) || if let Some(Ok(timestamp)) = new_timestamp {
            self.textures_timestamps.get(path).unwrap().ne(timestamp)
        } else {
            false
        }
    }

    pub(crate) fn missing_vertex_buffer(&self, entity: &Entity)-> bool{
        !self.vertex_buffer.contains(entity)
    }

    pub(crate) fn upsert_vertex_buffer(&mut self, entity: Entity) {
        self.vertex_buffer.insert(entity);
    }


    pub(crate) fn missing_indexes_buffer(&self, entity: &Entity)-> bool{
        !self.indexes_buffer.contains(entity)
    }

    pub(crate) fn upsert_indexes_buffer(&mut self, entity: Entity) {
        self.indexes_buffer.insert(entity);
    }

    fn clean_buffers(&mut self, data: &mut GameData) {
        self.vertex_buffer.retain(|&k| data.contains(k));
        self.indexes_buffer.retain(|&k| data.contains(k));
        self.transform_uniform.retain(|&k| data.contains(k));
        // TODO transfer a clean buffer update to the rendering thread
    }
}