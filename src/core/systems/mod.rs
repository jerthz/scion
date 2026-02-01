use crate::core::package::Package;
use crate::core::resources::asset_manager::AssetManager;
use crate::core::resources::audio::Audio;
use crate::core::resources::events::topic::TopicConfiguration;
use crate::core::resources::events::Events;
use crate::core::resources::focus_manager::FocusManager;
use crate::core::resources::font_atlas::FontAtlas;
use crate::core::resources::global_storage::GlobalStorage;
use crate::core::resources::inputs::inputs_controller::InputsController;
use crate::core::resources::time::{Time, TimerType, Timers};
use crate::core::scene::SceneController;
use crate::core::state::GameState;
use crate::core::systems::animations_system::animation_executer_system;
use crate::core::systems::asset_ref_resolver_system::asset_ref_resolver_system;
use crate::core::systems::asset_ref_resolver_system::MaterialAssetResolverFn;
use crate::core::systems::collider_systems::{collider_cleaner_system, collider_pivot_propagation_system, compute_collisions_system, debug_colliders_system};
use crate::core::systems::default_camera_system::default_camera_system;
use crate::core::systems::focus_systems::focus_switcher_system;
use crate::core::systems::hide_propagation_system::{
    hide_propagated_deletion_system, hide_propagation_system,
};
use crate::core::systems::missing_ui_component_system::{missing_focus_component_system, missing_ui_component_system};
use crate::core::systems::parent_transform_system::{dirty_child_system, dirty_transform_offset_system, dirty_transform_system};
use crate::core::systems::ui_button_systems::{compute_hover, set_childs_on_buttons};
use crate::core::systems::ui_input_systems::{register_keyboard_inputs_on_ui_input, set_childs_on_inputs, synchronize_input_and_text};
use crate::core::systems::ui_text_system::{sync_text_value_system, ui_text_atlas_system, ui_text_material_resolver};
use crate::core::world::GameData;
use crate::graphics::components::material::Material;
use crate::graphics::components::shapes::line::Line;
use crate::graphics::components::shapes::polygon::Polygon;
use crate::graphics::components::shapes::rectangle::Rectangle;
use crate::graphics::components::tiles::sprite::Sprite;
use crate::graphics::components::ui::ui_button::UiButton;
use crate::graphics::components::ui::ui_image::UiImage;
use crate::graphics::components::ui::ui_input::UiInput;
use crate::graphics::components::ui::ui_text::{UiText};
use crate::graphics::components::{Square, Triangle};
use crate::ScionBuilder;

pub(crate) mod animations_system;
pub(crate) mod asset_ref_resolver_system;
pub(crate) mod collider_systems;
pub(crate) mod default_camera_system;
pub(crate) mod hide_propagation_system;
pub(crate) mod missing_ui_component_system;
pub(crate) mod parent_transform_system;
pub(crate) mod ui_text_system;
pub(crate) mod ui_input_systems;
pub(crate) mod focus_systems;
pub(crate) mod ui_button_systems;

pub(crate) struct InternalPackage;
impl Package for InternalPackage {
    fn prepare(&self, data: &mut GameData) {
        let mut events = Events::default();
        events
            .create_topic("Inputs", TopicConfiguration::default())
            .expect("Error while creating topic for inputs event");

        let mut timers = Timers::default();

        if cfg!(feature = "hot-reload") {
            let _res = timers.add_timer("hot-reload-timer", TimerType::Cyclic, 5.);
        }

        data.insert_resource(Time::default());
        data.insert_resource(FocusManager::default());
        data.insert_resource(events);
        data.insert_resource(timers);
        data.insert_resource(AssetManager::default());
        data.insert_resource(InputsController::default());
        data.insert_resource(GameState::default());
        data.insert_resource(SceneController::default());
        data.insert_resource(Audio::default());
        data.insert_resource(FontAtlas::default());
        data.insert_resource(GlobalStorage::default());
    }

    fn load(&self, builder: ScionBuilder) -> ScionBuilder {
        builder
            .with_system(dirty_transform_offset_system)
            .with_system(collider_cleaner_system)
            .with_system(default_camera_system)
            .with_system(sync_text_value_system)
            .with_system(set_childs_on_buttons)
            .with_system(hide_propagated_deletion_system)
            .with_system(hide_propagation_system)
            .with_system(collider_pivot_propagation_system::<Sprite>)
            .with_system(collider_pivot_propagation_system::<Rectangle>)
            .with_system(collider_pivot_propagation_system::<Square>)
            .with_system(collider_pivot_propagation_system::<Triangle>)
            .with_system(collider_pivot_propagation_system::<Polygon>)
            .with_system(collider_pivot_propagation_system::<Line>)
            .with_system(debug_colliders_system)
            .with_system(missing_ui_component_system::<UiImage>)
            .with_system(missing_ui_component_system::<UiText>)
            .with_system(missing_ui_component_system::<UiButton>)
            .with_system(missing_focus_component_system::<UiInput>)
            .with_system(asset_ref_resolver_system::<Material, MaterialAssetResolverFn>)
            .with_system(animation_executer_system)
            .with_system(dirty_child_system)
            .with_system(dirty_transform_system)
            .with_system(compute_collisions_system)
            .with_system(set_childs_on_inputs)
            .with_system(ui_text_material_resolver)
            .with_system(ui_text_atlas_system)
            .with_system(compute_hover)
            .with_system(focus_switcher_system)
            .with_system(register_keyboard_inputs_on_ui_input)
            .with_system(synchronize_input_and_text)
    }
}
