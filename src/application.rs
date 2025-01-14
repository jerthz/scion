use std::path::Path;

use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Duration;


use log::{error, info};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};
use winit::application::ApplicationHandler;
use winit::error::EventLoopError;
use winit::event::{DeviceEvent, DeviceId};
use winit::event_loop::ActiveEventLoop;
use winit::window::{WindowAttributes, WindowId};
use crate::{
    config::scion_config::{ScionConfig, ScionConfigReader},
};
use crate::core::application_builder::ScionBuilder;


use crate::core::scene::{SceneMachine};
use crate::core::scheduler::Scheduler;
use crate::core::scion_runner::ScionRunner;


use crate::core::world::GameData;
use crate::graphics::rendering::RendererCallbackEvent;
use crate::graphics::rendering::scion2d::window_rendering_manager::ScionWindowRenderingManager;
use crate::graphics::windowing::WindowingEvent;

/// `Scion` is the entry point of any application made with Scion's lib.
pub struct Scion {
    #[allow(dead_code)]
    pub(crate) config: ScionConfig,
    pub(crate) game_data: Option<GameData>,
    pub(crate) scheduler: Option<Scheduler>,
    pub(crate) layer_machine: Option<SceneMachine>,
    pub(crate) window_event_sender: Option<mpsc::Sender<WindowingEvent>>,
}

impl Scion {
    /// Creates a new `Scion` application.
    /// The application will check for a scion.json file at the root to find its configurations.
    /// If this file does not exist, it will create one with default values
    pub fn app() -> ScionBuilder {
        let app_config = ScionConfigReader::read_or_create_default_scion_json().expect(
            "Fatal error when trying to retrieve and deserialize `scion.json` configuration file.",
        );
        Scion::app_with_config(app_config)
    }

    /// Creates a new `Scion` application.
    /// The application will try to read a json file using the provided path.
    pub fn app_with_config_path(config_path: &Path) -> ScionBuilder {
        let app_config = ScionConfigReader::read_scion_json(config_path).expect(
            "Fatal error when trying to retrieve and deserialize `scion.json` configuration file.",
        );
        Scion::app_with_config(app_config)
    }

    /// Creates a new `Scion` application.
    /// The application will use the provided configuration.
    pub fn app_with_config(app_config: ScionConfig) -> ScionBuilder {
        crate::utils::logger::Logger::init_logging(app_config.logger_config.clone());
        info!("Starting a Scion app, with the following configuration \n {:?}", app_config);
        ScionBuilder::new(app_config)
    }


    // There was no technical need to have the run function inside the Scion struct, but I made it here because I wanted the
    // main window loop & game loop to be in the main application file.
    pub(crate) fn run(mut self) {
        if self.config.window_config.is_none() {
            // Running window less mode, so launching the runner in the main thread
            info!("Launching game in text mode");
            ScionRunner {
                game_data: self.game_data.expect("Fatal error TODO"),
                scheduler: self.scheduler.expect("Fatal error TODO"),
                layer_machine: self.layer_machine.expect("Fatal error TODO"),
                window_rendering_manager: None,
                window: None,
                main_thread_receiver: None,
                render_callback_receiver: None,
                scion_pre_renderer: Default::default(),
            }.launch_game_loop();
        } else {
            // Game is running in a window, it must be created & handled in the main thread, so
            // the game loop is going to another thread.
            let event_loop = EventLoop::<ScionEvent>::with_user_event().build().expect("Event loop could not be created");
            event_loop.set_control_flow(ControlFlow::Wait);
            match event_loop.run_app(&mut self){
                Ok(_) => {
                    info!("Gracefully closed the game");
                }
                Err(e) => {
                    error!("CLOSING - Fatal error during the game {:?}", e);
                }
            }
        }
    }
}

struct ScionEvent;

impl ApplicationHandler<ScionEvent> for Scion {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window_event_sender.is_some() {
            return;
        }

        let window_builder: WindowAttributes = self
            .config
            .window_config
            .clone()
            .expect("The window configuration has not been found")
            .into(&self.config);

        let window = Arc::new(
            event_loop
                .create_window(window_builder)
                .expect("An error occurred while building the main game window"),
        );
        let (render_callback_sender, render_callback_receiver) = mpsc::channel::<(RendererCallbackEvent)>();
        let window_rendering_manager = futures::executor::block_on(
            ScionWindowRenderingManager::new(
                window.clone(),
                self.config
                    .window_config
                    .as_ref()
                    .unwrap()
                    .default_background_color
                    .clone(),
                render_callback_sender
            ),
        );

        let (event_sender, receiver) = mpsc::channel::<WindowingEvent>();

        let game_data = self
            .game_data
            .take()
            .expect("Fatal error during event loop creation: game_data missing");
        let scheduler = self
            .scheduler
            .take()
            .expect("Fatal error during event loop creation: scheduler missing");
        let layer_machine = self
            .layer_machine
            .take()
            .expect("Fatal error during event loop creation: layer_machine missing");

        thread::spawn(move || {
            ScionRunner {
                game_data,
                scheduler,
                layer_machine,
                window_rendering_manager: Some(window_rendering_manager),
                window: Some(window.clone()),
                main_thread_receiver: Some(receiver),
                render_callback_receiver: Some(render_callback_receiver),
                scion_pre_renderer: Default::default(),
            }
                .launch_game_loop();
        });

        self.window_event_sender = Some(event_sender);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, user_event: ScionEvent) {
        // Handle user event.
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                let _r = self.window_event_sender.as_mut().expect("missing event_sender").send(WindowingEvent { window_event: Some(WindowEvent::RedrawRequested), redraw: true });
            }
            e => {
                let _r = self.window_event_sender.as_mut().expect("missing event_sender").send(WindowingEvent { window_event: Some(e), redraw: false });
            }
        }
    }

    fn device_event(&mut self, event_loop: &ActiveEventLoop, device_id: DeviceId, event: DeviceEvent) {
        // Handle device event.
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
    }
}

