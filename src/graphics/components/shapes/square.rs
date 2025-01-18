use std::ops::Range;

use wgpu::{util::BufferInitDescriptor, PrimitiveTopology};

use crate::core::components::maths::coordinates::Coordinates;
use crate::core::components::maths::Pivot;
use crate::graphics::rendering::shaders::gl_representations::TexturedGlVertex;
use crate::utils::maths::Vector;
use crate::{
    graphics::components::material::Material,
    graphics::rendering::Renderable2D,
};

const INDICES: &[u16] = &[0, 1, 3, 3, 1, 2];

/// Renderable 2D Square.
pub struct Square {
    pub vertices: [Coordinates; 4],
    pub uvs: Option<[Coordinates; 4]>,
    contents: [TexturedGlVertex; 4],
    length: f32,
    pivot: Pivot
}

impl Square {
    /// Creates a new square using `length`.
    /// When rendering using a texture, you can customize uvs map using `uvs`. By default it will
    /// use 0 to 1 uvs
    pub fn new(length: f32, uvs: Option<[Coordinates; 4]>) -> Self {
        Square::new_with_pivot(length, uvs, Pivot::TopLeft)
    }

    /// Sets the pivot point of the square and returns it
    pub fn pivot(self, pivot: Pivot) -> Self {
        Square::new_with_pivot(self.length, self.uvs, pivot)
    }

    fn new_with_pivot(length: f32, uvs: Option<[Coordinates; 4]>, pivot: Pivot) -> Self {
        let offset = Self::compute_pivot_offset(&pivot, length);
        let a = Coordinates::new(0. - offset.x, 0. - offset.y);
        let b = Coordinates::new(a.x(), a.y() + length);
        let c = Coordinates::new(a.x() + length, a.y() + length);
        let d = Coordinates::new(a.x() + length, a.y());
        let uvs_ref = uvs.unwrap_or(default_uvs());
        let contents = [
            TexturedGlVertex::from((&a, &uvs_ref[0])),
            TexturedGlVertex::from((&b, &uvs_ref[1])),
            TexturedGlVertex::from((&c, &uvs_ref[2])),
            TexturedGlVertex::from((&d, &uvs_ref[3])),
        ];
        Self { vertices: [a, b, c, d], uvs, contents, length, pivot}
    }

    fn compute_pivot_offset(pivot: &Pivot, length: f32) -> Vector {
        match pivot {
            Pivot::TopLeft => Vector::new(0., 0.),
            Pivot::Center => Vector::new(length / 2., length / 2.),
            Pivot::Custom(x,y) => Vector::new(*x, *y)
        }
    }
}

fn default_uvs() -> [Coordinates; 4] {
    [
        Coordinates::new(0., 0.),
        Coordinates::new(0., 1.),
        Coordinates::new(1., 1.),
        Coordinates::new(1., 0.),
    ]
}

impl Renderable2D for Square {
    fn vertex_buffer_descriptor(&mut self, _material: Option<&Material>) -> BufferInitDescriptor {
        BufferInitDescriptor {
            label: Some("Square Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.contents),
            usage: wgpu::BufferUsages::VERTEX,
        }
    }

    fn indexes_buffer_descriptor(&self) -> BufferInitDescriptor {
        BufferInitDescriptor {
            label: Some("Square Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        }
    }

    fn range(&self) -> Range<u32> {
        0..INDICES.len() as u32
    }

    fn topology() -> PrimitiveTopology {
        wgpu::PrimitiveTopology::TriangleList
    }

    fn dirty(&self) -> bool {
        false
    }

    fn set_dirty(&mut self, _is_dirty: bool) {}

    fn get_pivot_offset(&self, _m: Option<&Material>) -> Vector {
        Self::compute_pivot_offset(&self.pivot, self.length)
    }
    fn get_pivot(&self) -> Pivot {
        self.pivot
    }
}
