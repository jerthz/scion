use crate::core::components::maths::camera::Camera;
use crate::core::components::maths::transform::Transform;
use crate::core::components::Dirty;
use crate::core::world::{GameData, World};
use crate::graphics::components::material::Material;
use crate::graphics::components::shapes::line::Line;
use crate::graphics::components::shapes::polygon::Polygon;
use crate::graphics::components::shapes::rectangle::Rectangle;
use crate::graphics::components::tiles::sprite::Sprite;
use crate::graphics::components::tiles::tilemap::{Tile, Tilemap};
use crate::graphics::components::ui::ui_image::UiImage;
use crate::graphics::components::ui::ui_text::UiText;
use crate::graphics::components::ui::UiComponent;
use crate::graphics::components::{Square, Triangle};
use crate::graphics::rendering::scion2d::pre_renderer::Scion2DPreRenderer;
use crate::graphics::rendering::shaders::gl_representations::{GlUniform, UniformData};
use crate::graphics::rendering::{Renderable2D, RenderingUpdate};
use hecs::Component;

pub(crate) fn call(renderer: &mut Scion2DPreRenderer, data: &mut GameData) -> (Vec<RenderingUpdate>, (Camera, Transform)) {
    let camera = retrieve_camera_transform(data);

    let dirty_camera = if let Some((old_camera, old_transform)) = renderer.camera.as_ref() {
        old_transform.global_translation != camera.1.global_translation
    } else {
        false
    };

    let mut updates = vec![];
    if dirty_camera{
        updates.append(&mut update_transforms_for_type_no_dirty_check::<Triangle>(renderer, data, &camera));
        updates.append(&mut update_transforms_for_type_no_dirty_check::<Square>(renderer, data, &camera));
        updates.append(&mut update_transforms_for_type_no_dirty_check::<Rectangle>(renderer, data, &camera));
        updates.append(&mut update_transforms_for_sprites_no_dirty_check(renderer, data, &camera));
        updates.append(&mut update_transforms_for_type_no_dirty_check::<Line>(renderer, data, &camera));
        updates.append(&mut update_transforms_for_type_no_dirty_check::<Polygon>(renderer, data, &camera));
        updates.append(&mut update_transforms_for_type_no_dirty_check::<UiImage>(renderer, data, &camera));
        updates.append(&mut update_transforms_for_type_no_dirty_check::<UiText>(renderer, data, &camera));
        updates.append(&mut update_transforms_for_type_no_dirty_check::<Tilemap>(renderer, data, &camera));
    } else{
        updates.append(&mut update_transforms_for_type::<Triangle>(renderer, data, &camera));
        updates.append(&mut update_transforms_for_type::<Square>(renderer, data, &camera));
        updates.append(&mut update_transforms_for_type::<Rectangle>(renderer, data, &camera));
        updates.append(&mut update_transforms_for_sprites(renderer, data, &camera));
        updates.append(&mut update_transforms_for_type::<Line>(renderer, data, &camera));
        updates.append(&mut update_transforms_for_type::<Polygon>(renderer, data, &camera));
        updates.append(&mut update_transforms_for_type::<UiImage>(renderer, data, &camera));
        updates.append(&mut update_transforms_for_type::<UiText>(renderer, data, &camera));
        updates.append(&mut update_transforms_for_type::<Tilemap>(renderer, data, &camera));
    }

    (updates, camera)
}

fn update_transforms_for_type<T: Component + Renderable2D>(
    _renderer: &mut Scion2DPreRenderer,
    data: &mut GameData,
    camera: &(Camera, Transform),
) -> Vec<RenderingUpdate> {
    let mut updates = vec![];
    for (entity, (transform, optional_ui_component, renderable, optional_material, _)) in
        data.query::<(&Transform, Option<&UiComponent>, &T, Option<&Material>, &Dirty)>().iter()
    {
        let uniform = GlUniform::from(UniformData {
            transform,
            camera,
            is_ui_component: optional_ui_component.is_some(),
            pivot_offset: renderable.get_pivot_offset(optional_material),
        });
        updates.push(RenderingUpdate::TransformUniform { entity, uniform });
    }
    updates
}

fn retrieve_camera_transform(data: &mut GameData) -> (Camera, Transform) {
    let camera1 = {
        let mut t = Transform::default();
        let mut c = Camera::new(1.0, 1.0);

        for (_, (cam, tra)) in data.query::<(&Camera, &Transform)>().iter() {
            c = cam.clone();
            t = *tra;
        }
        (c, t)
    };
    let camera = (camera1.0.clone(), camera1.1.clone());
    camera
}

fn update_transforms_for_sprites(
    _renderer: &mut Scion2DPreRenderer,
    data: &mut GameData,
    camera: &(Camera, Transform),
) -> Vec<RenderingUpdate> {
    let mut updates = vec![];
    for (entity, (transform, optional_ui_component, renderable, optional_material, &Dirty)) in data
        .query::<(&Transform, Option<&UiComponent>, &Sprite, Option<&Material>, &Dirty)>()
        .without::<&Tile>()
        .iter()
    {
        let uniform = GlUniform::from(UniformData {
            transform,
            camera,
            is_ui_component: optional_ui_component.is_some(),
            pivot_offset: renderable.get_pivot_offset(optional_material),
        });
        updates.push(RenderingUpdate::TransformUniform { entity, uniform });
    }
    updates
}


fn update_transforms_for_type_no_dirty_check<T: Component + Renderable2D>(
    _renderer: &mut Scion2DPreRenderer,
    data: &mut GameData,
    camera: &(Camera, Transform),
) -> Vec<RenderingUpdate> {
    let mut updates = vec![];
    for (entity, (transform, optional_ui_component, renderable, optional_material)) in
        data.query::<(&Transform, Option<&UiComponent>, &T, Option<&Material>)>().iter()
    {
        let uniform = GlUniform::from(UniformData {
            transform,
            camera,
            is_ui_component: optional_ui_component.is_some(),
            pivot_offset: renderable.get_pivot_offset(optional_material),
        });
        updates.push(RenderingUpdate::TransformUniform { entity, uniform });
    }
    updates
}

fn update_transforms_for_sprites_no_dirty_check(
    _renderer: &mut Scion2DPreRenderer,
    data: &mut GameData,
    camera: &(Camera, Transform),
) -> Vec<RenderingUpdate> {
    let mut updates = vec![];
    for (entity, (transform, optional_ui_component, renderable, optional_material)) in data
        .query::<(&Transform, Option<&UiComponent>, &Sprite, Option<&Material>)>()
        .without::<&Tile>()
        .iter()
    {
        let uniform = GlUniform::from(UniformData {
            transform,
            camera,
            is_ui_component: optional_ui_component.is_some(),
            pivot_offset: renderable.get_pivot_offset(optional_material),
        });
        updates.push(RenderingUpdate::TransformUniform { entity, uniform });
    }
    updates
}
