use std::sync::mpsc::{Receiver};
use log::{info};
use crate::graphics::rendering::{RendererEvent, RenderingInfos, RenderingUpdate};
use crate::graphics::rendering::scion2d::window_rendering_manager::ScionWindowRenderingManager;

pub(crate) struct ScionRenderingThread {
    pub(crate) window_rendering_manager: Option<ScionWindowRenderingManager>,
    pub(crate) render_receiver: Receiver<(Vec<RendererEvent>, Vec<RenderingUpdate>, Vec<RenderingInfos>)>,
}

impl ScionRenderingThread{
    pub(crate) fn run(mut self) {
        info!("Initializing rendering thread");
        let mut update_accumulator: Vec<RenderingUpdate> = Vec::new();
        loop {
            if let Ok((mut events, mut updates, rendering_infos)) = self.render_receiver.recv() {
                events.drain(0..events.len()).for_each(|event|{
                    match event {
                        RendererEvent::ForceRedraw => {
                            // TODO
                        }
                        RendererEvent::Resize(physical_size, scale_factor) => {
                            self.window_rendering_manager.as_mut().unwrap().resize(physical_size, scale_factor);
                        }
                        RendererEvent::CursorPositionUpdate(v) => {
                            self.window_rendering_manager.as_mut().unwrap().update_cursor(v);
                        }
                    }
                });

                if !updates.is_empty(){
                    update_accumulator.append(&mut updates);
                }

                if !update_accumulator.is_empty() || !rendering_infos.is_empty() {
                    if self.window_rendering_manager.as_ref().unwrap().should_render(){
                        self.window_rendering_manager.as_mut().unwrap().update(&mut update_accumulator);
                        match self.window_rendering_manager.as_mut().unwrap().render(rendering_infos) {
                            Ok(_) => {}
                            Err(e) => log::error!("{:?}", e),
                        }
                    }
                }
            }
        }
    }

    pub fn new(window_rendering_manager: Option<ScionWindowRenderingManager>, render_receiver: Receiver<(Vec<RendererEvent>, Vec<RenderingUpdate>, Vec<RenderingInfos>)>) -> Self{
        Self{
            window_rendering_manager,
            render_receiver,
        }
    }
}


