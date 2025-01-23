use crate::core::components::maths::camera::Camera;
use crate::core::components::maths::transform::Transform;
use crate::core::package::Package;
use crate::core::resources::inputs::types::{Input, KeyCode};
use crate::core::world::{GameData, World};
use crate::ScionBuilder;

pub struct DummyCameraConfig {
    pub(crate) vertical_velocity: f32,
    pub(crate) horizontal_velocity: f32
}

impl DummyCameraConfig{
    pub fn set_vertical_velocity(&mut self, new_value: f32){
        self.vertical_velocity = new_value;
    }

    pub fn set_horizontal_velocity(&mut self, new_value: f32){
        self.horizontal_velocity = new_value;
    }

    pub fn get_velocities(&self) -> (f32, f32){
        (self.horizontal_velocity, self.vertical_velocity)
    }
}

impl Default for DummyCameraConfig {
    fn default() -> Self {
        Self {
            vertical_velocity: 5.0,
            horizontal_velocity: 5.0,
        }
    }
}


/// This package allows to add a premade classic camera controller based on arrows from the keyboard.
/// It can be configured by changing the DummyCameraConfig in the resources, in any scene or system like so :
///
/// `data.get_resource_mut::<DummyCameraConfig>().expect("Missing dummy camera config ?").set_horizontal_velocity(15.);`
pub struct DummyCamera;

impl Package for DummyCamera {
    fn prepare(&self, data: &mut GameData) {
        data.resources.insert_resource(DummyCameraConfig{ vertical_velocity: 5.0, horizontal_velocity: 5.0 });
    }

    fn load(&self, builder: ScionBuilder) -> ScionBuilder {
        builder.with_system(dummy_camera_controller_system)
    }
}



/// A premade camera controller allowing the player to move the camera using keyboard arrows
pub fn dummy_camera_controller_system(data: &mut GameData){
    let left = data.inputs().input_pressed(&Input::Key(KeyCode::Left));
    let right = data.inputs().input_pressed(&Input::Key(KeyCode::Right));
    let up = data.inputs().input_pressed(&Input::Key(KeyCode::Up));
    let down = data.inputs().input_pressed(&Input::Key(KeyCode::Down));

    let (vh, vv) = data.resources.get_resource::<DummyCameraConfig>()
        .expect("Missing mandatory DummyCameraConfig").get_velocities();

    let horizontal_input = 0. + if left { -1. * vh } else { 0. } + if right { vh } else { 0. };
    let vertical_input = 0. + if up { -1. * vv } else { 0. } + if down { vv } else { 0. };

    if horizontal_input != 0. || vertical_input != 0. {
        for (_, (t, _)) in data.query_mut::<(&mut Transform, &Camera)>(){
            t.append_x(horizontal_input);
            t.append_y(vertical_input);
        }
    }
}