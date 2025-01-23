use hecs::Component;

use crate::core::components::maths::transform::Transform;
use crate::core::world::{GameData, World};
use crate::graphics::components::material::Material;
use crate::graphics::components::tiles::sprite::Sprite;
use crate::graphics::components::tiles::tilemap::{Tile, Tilemap};
use crate::graphics::components::ui::ui_text::UiText;
use crate::graphics::components::{Hide, HidePropagated};
use crate::graphics::rendering::{Renderable2D, RenderableUi, RenderingInfos};

pub(crate) fn pre_render_component<T: Component + Renderable2D>(
    data: &mut GameData,
) -> Vec<RenderingInfos> {
    let type_name = std::any::type_name::<T>();
    let mut render_infos = Vec::new();
    for (entity, (component, material, transform)) in data
        .query::<(&mut T, &Material, &Transform)>()
        .without::<&Tile>()
        .without::<&Hide>()
        .without::<&HidePropagated>()
        .iter()
    {
        let path = match material {
            Material::Diffuse(color) => Some(color.to_texture_path()),
            Material::Texture(p) => Some(p.clone()),
            Material::Tileset(tileset) => Some(tileset.texture.clone()),
        };
        render_infos.push(RenderingInfos {
            layer: transform.global_translation().z(),
            range: component.range(),
            entity,
            texture_path: path,
            type_name: type_name.to_string(),
            render_priority: component.get_rendering_priority()
        });
    }
    render_infos
}

pub(crate) fn pre_render_tilemaps(data: &mut GameData) -> Vec<RenderingInfos> {
    let type_name = std::any::type_name::<Tilemap>();
    let mut render_infos = Vec::new();

    let tiles = data.query::<(&Tile, &Sprite)>().iter().map(|(e, _)| e).collect::<Vec<_>>();

    for (entity, (_, material, transform)) in data
        .query::<(&mut Tilemap, &Material, &Transform)>()
        .without::<&Hide>()
        .without::<&HidePropagated>()
        .iter()
    {
        let tiles_nb = tiles
            .iter()
            .filter(|t| data.entry::<&Tile>(**t).expect("").get().expect("").tilemap == entity)
            .count();

        let path = match material {
            Material::Tileset(tileset) => Some(tileset.texture.clone()),
            _ => None,
        };
        render_infos.push(RenderingInfos {
            layer: transform.translation().z(),
            range: 0..(tiles_nb * Sprite::indices().len()) as u32,
            entity,
            texture_path: path,
            type_name: type_name.to_string(),
            render_priority: 0
        });
    }
    render_infos
}

pub(crate) fn pre_render_ui_text(data: &mut GameData) -> Vec<RenderingInfos> {
    let type_name = std::any::type_name::<UiText>();
    let mut render_infos = Vec::new();

    for (entity, (component, material, transform)) in data.query::<(&UiText, &Material, &Transform)>().iter() {
        let path = match material {
            Material::Texture(path) => Some(path.clone()),
            _ => None,
        };
        render_infos.push(RenderingInfos {
            layer: transform.translation().z(),
            range: 0..(component.text().chars().filter(|c| !c.is_whitespace()).count() * UiText::char_indices().len()) as u32,
            entity,
            texture_path: path,
            type_name: type_name.to_string(),
            render_priority: 0
        });
    }

    render_infos
}

pub(crate) fn pre_render_ui_component<T: Component + Renderable2D + RenderableUi>(
    data: &mut GameData,
) -> Vec<RenderingInfos> {
    let type_name = std::any::type_name::<T>();
    let mut render_infos = Vec::new();
    for (entity, (component, transform, material)) in
    data.query::<(&mut T, &Transform, Option<&Material>)>()
        .without::<&Hide>()
        .without::<&HidePropagated>()
        .iter()
    {
        let path = if material.is_some() {
            match material.unwrap() {
                Material::Diffuse(color) => Some(color.to_texture_path()),
                Material::Texture(p) => Some(p.clone()),
                _ => None
            }
        } else {
            None
        };

        render_infos.push(RenderingInfos {
            layer: transform.translation().z(),
            range: component.range(),
            entity,
            texture_path: path,
            type_name: type_name.to_string(),
            render_priority: component.get_rendering_priority()
        });
    }
    render_infos
}