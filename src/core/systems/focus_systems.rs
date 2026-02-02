use hecs::Entity;
use log::debug;

use crate::core::resources::inputs::types::{Input, KeyCode};
use crate::core::world::{GameData, ScionWorld, World};
use crate::graphics::components::ui::UiFocusable;
use crate::graphics::components::{Hide, HidePropagated};

#[derive(PartialEq, Eq)]
enum FocusAction {
    None,
    Next,
    Previous,
    Reset,
}

pub(crate) fn focus_switcher_system(data: &mut GameData) {
    let action = compute_action(data);
    let (world, resource) = data.split();
    match action {
        FocusAction::Next | FocusAction::Previous => {
            let current = resource.focus_manager().current_focus_index();
            let new_focus =
                if action == FocusAction::Next { retrieve_next_focus(current, world) } else { retrieve_previous_focus(current, world) };
            if let Some((e, rank)) = new_focus {

                // No need to change anything if we keep the current focus
                if current.is_some() && current.unwrap() == rank {
                    return;
                }

                let current_focused = resource.focus_manager().current_focus_entity();
                if current_focused.is_some() {
                    if let Ok(input) = world.entry_mut::<&mut UiFocusable>(current_focused.unwrap()) {
                        input.focused = false;
                    }
                }
                debug!("Focused tabIndex changed, focus is now on tabIndex {}", rank);
                let new_input = world.entry_mut::<&mut UiFocusable>(e).expect("Previously checked entity doesn't exist");
                new_input.focused = true;
                resource.focus_manager().change_focus(e, rank);
            }
        }
        FocusAction::Reset => {
            resource.focus_manager().reset_focus();
        }
        FocusAction::None => {}
    }
}

fn compute_action(data: &mut GameData) -> FocusAction {
    let previous_focus = vec![Input::Key(KeyCode::LShift), Input::Key(KeyCode::Tab)];
    if (data.resources.inputs().input_pressed_event(&Input::Key(KeyCode::Tab))
        || data.resources.inputs().input_pressed_event(&Input::Key(KeyCode::LShift))) &&
        data.resources.inputs().shortcut_pressed_event(&previous_focus) {
        return FocusAction::Previous;
    }

    if data.resources.inputs().input_pressed_event(&Input::Key(KeyCode::Tab)) {
        return FocusAction::Next;
    }

    if data.resources.inputs().input_pressed_event(&Input::Key(KeyCode::Escape)) {
        return FocusAction::Reset;
    }
    FocusAction::None
}

fn retrieve_next_focus(current_focus: Option<usize>,
                       world: &mut ScionWorld) -> Option<(Entity, usize)> {
    let (mut min_entity, mut next_entity) = (None, None);
    let (mut min, mut next) = (usize::MAX, usize::MAX);

    world.query::<&UiFocusable>()
        .without::<&Hide>()
        .without::<&HidePropagated>()
        .iter()
        .for_each(|(e, uif)| {
            if min_entity.is_some() {
                if min > uif.rank {
                    min = uif.rank;
                    min_entity = Some(e);
                }
                if current_focus.is_some()
                    && uif.rank > current_focus.unwrap()
                    && (uif.rank < next || current_focus.unwrap() == next) {
                    next = uif.rank;
                    next_entity = Some(e);
                }
            } else {
                (min, next) = (uif.rank, uif.rank);
                (min_entity, next_entity) = (Some(e), Some(e));
            }
        });
    if min_entity.is_none() {
        None
    } else if current_focus.is_none() || next <= current_focus.unwrap() {
        Some((min_entity.unwrap(), min))
    } else {
        Some((next_entity.unwrap(), next))
    }
}

fn retrieve_previous_focus(current_focus: Option<usize>,
                           world: &mut ScionWorld) -> Option<(Entity, usize)> {
    let (mut max_entity, mut previous_entity) = (None, None);
    let (mut max, mut previous) = (usize::MIN, usize::MIN);

    world.query::<&UiFocusable>()
        .without::<&Hide>()
        .without::<&HidePropagated>()
        .iter()
        .for_each(|(e, uif)| {
            if max_entity.is_some() {
                if max < uif.rank {
                    max = uif.rank;
                    max_entity = Some(e);
                }
                if current_focus.is_some()
                    && uif.rank < current_focus.unwrap()
                    && (uif.rank > previous || current_focus.unwrap() == previous) {
                    previous = uif.rank;
                    previous_entity = Some(e);
                }
            } else {
                (max, previous) = (uif.rank, uif.rank);
                (max_entity, previous_entity) = (Some(e), Some(e));
            }
        });
    if previous_entity.is_none() {
        None
    } else if current_focus.is_none() || previous >= current_focus.unwrap() {
        Some((max_entity.unwrap(), max))
    } else {
        Some((previous_entity.unwrap(), previous))
    }
}