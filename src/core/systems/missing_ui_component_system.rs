use std::any;

use hecs::Component;
use log::trace;
use profiling_macros::profile;
use crate::core::world::{GameData, World};
use crate::graphics::components::ui::{Focusable, UiComponent, UiFocusable};

/// System responsible to add the UiComponent to any T missing its uiComponent
#[profile("system::missing_ui_component_system")]
pub(crate) fn missing_ui_component_system<T: Component>(data: &mut GameData) {
    let mut to_add = Vec::new();
    {
        for (e, _) in data.query::<&T>().without::<&UiComponent>().iter() {
            to_add.push(e);
        }
    }
    to_add.drain(0..).for_each(|e| {
        let _r = data.add_components(e, (UiComponent,));
    });
}

/// System responsible to add UiFocusable to eligible Focusable entities
#[profile("system::missing_focus_component_system")]
pub(crate) fn missing_focus_component_system<T: Component + Focusable>(data: &mut GameData) {
    let mut to_add = Vec::new();
    {
        for (e, component) in data.query::<&T>().without::<&UiFocusable>().iter() {
            to_add.push((e, component.tab_index()));
        }
    }
    to_add.drain(0..).for_each(|(e, tab_index)| {
        trace!("Adding UiFocusable component to entity of type {:?}", any::type_name::<T>());
        let _r = data.add_components(e, (UiFocusable{ rank: tab_index, focused: false },));
    });
}

#[cfg(test)]
mod tests {
    use crate::core::resources::asset_manager::AssetManager;
    use crate::core::world::World;
    use crate::graphics::components::ui::font::Font;
    use crate::graphics::components::ui::ui_input::UiInput;
    use crate::graphics::components::ui::{ui_image::UiImage, UiComponent};

    use super::*;

    #[test]
    fn missing_ui_comp_system_test() {
        let mut world = GameData::default();

        let e = world.push((UiImage::new(1., 1.),));

        assert!(world.entry::<&UiComponent>(e).expect("").get().is_none());

        missing_ui_component_system::<UiImage>(&mut world);

        assert!(world.entry::<&UiComponent>(e).expect("").get().is_some());
    }

    #[test]
    fn missing_ui_focus_system_test() {
        let mut world = GameData::default();
        let mut manager = AssetManager::default();
        let asset_ref = manager.register_font(Font::TrueType { font_path: "".to_string() });

        let e = world.push((UiInput::new(1,2,asset_ref),));

        assert!(world.entry::<&UiFocusable>(e).expect("").get().is_none());

        missing_focus_component_system::<UiInput>(&mut world);

        assert!(world.entry::<&UiFocusable>(e).expect("").get().is_some());
    }
}
