use log::debug;

use crate::core::components::maths::{
    camera::{Camera, DefaultCamera},
    transform::Transform,
};
use crate::core::world::{GameData, World};

/// System responsible of adding a Camera on each entity with a DefaultCamera component
pub(crate) fn default_camera_system(data: &mut GameData) {
    let default_entity =
        data.query::<&DefaultCamera>().without::<&Camera>().iter().map(|(e, _d)| e).next();

    let (subworld, resources) = data.split();

    if let Some(e) = default_entity {
        debug!("Adding default camera to the entity {:?}", e);
        let window = resources.window();
        let mut camera = Camera::new(
            window.width() as f32,
            window.height() as f32,
        );
        camera.dpi = window.dpi();
        let _r = subworld.add_components(e, (camera,Transform::default()));
        let _r = subworld.remove_component::<&DefaultCamera>(e);
    }
}
