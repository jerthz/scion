use std::collections::HashMap;

use hecs::{Component, Entity};
use wgpu::{BindGroup, BindGroupLayout, Buffer, CommandEncoder, Device, Queue, RenderPassColorAttachment, RenderPipeline, SamplerBindingType, StoreOp, SurfaceConfiguration, TextureFormat, TextureView, util::DeviceExt};
use wgpu::util::BufferInitDescriptor;

use crate::{
    graphics::components::{
        color::Color

        ,
        material::Texture
        ,
        shapes::{
            line::Line, polygon::Polygon, rectangle::Rectangle, square::Square, triangle::Triangle,
        },
        tiles::{
            sprite::Sprite,
            tilemap::Tilemap,
        }, ui::{ui_image::UiImage, ui_text::UiTextImage},
    },
    graphics::rendering::{
        Renderable2D
        , shaders::pipeline::pipeline,
    }
    ,
};
use crate::graphics::rendering::{DiffuseBindGroupUpdate, RenderingInfos, RenderingUpdate};
use crate::graphics::rendering::scion2d::rendering_texture_management::load_texture_array_to_queue;
use crate::graphics::rendering::shaders::gl_representations::GlUniform;
use crate::graphics::rendering::shaders::pipeline::pipeline_sprite;

#[derive(Default)]
pub(crate) struct Scion2D {
    vertex_buffers: HashMap<Entity, Buffer>,
    index_buffers: HashMap<Entity, Buffer>,
    render_pipelines: HashMap<String, RenderPipeline>,
    texture_bind_group_layout: Option<BindGroupLayout>,
    texture_array_bind_group_layout: Option<BindGroupLayout>,
    transform_bind_group_layout: Option<BindGroupLayout>,
    diffuse_bind_groups: HashMap<String, (BindGroup, wgpu::Texture)>,
    transform_uniform_bind_groups: HashMap<Entity, (GlUniform, Buffer, BindGroup)>,
}
impl Scion2D {

    pub(crate) fn start(&mut self, device: &Device, surface_config: &SurfaceConfiguration) {
        self.transform_bind_group_layout = Some(Self::create_uniform_bind_group_layout(device));
        self.texture_bind_group_layout = Some(Self::create_texture_bind_group_layout(device));
        self.texture_array_bind_group_layout = Some(Self::create_texture_array_bind_group_layout(device));
        self.insert_components_pipelines::<Triangle>(&device, &surface_config);
        self.insert_components_pipelines::<Square>(&device, &surface_config);
        self.insert_components_pipelines::<Rectangle>(&device, &surface_config);
        self.insert_components_pipelines::<Sprite>(&device, &surface_config);
        self.insert_components_pipelines::<Line>(&device, &surface_config);
        self.insert_components_pipelines::<Polygon>(&device, &surface_config);
        self.insert_components_pipelines::<UiImage>(&device, &surface_config);
        self.insert_components_pipelines::<UiTextImage>(&device, &surface_config);
        self.insert_components_pipelines::<Tilemap>(&device, &surface_config);
    }

    pub(crate) fn update(
        &mut self,
        data: &mut Vec<RenderingUpdate>,
        device: &Device,
        _surface_config: &SurfaceConfiguration,
        queue: &mut Queue,
    ) {
        for update in data.drain(0..data.len()) {
            match update {
                RenderingUpdate::DiffuseBindGroup { data, path } => {
                    self.update_material(device, queue, data, path);
                }
                RenderingUpdate::TransformUniform { entity, uniform } => {
                    self.update_transform_uniform(device, queue, entity, uniform);
                }
                RenderingUpdate::VertexBuffer { entity,  contents, usage } => {
                    let vertex_buffer =
                        device.create_buffer_init(&BufferInitDescriptor {
                            label: Some("Vertex buffer"),
                            contents: contents.as_slice(),
                            usage,
                        });
                    self.vertex_buffers.insert(entity, vertex_buffer);
                }
                RenderingUpdate::IndexBuffer { entity, contents, usage } => {
                    let index_buffer =
                        device.create_buffer_init(&BufferInitDescriptor {
                            label: Some("Index buffer"),
                            contents: contents.as_slice(),
                            usage,
                        });
                    self.index_buffers.insert(entity, index_buffer);
                }
            }
        }

        // FIXME : self.clean_buffers(data);
    }

    pub(crate) fn render(
        &mut self,
        data: Vec<RenderingInfos>,
        default_background: &Option<Color>,
        texture_view: TextureView,
        depth_texture_view: TextureView,
        encoder: &mut CommandEncoder,
    ) {
        self.render_component(default_background, texture_view, depth_texture_view, encoder, data);
    }

    fn insert_components_pipelines<T: Component + Renderable2D>(
        &mut self,
        device: &&Device,
        surface_config: &&SurfaceConfiguration,
    ) {
        self.insert_pipeline_if_not_finded::<T>(device, surface_config);
    }

    fn insert_pipeline_if_not_finded<T: Component + Renderable2D>(
        &mut self,
        device: &&Device,
        surface_config: &&SurfaceConfiguration,
    ) {
        let type_name = std::any::type_name::<T>();
        if !self.render_pipelines.contains_key(type_name) {
            self.render_pipelines.insert(
                type_name.to_string(),
                if type_name.eq_ignore_ascii_case("scion::graphics::components::tiles::sprite::Sprite") ||
                    type_name.eq_ignore_ascii_case("scion::graphics::components::tiles::tilemap::Tilemap") {
                    pipeline_sprite(
                        device,
                        surface_config,
                        self.texture_array_bind_group_layout.as_ref().unwrap(),
                        self.transform_bind_group_layout.as_ref().unwrap(),
                        T::topology(),
                    )
                } else {
                    pipeline(
                        device,
                        surface_config,
                        self.texture_bind_group_layout.as_ref().unwrap(),
                        self.transform_bind_group_layout.as_ref().unwrap(),
                        T::topology(),
                    )
                }
                ,
            );
        }
    }

    fn render_component(
        &mut self,
        default_background: &Option<Color>,
        texture_view: TextureView,
        depth_texture_view: TextureView,
        encoder: &mut CommandEncoder,
        mut infos: Vec<RenderingInfos>,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[get_default_color_attachment(&texture_view, default_background)],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        infos.sort_by(|a, b| a.layer.cmp(&b.layer));

        while let Some(rendering_infos) = infos.pop() {
            render_pass.set_bind_group(
                0,
                &self.transform_uniform_bind_groups.get(&rendering_infos.entity).unwrap().2,
                &[],
            );
            render_pass.set_vertex_buffer(
                0,
                self.vertex_buffers.get(&rendering_infos.entity).as_ref().unwrap().slice(..),
            );
            render_pass.set_index_buffer(
                self.index_buffers.get(&rendering_infos.entity).as_ref().unwrap().slice(..),
                wgpu::IndexFormat::Uint16,
            );
            render_pass.set_pipeline(
                self.render_pipelines.get(rendering_infos.type_name.as_str()).as_ref().unwrap(),
            );
            if let Some(path) = rendering_infos.texture_path {
                render_pass.set_bind_group(
                    1,
                    &self.diffuse_bind_groups.get(path.as_str()).unwrap().0,
                    &[],
                );
            }

            render_pass.draw_indexed(rendering_infos.range, 0, 0..1);
        }
    }

    fn update_material(&mut self, device: &Device, queue: &mut Queue, data: DiffuseBindGroupUpdate, path: String) {
        match data {
            DiffuseBindGroupUpdate::ColorBindGroup(tex) => {
                self.diffuse_bind_groups.insert(
                    path,
                    load_texture_to_queue(
                        tex,
                        queue,
                        device,
                        self.texture_bind_group_layout.as_ref().unwrap(),
                    ),
                );
            }
            DiffuseBindGroupUpdate::TextureBindGroup(tex) => {
                if self.diffuse_bind_groups.contains_key(path.as_str()) {
                    self.diffuse_bind_groups
                        .get(path.as_str())
                        .expect("Unreachable diffuse bind group after check")
                        .1.destroy();
                    self.diffuse_bind_groups.remove(path.as_str());
                }
                self.diffuse_bind_groups.insert(
                    path,
                    load_texture_to_queue(
                        tex,
                        queue,
                        device,
                        self.texture_bind_group_layout.as_ref().unwrap(),
                    ),
                );
            }
            DiffuseBindGroupUpdate::TilesetBindGroup(texture_array) => {
                self.diffuse_bind_groups.insert(
                    path,
                    load_texture_array_to_queue(texture_array, queue, device),
                );
            }
        }
    }

    fn update_transform_uniform(&mut self, device: &Device, queue: &mut Queue, entity: Entity, uniform: GlUniform) {
        if let std::collections::hash_map::Entry::Vacant(e) = self.transform_uniform_bind_groups.entry(entity) {
            let (uniform, uniform_buffer, group) = create_transform_uniform_bind_group(
                device,
                uniform,
                self.transform_bind_group_layout.as_ref().unwrap(),
            );
            queue.write_buffer(&uniform_buffer, 0, bytemuck::cast_slice(&[uniform]));
            e.insert((uniform, uniform_buffer, group));
        } else {
            let (current_uniform, uniform_buffer, _) = self
                .transform_uniform_bind_groups
                .get_mut(&entity)
                .expect("Fatal error, a transform has been marked as found but doesn't exist");
            current_uniform.replace_with(uniform);
            queue.write_buffer(uniform_buffer, 0, bytemuck::cast_slice(&[*current_uniform]));
        }
    }

    fn create_uniform_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("uniform_bind_group_layout"),
        })
    }

    fn create_texture_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        })
    }

    fn create_texture_array_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        })
    }
}

fn load_texture_to_queue(
    texture: Texture,
    queue: &mut Queue,
    device: &Device,
    texture_bind_group_layout: &BindGroupLayout,
) -> (BindGroup, wgpu::Texture) {
    let texture_size = wgpu::Extent3d {
        width: texture.width,
        height: texture.height,
        depth_or_array_layers: 1,
    };

    let diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        label: Some("diffuse_texture"),
        format: TextureFormat::Rgba8UnormSrgb,
        view_formats: &[TextureFormat::Rgba8UnormSrgb],
    });

    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &diffuse_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &texture.bytes,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * &texture.width),
            rows_per_image: Some(texture.height),
        },
        texture_size,
    );
    let diffuse_texture_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
            },
        ],
        label: Some("diffuse_bind_group"),
    });

    (diffuse_bind_group, diffuse_texture)
}

fn create_transform_uniform_bind_group(
    device: &Device,
    gl_uniform: GlUniform,
    uniform_bind_group_layout: &BindGroupLayout,
) -> (GlUniform, Buffer, BindGroup) {
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Uniform Buffer"),
        contents: bytemuck::cast_slice(&[gl_uniform]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: uniform_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
        label: Some("uniform_bind_group"),
    });

    (gl_uniform, uniform_buffer, uniform_bind_group)
}

fn get_default_color_attachment<'a>(
    texture_view: &'a TextureView,
    default_background: &'a Option<Color>,
) -> Option<RenderPassColorAttachment<'a>> {
    Some(RenderPassColorAttachment {
        view: texture_view,
        resolve_target: None,
        ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(
                if let Some(color) = &default_background {
                    color.to_linear()
                } else {
                    wgpu::Color { r: 1., g: 0., b: 0., a: 1.0 }
                },
            ),
            store: StoreOp::Store,
        },
    })
}
