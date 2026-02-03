use profiling_macros::profile;
use crate::core::components::maths::hierarchy::{Children, Parent};
use crate::core::components::maths::transform::Transform;
use crate::core::resources::inputs::types::{Input, KeyCode};
use crate::core::world::{GameData, World};
use crate::graphics::components::ui::ui_input::UiInput;
use crate::graphics::components::ui::ui_text::UiText;

/// This system is responsible of handling components needed to represent a ui_input
/// It will detect and create needed components
#[profile("system::set_childs_on_inputs")]
pub(crate) fn set_childs_on_inputs(data: &mut GameData) {
    let mut to_add_entities = Vec::new();
    for (e, (ui_input, transform)) in data.query_mut::<(&UiInput, &Transform)>().without::<&Children>() {
        let mut ui_text = UiText::new(ui_input.text().to_string(), ui_input.font_ref());
        ui_text = ui_text.with_font_size(ui_input.font_size());
        if let Some(color) = ui_input.font_color() {
            ui_text = ui_text.with_font_color(color);
        }
        to_add_entities.push((e, ui_text, Transform::from_xyz(0., 0., transform.local_translation.z), Parent::new(e)));
    }
    to_add_entities.drain(0..).for_each(|(_, ui_text, transform, parent)| {
        data.push((ui_text, transform, parent));
    });
}

/// This system is responsible of handling inputs from keyboard on a focused input
#[profile("system::register_keyboard_inputs_on_ui_input")]
pub(crate) fn register_keyboard_inputs_on_ui_input(data: &mut GameData) {
    let (world, resources) = data.split();
    let current_focused = resources.focus_manager().current_focus_entity();
    if let Some(focused_entity) = current_focused {
        if let Ok(input) = world.entry_mut::<&mut UiInput>(focused_entity) {
            if !input.text().is_empty() && resources.inputs().input_pressed_event(&Input::Key(KeyCode::BackSpace)){
                input.set_text(input.text()[..input.text().len()-1].to_string());
                return;
            }
            let mut chars = Vec::new();
            resources.inputs()
                .all_pressed_events()
                .iter()
                .for_each(|i| {
                    if let Input::Key(k) = i {
                        let new_char = k.to_char();
                        if let Some(c) = new_char {
                            chars.push(c);
                        }
                    }
                });
            let upper = resources.inputs().key_pressed(&KeyCode::LShift) || resources.inputs().key_pressed(&KeyCode::RShift);
            chars.drain(0..).for_each(|c| {
                let final_char = if upper { c.to_uppercase().to_string() } else { c.to_string() };
                input.set_text(format!("{}{}",input.text(), final_char));
                input.dirty = true;
            })
        }
    }
}

/// This system is responsible of handling synchronization between ui inputs and ui text
#[profile("system::synchronize_input_and_text")]
pub(crate) fn synchronize_input_and_text(data: &mut GameData) {
    let mut dirty_input = Vec::new();
    for (_, (input, children)) in data.query_mut::<(&mut UiInput, &Children)>() {
        if input.dirty {
            dirty_input.push((*children.0.first().unwrap(), input.text().to_string()));
            input.dirty = false;
        }
    }

    dirty_input.drain(0..).for_each(|v| {
        let text = data.entry_mut::<&mut UiText>(v.0).unwrap();
            text.set_text(v.1);
            text.dirty = true;
    });
}

#[cfg(test)]
mod tests {
    use crate::core::components::maths::hierarchy::{Children, Parent};
    use crate::core::components::maths::transform::Transform;
    use crate::core::resources::asset_manager::AssetManager;
    use crate::core::systems::ui_input_systems::set_childs_on_inputs;
    use crate::core::world::{GameData, World};
    use crate::graphics::components::ui::font::Font;
    use crate::graphics::components::ui::ui_input::UiInput;
    use crate::graphics::components::ui::ui_text::UiText;

    #[test]
    fn set_childs_on_inputs_tests() {
        let mut data = GameData::default();
        let mut manager = AssetManager::default();
        let asset_ref = manager.register_font(Font::TrueType { font_path: "".to_string() });
        let e = data.push((Transform::default(), UiInput::new(10, 10, asset_ref)));

        set_childs_on_inputs(&mut data);

        data.entry::<&Children>(e).unwrap();
        let text_entity = data.query::<&UiText>().iter().next().unwrap().0;

        data.entry::<&Parent>(text_entity).unwrap();
    }
}