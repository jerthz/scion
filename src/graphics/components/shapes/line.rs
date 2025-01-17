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

const INDICES: &[u16] = &[0, 1];

/// Renderable 2D Line.
pub struct Line {
    pub vertices: [Coordinates; 2],
    contents: [TexturedGlVertex; 2],
    pivot: Pivot
}

impl Line {
    /// Creates a new line using `vertices`.
    pub fn new(vertices: [Coordinates; 2]) -> Self {
        Self::new_with_pivot(vertices, Pivot::TopLeft)
    }

    /// Sets the pivot point of the line and returns it
    pub fn pivot(self, pivot: Pivot) -> Self {
        Self::new_with_pivot(self.vertices, pivot)
    }

    fn compute_pivot_offset(pivot: &Pivot, vertices: &[Coordinates;2]) -> Vector {
        match pivot {
            Pivot::TopLeft => Vector::new(0., 0.),
            Pivot::Center => Vector::new(
                (vertices[1].x - vertices[0].x).abs() / 2.,
                (vertices[1].y - vertices[0].y).abs() / 2.,
            ),
            Pivot::Custom(x,y) => Vector::new(*x, *y)
        }
    }

    /// Creates a new line using `vertices`.
    fn new_with_pivot(vertices: [Coordinates; 2], pivot: Pivot) -> Self {
        let offset = Self::compute_pivot_offset(&pivot, &vertices);
        let contents = [
            TexturedGlVertex::from((
                &Coordinates::new(&vertices[0].x - offset.x, &vertices[0].y - offset.y),
                &Coordinates::new(0., 0.),
            )),
            TexturedGlVertex::from((
                &Coordinates::new(&vertices[1].x - offset.x, &vertices[1].y - offset.y),
                &Coordinates::new(0., 0.),
            )),
        ];
        Self { vertices, contents, pivot }
    }
}

impl Renderable2D for Line {
    fn vertex_buffer_descriptor(&mut self, _material: Option<&Material>) -> BufferInitDescriptor {
        BufferInitDescriptor {
            label: Some("Line Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.contents),
            usage: wgpu::BufferUsages::VERTEX,
        }
    }

    fn indexes_buffer_descriptor(&self) -> BufferInitDescriptor {
        BufferInitDescriptor {
            label: Some("Line Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        }
    }

    fn range(&self) -> Range<u32> {
        0..INDICES.len() as u32
    }

    fn topology() -> PrimitiveTopology {
        wgpu::PrimitiveTopology::LineList
    }

    fn dirty(&self) -> bool {
        false
    }

    fn set_dirty(&mut self, _is_dirty: bool) {}

    fn get_pivot_offset(&self, _material: Option<&Material>) -> Vector {
        Self::compute_pivot_offset(&self.pivot, &self.vertices)
    }
    fn get_pivot(&self) -> Pivot {
        self.pivot
    }
}
