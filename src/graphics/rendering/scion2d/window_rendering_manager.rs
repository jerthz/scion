use std::sync::Arc;
use std::sync::mpsc::Sender;
use wgpu::{Limits, Surface, SurfaceConfiguration};
use winit::window::Window;

use crate::graphics::components::color::Color;
use crate::graphics::rendering::scion2d::renderer::Scion2D;
use crate::graphics::rendering::{RendererCallbackEvent, RenderingInfos, RenderingUpdate};

pub(crate) struct ScionWindowRenderingManager {
    surface: Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: SurfaceConfiguration,
    scion_renderer: Scion2D,
    default_background_color: Option<Color>,
    should_render: bool,
    should_compute_cursor_color_picking: bool,
    cursor_position: Option<(u32, u32)>,
    render_callback_sender: Sender<RendererCallbackEvent>
}

impl ScionWindowRenderingManager {
    pub(crate) async fn new(window: Arc<Window>,
                            default_background : Option<Color>,
                            render_callback_sender: Sender<RendererCallbackEvent>) -> Self {
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        let backends = wgpu::util::backend_bits_from_env().unwrap_or_default();
        let dx12_shader_compiler = wgpu::util::dx12_shader_compiler_from_env().unwrap_or_default();
        let gles_minor_version = wgpu::util::gles_minor_version_from_env().unwrap_or_default();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            flags: wgpu::InstanceFlags::from_build_config().with_env(),
            dx12_shader_compiler,
            gles_minor_version,
        });

        let surface = instance.create_surface(window).expect("Surface creation failed");
        let adapter = wgpu::util::initialize_adapter_from_env_or_default(&instance, Some(&surface))
            .await
            .expect("No suitable GPU adapters found on the system!");

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::TEXTURE_BINDING_ARRAY | wgpu::Features::ADDRESS_MODE_CLAMP_TO_BORDER,
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    required_limits: Limits {
                        max_texture_array_layers: 512,
                        ..Limits::default()
                    }
                        .using_resolution(adapter.limits()),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let mut config = surface
            .get_default_config(&adapter, width, height)
            .unwrap();
        surface.configure(&device, &config);

        let mut scion_renderer = Scion2D::default();
        scion_renderer.start(&device, &config);

        Self { surface, device, queue, config, scion_renderer, default_background_color: default_background, should_render: true, should_compute_cursor_color_picking: true, cursor_position: None, render_callback_sender }
    }

    pub(crate) fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, _scale_factor: f64) {
        if new_size.width == 0 && new_size.height == 0 {
            self.should_render = false;
            return;
        }
        self.should_render = true;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
    }

    pub(crate) fn update(&mut self, updates: &mut Vec<RenderingUpdate>) {
        self.scion_renderer.update(updates, &self.device, &self.config, &mut self.queue);
    }

    pub(crate) fn update_cursor(&mut self, cursor_update: Option<(u32,u32)>) {
        self.cursor_position = cursor_update;
    }

    pub(crate) fn render(
        &mut self,
        data: Vec<RenderingInfos>,
    ) -> Result<(), wgpu::SurfaceError> {
        if !self.should_render {
            return Ok(());
        }

        let frame = self.surface.get_current_texture()?;
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: self.config.width,
                height: self.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[wgpu::TextureFormat::Depth32Float],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());


        if self.should_compute_cursor_color_picking && self.cursor_position.is_some() {
            self.compute_color_pixel(&data);
        }

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Window commands encoder"),
        });

        self.scion_renderer.render(
            data,
            &self.default_background_color,
            view,
            depth_view,
            &mut encoder,
        );

        self.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }

    fn compute_color_pixel(&mut self, data: &Vec<RenderingInfos>) {
        let depth_texture2 = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: self.config.width,
                height: self.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[wgpu::TextureFormat::Depth32Float],
        });
        let depth_view2 = depth_texture2.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Offscreen Command Encoder"),
        });
        let offscreen_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Offscreen Render Texture"),
            size: wgpu::Extent3d {
                width: self.config.width,
                height: self.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[wgpu::TextureFormat::Bgra8UnormSrgb],
        });
        let offscreen_view = offscreen_texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.scion_renderer.render(
            data.clone(),
            &self.default_background_color,
            offscreen_view,
            depth_view2,
            &mut encoder,
        );

        let (pixel_x, pixel_y) = self.cursor_position.as_ref().unwrap();
        let pixel_size = 4;

        let pixel_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Pixel Buffer"),
            size: pixel_size as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &offscreen_texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: *pixel_x,
                    y: *pixel_y,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &pixel_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: None,
                    rows_per_image: None,
                },
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(Some(encoder.finish()));
        self.device.poll(wgpu::Maintain::Wait);

        let buffer_slice = pixel_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device.poll(wgpu::Maintain::Wait);

        let mapped_range = buffer_slice.get_mapped_range();
        let b = mapped_range[0];
        let g = mapped_range[1];
        let r = mapped_range[2];

        let _r = self.render_callback_sender.send(RendererCallbackEvent::CursorColorPicking(Some(Color::new_rgb(r,g,b))));

        drop(mapped_range);
        pixel_buffer.unmap();
    }

    pub(crate) fn should_render(&self) -> bool {
        self.should_render
    }
}
