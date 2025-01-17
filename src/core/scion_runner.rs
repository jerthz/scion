use hecs::Entity;
use std::sync::mpsc::Receiver;
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Instant;
use winit::dpi::{PhysicalSize, Size};
use winit::window::Window;

use crate::core::resources::time::Time;
use crate::core::scene::{SceneAction, SceneMachine};
use crate::core::scheduler::Scheduler;
use crate::core::world::GameData;
use crate::graphics::rendering::scion2d::pre_renderer::Scion2DPreRenderer;
use crate::graphics::rendering::scion2d::rendering_thread::ScionRenderingThread;
use crate::graphics::rendering::scion2d::window_rendering_manager::ScionWindowRenderingManager;
use crate::graphics::rendering::{RendererCallbackEvent, RendererEvent, RenderingInfos, RenderingUpdate};
use crate::graphics::windowing::window_event_handler::handle_window_event;
use crate::graphics::windowing::WindowingEvent;
use crate::utils::frame_limiter::{FrameLimiter, FrameLimiterConfig};

pub struct ScionRunner {
    pub(crate) game_data: GameData,
    pub(crate) scheduler: Scheduler,
    pub(crate) layer_machine: SceneMachine,
    pub(crate) window_rendering_manager: Option<ScionWindowRenderingManager>,
    pub(crate) window: Option<Arc<Window>>,
    pub(crate) main_thread_receiver: Option<Receiver<WindowingEvent>>,
    pub(crate) render_callback_receiver: Option<Receiver<RendererCallbackEvent>>,
    pub(crate) scion_pre_renderer: Scion2DPreRenderer,
}

impl ScionRunner {
    pub(crate) fn launch_game_loop(mut self) {
        self.setup();
        let mut frame_limiter = FrameLimiter::new(FrameLimiterConfig::default());
        let (render_sender, render_receiver) = mpsc::channel::<(Vec<RendererEvent>, Vec<RenderingUpdate>, Vec<RenderingInfos>, Vec<Entity>)>();
        let window_rendering_manager = self.window_rendering_manager.take();

        thread::spawn(move || { ScionRenderingThread::new(window_rendering_manager, render_receiver).run() });

        let mut start_tick = Instant::now();
        let mut fixed_tick = Instant::now();
        let mut render_tick = Instant::now();

        loop {
            self.compute_color_picked_entity();

            let should_tick = frame_limiter.is_min_tick();
            if should_tick {
                start_tick = Instant::now();
                let frame_duration = self
                    .game_data
                    .get_resource_mut::<Time>()
                    .expect("Time is an internal resource and can't be missing")
                    .frame();
                self.game_data.timers().add_delta_duration(frame_duration);
                let _r = render_sender.send((handle_window_event(&mut self), vec![], vec![],vec![]));
                self.layer_machine.apply_scene_action(SceneAction::Update, &mut self.game_data);
                self.scheduler.execute(&mut self.game_data);
                self.layer_machine.apply_scene_action(SceneAction::LateUpdate, &mut self.game_data);
                self.update_cursor();
            }

            if frame_limiter.is_fixed_update() {
                fixed_tick = Instant::now();
                self.layer_machine.apply_scene_action(SceneAction::FixedUpdate, &mut self.game_data);
                frame_limiter.fixed_tick();
            }

            if frame_limiter.render_unlocked() {
                render_tick = Instant::now();

                let updates = self.scion_pre_renderer.prepare_update(&mut self.game_data);
                let rendering_infos = Scion2DPreRenderer::prepare_rendering(&mut self.game_data);
                let _r = render_sender.send((vec![], updates, rendering_infos,vec![]));

                frame_limiter.render();
            }

            if should_tick {
                self.game_data.inputs().reset_inputs();
                self.game_data.events().cleanup();
                self.layer_machine
                    .apply_scene_action(SceneAction::EndFrame, &mut self.game_data);
                frame_limiter.tick(&start_tick);
                if let Some(e) = self.game_data.take_despawned(){
                    render_sender.send((vec![], vec![], vec![], e)).unwrap();
                }
            }
            if let Some(status) = self.game_data.resources.game_state_mut().take_picking_update(){
                render_sender.send((vec![RendererEvent::CursorPickingStatusUpdate(status)],vec![], vec![], vec![])).unwrap();
            }

            thread::sleep(frame_limiter.min_tick_duration.clone());
        }
    }

    fn compute_color_picked_entity(&mut self) {
        if let Some(rcv) = self.render_callback_receiver.as_mut() {
            if let Some(picked) = get_last_event(rcv) {
                match picked {
                    RendererCallbackEvent::CursorColorPicking(c) => {
                        if let Some(color) = c {
                            self.game_data.resources.game_state_mut().set_color_picked_entity(self.scion_pre_renderer.color_picking_storage.get_entity_from_color(&color));
                        }else{
                            self.game_data.resources.game_state_mut().set_color_picked_entity(None);
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn setup(&mut self) {
        self.game_data.insert_resource(crate::core::resources::window::Window::new(
            (self.window.as_ref().unwrap().inner_size().width, self.window.as_ref().unwrap().inner_size().height),
            self.window.as_ref().unwrap().scale_factor(),
        ));
        self.layer_machine.apply_scene_action(SceneAction::Start, &mut self.game_data);
    }
    fn update_cursor(&mut self) {
        let mut window = self.game_data.window();
        if let Some(icon) = window.new_cursor() {
            let w = self.window.as_mut().expect("A window is mandatory to run this game !");
            w.set_cursor_icon(*icon);
        }
        if let Some(dimensions) = window.new_dimensions() {
            let w = self.window.as_mut().expect("A window is mandatory to run this game !");
            let _r = w.request_inner_size(Size::Physical(PhysicalSize::new(dimensions.0 * window.dpi() as u32,
                                                                           dimensions.1 * window.dpi() as u32)));
        }
        if window.new_cursor().is_some(){
            window.reset_future_settings();
        }
    }
}

fn get_last_event(receiver: &Receiver<RendererCallbackEvent>) -> Option<RendererCallbackEvent> {
    let mut last_event = None;
    while let Ok(event) = receiver.try_recv() {
        last_event = Some(event);
    }
    last_event
}
