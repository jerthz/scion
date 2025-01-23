use std::ops::Range;

use wgpu::{util::BufferInitDescriptor, PrimitiveTopology};

use crate::core::components::maths::coordinates::Coordinates;
use crate::graphics::rendering::shaders::gl_representations::TexturedGlVertex;
use crate::{
    graphics::components::material::Material,
    graphics::rendering::{Renderable2D, RenderableUi},
};

/// Renderable 2D UIImage
#[derive(Debug)]
pub struct UiImage {
    contents: [TexturedGlVertex; 4],
}

const INDICES: &[u16] = &[0, 1, 3, 3, 1, 2];

impl UiImage {
    /// Creates a ui_image with `width` and `height` using image at `image_path`
    pub fn new(width: f32, height: f32) -> Self {
        let uvs = [
            Coordinates::new(0., 0.),
            Coordinates::new(0., 1.),
            Coordinates::new(1., 1.),
            Coordinates::new(1., 0.),
        ];
        UiImage::new_with_uv_map(width, height, uvs)
    }

    /// Creates a ui_image with `width` and `height` using image at `image_path`
    /// and customising uv_map
    pub fn new_with_uv_map(
        width: f32,
        height: f32,
        uvs: [Coordinates; 4],
    ) -> Self {
        let a = Coordinates::new(0., 0.);
        let b = Coordinates::new(a.x(), a.y() + height);
        let c = Coordinates::new(a.x() + width, a.y() + height);
        let d = Coordinates::new(a.x() + width, a.y());

        let contents = [
            TexturedGlVertex::from((&a, &uvs[0])),
            TexturedGlVertex::from((&b, &uvs[1])),
            TexturedGlVertex::from((&c, &uvs[2])),
            TexturedGlVertex::from((&d, &uvs[3])),
        ];

        Self { contents }
    }
}

impl Renderable2D for UiImage {
    fn vertex_buffer_descriptor(&mut self, _material: Option<&Material>) -> BufferInitDescriptor {
        BufferInitDescriptor {
            label: Some("UI Image vertex buffer"),
            contents: bytemuck::cast_slice(&self.contents),
            usage: wgpu::BufferUsages::VERTEX,
        }
    }

    fn indexes_buffer_descriptor(&self) -> BufferInitDescriptor {
        BufferInitDescriptor {
            label: Some("UI Image index buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        }
    }

    fn range(&self) -> Range<u32> {
        0..INDICES.len() as u32
    }

    fn topology() -> PrimitiveTopology {
        PrimitiveTopology::TriangleList
    }

    fn dirty(&self) -> bool {
        false
    }

    fn set_dirty(&mut self, _is_dirty: bool) {}

    fn get_rendering_priority(&self) -> usize {
        0
    }

}

impl RenderableUi for UiImage {

}
