
use log::LevelFilter;

use scion::config::logger_config::LoggerConfig;
use scion::config::scion_config::{ScionConfig, ScionConfigBuilder};
use scion::config::window_config::WindowConfigBuilder;
use scion::graphics::components::color::Color;
use scion::Scion;
use scion::utils::premade::default_developper_console::DummyDeveloperConsole;
use scion::utils::premade::dummy_camera_controller::DummyCamera;
use crate::scene::DemoScene;

mod scene;

fn main() {
    Scion::app_with_config(create_config())
        .with_scene::<DemoScene>()
        .with_package(DummyCamera)
        .with_package(DummyDeveloperConsole)
        .run();
}


fn create_config() -> ScionConfig {
    ScionConfigBuilder::new()
        .with_logger_config(LoggerConfig{
            scion_level_filter: LevelFilter::Debug,
            level_filter: LevelFilter::Debug })
        .with_app_name("Showcase - Scion".to_string())
        .with_window_config(
            WindowConfigBuilder::new()
                .with_resizable(true)
                .with_dimensions((1920, 1080))
                .with_default_background_color(Some(Color::new_rgb(255,0,0)))
                .get())
        .get()
}