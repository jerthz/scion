use std::ops::Range;

use wgpu::{PrimitiveTopology, util::BufferInitDescriptor};

use crate::{
    graphics::components::{
        material::Material,
    },
    graphics::rendering::Renderable2D,
};
use crate::core::components::maths::coordinates::Coordinates;
use crate::core::components::maths::Pivot;
use crate::graphics::rendering::shaders::gl_representations::TexturedGlVertexWithLayer;
use crate::utils::maths::Vector;

const INDICES: &[u16] = &[0, 1, 3, 3, 1, 2];

/// Renderable Sprite.
#[derive(Debug)]
pub struct Sprite {
    /// Desired tile to render for this material.
    tile_number: usize,
    /// Current computed content for vertex
    contents: Option<[TexturedGlVertexWithLayer; 4]>,
    /// Flag to keep track of changed tile number
    dirty: bool,
    /// Pivot point of the sprite, default topleft
    pivot: Pivot,
}

impl Sprite {
    /// Creates a new sprite that will use the `tile_number` from the tileset associated in the same
    /// entity
    pub fn new(tile_number: usize) -> Self {
        Self { tile_number, contents: None, dirty: false, pivot: Pivot::TopLeft}
    }

    pub fn pivot(self, pivot: Pivot) -> Self {
        Self { tile_number: self.tile_number, contents: None, dirty: false, pivot}
    }

    /// Modify the current sprite tile number
    pub fn set_tile_nb(&mut self, new_tile_nb: usize) {
        self.tile_number = new_tile_nb;
        self.dirty = true;
    }

    pub fn get_tile_nb(&self) -> usize {
        self.tile_number
    }

    fn compute_pivot_offset(pivot: &Pivot, width: usize, height: usize) -> Vector {
        match pivot {
            Pivot::TopLeft => Vector::new(0., 0.),
            Pivot::Center => Vector::new(width as f32 / 2., height as f32 / 2.),
            Pivot::Custom(x, y) => Vector::new(*x, *y)
        }
    }

    fn uv_refs(&self) -> [Coordinates; 4] {
        let a = Coordinates::new(0., 0.);
        let b = Coordinates::new(0., 1.);
        let c = Coordinates::new(1., 1.);
        let d = Coordinates::new(1., 0.);
        [a, b, c, d]
    }

    pub(crate) fn compute_content(&self, material: Option<&Material>) -> [TexturedGlVertexWithLayer; 4] {
        if (self.dirty || self.contents.is_none()) && material.is_some() {
            if let Material::Tileset(tileset) = material.unwrap() {
                let offset = Self::compute_pivot_offset(&self.pivot, tileset.tile_width, tileset.tile_height);
                let a = Coordinates::new(0. - offset.x, 0. - offset.y);
                let b = Coordinates::new(a.x, a.y + tileset.tile_height as f32);
                let c = Coordinates::new(a.x + tileset.tile_width as f32, a.y + tileset.tile_height as f32);
                let d = Coordinates::new(a.x + tileset.tile_width as f32, a.y);
                let uvs_ref = self.uv_refs();
                return [
                    TexturedGlVertexWithLayer::from((&a, &uvs_ref[0], self.tile_number, 0.)),
                    TexturedGlVertexWithLayer::from((&b, &uvs_ref[1], self.tile_number, 0.)),
                    TexturedGlVertexWithLayer::from((&c, &uvs_ref[2], self.tile_number, 0.)),
                    TexturedGlVertexWithLayer::from((&d, &uvs_ref[3], self.tile_number, 0.)),
                ];
            }
        }
        *self.contents.as_ref().expect("A computed content is missing in Sprite component")
    }

    pub(crate) fn indices() -> Vec<u16> {
        INDICES.to_vec()
    }

    pub(crate) fn set_content(&mut self, content: [TexturedGlVertexWithLayer; 4]) {
        self.contents = Some(content);
    }
}

impl Renderable2D for Sprite {
    fn vertex_buffer_descriptor(&mut self, material: Option<&Material>) -> BufferInitDescriptor {
        let content = self.compute_content(material);
        self.contents = Some(content);
        BufferInitDescriptor {
            label: Some("Sprite Vertex Buffer"),
            contents: bytemuck::cast_slice(
                self.contents.as_ref().expect("A computed content is missing in Sprite component"),
            ),
            usage: wgpu::BufferUsages::VERTEX,
        }
    }

    fn indexes_buffer_descriptor(&self) -> BufferInitDescriptor {
        BufferInitDescriptor {
            label: Some("Sprite Index Buffer"),
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
        self.dirty
    }

    fn set_dirty(&mut self, is_dirty: bool) {
        self.dirty = is_dirty;
    }

    fn get_pivot_offset(&self, material: Option<&Material>) -> Vector {
        if material.is_none() {
            Vector::default()
        } else if let Material::Tileset(tileset) = material.unwrap() {
            Self::compute_pivot_offset(&self.pivot, tileset.tile_width, tileset.tile_height)
        } else { Vector::default() }
    }
    fn get_pivot(&self) -> Pivot {
        self.pivot
    }

    fn color_picking_enabled(&self) -> bool {
        true
    }
}
