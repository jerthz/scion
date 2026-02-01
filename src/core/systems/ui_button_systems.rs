use hecs::Entity;
use winit::window::CursorIcon;

use crate::core::components::maths::hierarchy::{Children, Parent};
use crate::core::components::maths::transform::Transform;
use crate::core::resources::asset_manager::AssetRef;
use crate::core::resources::inputs::types::{Input, MouseButton};
use crate::core::world::{GameData, Resources, SubWorld, World};
use crate::graphics::components::color::Color;
use crate::graphics::components::material::Material;
use crate::graphics::components::ui::ui_button::UiButton;
use crate::graphics::components::ui::ui_image::UiImage;
use crate::graphics::components::ui::ui_text::UiText;
use crate::graphics::components::{Hide, HidePropagated};
use crate::graphics::rendering::Renderable2D;

/// This system is responsible of handling components needed to represent buttons
/// It will detect and create needed components
pub(crate) fn set_childs_on_buttons(data: &mut GameData) {
    let (world, resources) = data.split();

    let mut to_add_entities = Vec::new();
    let mut to_add_entities_2 = Vec::new();
    for (e, (ui_button, _)) in world.query_mut::<(&UiButton, &Transform)>().without::<&Children>() {
        let mut ui_text = UiText::new(ui_button.text().to_string(), ui_button.font_ref());
        ui_text = ui_text.with_font_size(ui_button.font_size());
        if let Some(color) = ui_button.font_color() {
            ui_text = ui_text.with_font_color(color);
        }
        ui_text.set_padding(ui_button.padding());
        ui_text.set_dirty(true);
        let mut material = Material::Diffuse(Color::new(0, 0, 0, 0.));
        if let Some(a) = ui_button.background() {
            let mat = resources.assets().get_material_for_ref(&a);
            material = mat;
        }

        to_add_entities.push((e, UiImage::new(ui_button.width() as f32, ui_button.height() as f32), material, Transform::from_xyz(0., 0., 1), Parent::new(e)));
        to_add_entities_2.push((e, ui_text, Transform::from_xyz(0., 0., 0), Parent::new(e)));
    }
    to_add_entities.drain(0..).for_each(|(_, ui_image, texture, transform, parent)| {
        data.push((ui_image, texture, transform, parent));
    });

    to_add_entities_2.drain(0..).for_each(|(_, ui_text, transform, parent)| {
        data.push((ui_text, transform, parent));
    });
}

pub(crate) fn compute_hover(data: &mut GameData) {
    let (world, resources) = data.split();
    let (mx, my) = resources.inputs().mouse_xy();
    let clicked = resources.inputs().input_pressed(&Input::Mouse(MouseButton::Left));
    let click_event = resources.inputs().input_pressed_event(&Input::Mouse(MouseButton::Left));
    let mut hover = false;
    let mut reset_hover = false;
    let mut hovered_buttons = Vec::new();
    let mut clicked_buttons = Vec::new();
    let mut not_hovered_buttons = Vec::new();
    for (_, (ui_button, transform, children))
    in world.query_mut::<(&mut UiButton, &Transform, &mut Children)>()
        .without::<&Hide>()
        .without::<&HidePropagated>() {
        if transform.global_translation.x as f64 <= mx
            && (transform.global_translation.x + ui_button.width() as f32) as f64 >= mx
            && transform.global_translation.y as f64 <= my
            && (transform.global_translation.y + ui_button.height() as f32) as f64 >= my {
            if !clicked && ui_button.hover().is_some() {
                ui_button.hovered = true;
                children.0.iter().for_each(|v|{
                    hovered_buttons.push((*v, ui_button.clone_hover_unchecked()));
                });

            } else if ui_button.clicked().is_some() {
                if let Some(function) = ui_button.on_click {
                    if click_event {
                        function(resources);
                    }
                }
                children.0.iter().for_each(|v|{
                    clicked_buttons.push((*v, ui_button.clone_clicked_unchecked()));
                });
            }
            hover = true;
        } else {
            if ui_button.background().is_some() {
                children.0.iter().for_each(|v|{
                    not_hovered_buttons.push((*v, ui_button.clone_background_unchecked()));
                });
            }
            if ui_button.hovered {
                ui_button.hovered = false;
                reset_hover = true;
            }
        }
    }

    change_button_material(world, resources, &mut clicked_buttons);
    change_button_material(world, resources, &mut not_hovered_buttons);
    change_button_material(world, resources, &mut hovered_buttons);


    if hover {
        resources.window().set_cursor(CursorIcon::Pointer);
    } else if reset_hover {
        resources.window().set_cursor(CursorIcon::Default);
    }
}

fn change_button_material(world: &mut SubWorld, resources: &mut Resources, clicked_buttons: &mut Vec<(Entity, AssetRef<Material>)>) {
    clicked_buttons.drain(0..).for_each(|(child, asset_ref)| {
        let button_asset_ref = world.entry_mut::<(Option<&AssetRef<Material>>, Option<&UiImage>)>(child);

        if button_asset_ref.is_err() || button_asset_ref.as_ref().expect("").1.is_none(){
            return;
        }
        if button_asset_ref.as_ref().expect("").0.is_none()
            || button_asset_ref.as_ref().expect("").0.as_ref().unwrap().0 != asset_ref.0 {
            let material = resources.assets().get_material_for_ref(&asset_ref);
            let _r = world.remove_component::<Material>(child);
            let _r = world.remove_component::<AssetRef<Material>>(child);
            let _r = world.add_components(child, (asset_ref, material));
        }
    });
}