use winit::event::WindowEvent;
use winit::keyboard::Key;
use crate::core::components::maths::camera::Camera;

use crate::core::resources::inputs::mouse::MouseEvent;
use crate::core::resources::inputs::types::{InputState, KeyboardEvent, KeyCode, MouseButton};
use crate::core::scion_runner::ScionRunner;
use crate::core::world::World;
use crate::graphics::rendering::RendererEvent;
use log::info;

pub fn handle_window_event(runner: &mut ScionRunner) -> Vec<RendererEvent> {
    let mut update = vec![];
    let mut force_redraw = false;
    if let Some(receiver) = runner.main_thread_receiver.as_ref() {
        while let Ok(event) = receiver.try_recv() {
            if event.redraw {
                force_redraw = true;
            };
            if let Some(window_event) = event.window_event {
                match window_event {
                    WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                        update.push(RendererEvent::Resize(runner.window.as_ref().expect("Missing window").inner_size(), scale_factor));
                        for (_, camera) in runner.game_data.query_mut::<&mut Camera>() {
                            camera.dpi = scale_factor;
                        }
                        force_redraw = true;
                    }
                    WindowEvent::Resized(physical_size) => {
                        info!("Window has been resized !");
                        runner.game_data
                            .window()
                            .set_dimensions(physical_size.width, physical_size.height);
                        runner.game_data
                            .window()
                            .set_dpi(runner.window.as_ref().expect("Missing window").scale_factor());
                        update.push(RendererEvent::Resize(physical_size, runner.window.as_ref().expect("Missing window").scale_factor()));
                        force_redraw = true;
                    }
                    WindowEvent::KeyboardInput { event, .. } => {
                        if let &Key::Dead(_) = &event.logical_key {} else {
                            let k_event = KeyboardEvent {
                                keycode: KeyCode::from(&event.logical_key),
                                state: InputState::from(event.state),
                            };
                            runner.game_data.inputs().add_keyboard_event(k_event);
                        }
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        let m_event = MouseEvent {
                            button: MouseButton::from(button),
                            state: InputState::from(state),
                        };
                        runner.game_data.inputs().add_click_event(m_event);
                    }
                    WindowEvent::CursorMoved { device_id: _, position, .. } => {
                        let dpi_factor = runner
                            .window
                            .as_mut()
                            .unwrap()
                            .current_monitor()
                            .expect("Missing the monitor")
                            .scale_factor();
                        runner.game_data.inputs().set_mouse_position(
                            position.x / dpi_factor,
                            position.y / dpi_factor,
                        );
                        update.push(RendererEvent::CursorPositionUpdate(Some(((position.x / dpi_factor) as u32, (position.y / dpi_factor) as u32))));
                    }
                    WindowEvent::CursorLeft{ .. } => {
                        update.push(RendererEvent::CursorPositionUpdate(None));
                    }
                    _ => {}
                }
            }
        }
    }
    if force_redraw {
        update.push(RendererEvent::ForceRedraw);
    }
    update
}