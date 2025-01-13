//! Everything that is relative to graphics to the window (Like renderable components, camera, transforms..)
use std::ops::Range;

use hecs::Entity;
use wgpu::{BufferUsages, util::BufferInitDescriptor};
use winit::dpi::PhysicalSize;

use crate::graphics::components::material::{Material, Texture, TextureArray};
use crate::core::components::maths::Pivot;
use shaders::gl_representations::GlUniform;
use crate::graphics::components::color::Color;
use crate::graphics::rendering::shaders::gl_representations::ColorPickingUniform;
use crate::utils::maths::Vector;

pub(crate) mod shaders;
pub(crate) mod scion2d;

pub(crate) trait Renderable2D {
    fn vertex_buffer_descriptor(&mut self, material: Option<&Material>) -> BufferInitDescriptor;
    fn indexes_buffer_descriptor(&self) -> BufferInitDescriptor;
    fn range(&self) -> Range<u32>;
    fn topology() -> wgpu::PrimitiveTopology;
    fn dirty(&self) -> bool;
    fn set_dirty(&mut self, is_dirty: bool);
    fn get_pivot_offset(&self, _material: Option<&Material>) -> Vector { Vector::default() }
    fn get_pivot(&self) -> Pivot { Pivot::TopLeft }
    fn get_rendering_priority(&self) -> usize { 0 }
    fn color_picking_enabled(&self) -> bool { false }
}

pub(crate) trait RenderableUi: Renderable2D {}


#[derive(Debug)]
pub(crate) enum RenderingUpdate {
    DiffuseBindGroup {
        path: String,
        data: DiffuseBindGroupUpdate,
    },
    TransformUniform {
        entity: Entity,
        uniform: GlUniform,
    },
    ColorPickingUniform {
        entity: Entity,
        uniform: ColorPickingUniform,
    },
    VertexBuffer{
        entity: Entity,
        contents: Vec<u8>,
        usage: BufferUsages

    },
    IndexBuffer{
        entity: Entity,
        contents: Vec<u8>,
        usage: BufferUsages
    }
}

pub enum RendererEvent {
    ForceRedraw,
    CursorPositionUpdate(Option<(u32,u32)>),
    Resize(PhysicalSize<u32>, f64)
}

#[derive(Debug)]
pub enum RendererCallbackEvent {
    CursorColorPicking(Option<Color>)
}


#[derive(Debug)]
pub(crate) enum DiffuseBindGroupUpdate {
    ColorBindGroup(Texture),
    TextureBindGroup(Texture),
    TilesetBindGroup(TextureArray),
}

#[derive(Debug, Clone)]
pub struct RenderingInfos {
    pub(crate) layer: usize,
    pub(crate) range: Range<u32>,
    pub(crate) entity: Entity,
    pub(crate) texture_path: Option<String>,
    pub(crate) type_name: String,
    pub(crate) render_priority: usize,
}
