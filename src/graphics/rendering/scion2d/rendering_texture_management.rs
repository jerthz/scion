use wgpu::{BindGroup, BindGroupEntry, Device, Queue, SamplerBindingType, TexelCopyBufferLayout, Texture, TextureFormat, TextureViewDescriptor, TextureViewDimension};

use crate::graphics::components::material::TextureArray;

pub(crate) fn load_texture_array_to_queue(
    texture_array: TextureArray,
    queue: &mut Queue,
    device: &Device,
) -> (BindGroup, Texture) {
    let (unit_width, unit_height, _lines, total_sprites, mut array)
        = (texture_array.unit_width, texture_array.unit_height, texture_array.lines, texture_array.bytes_array.len(), texture_array.bytes_array);

    let texture_size = wgpu::Extent3d {
        width: unit_width,
        height: unit_height,
        depth_or_array_layers: total_sprites as u32,
    };
    let diffuse_texture_descriptor = wgpu::TextureDescriptor {
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        label: Some("diffuse texture array"),
        format: TextureFormat::Rgba8UnormSrgb,
        view_formats: &[TextureFormat::Rgba8UnormSrgb],
    };
    let tileset_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("tileset"),
        view_formats: &[],
        ..diffuse_texture_descriptor
    });

    let mut counter = 0;
    array.drain(0..total_sprites)
        .for_each(|sprite| {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &tileset_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: counter as u32,
                    },
                    aspect: Default::default(),
                },
                &sprite,
                TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * unit_width),
                    rows_per_image: Some(unit_height),
                },
                wgpu::Extent3d {
                    width: unit_width,
                    height: unit_height,
                    depth_or_array_layers: 1,
                },
            );
            counter+=1;
        });
    let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToBorder,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let diffuse_bind_group = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("bind group layout sprite"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2Array,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            }
        ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &diffuse_bind_group,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(
                &tileset_texture.create_view(&TextureViewDescriptor {
                    dimension: Some(TextureViewDimension::D2Array),
                    ..TextureViewDescriptor::default()
                }),
            ),
        },
            BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
            }],
        label: None,
    });

    (bind_group, tileset_texture)
}