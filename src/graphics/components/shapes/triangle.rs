use std::ops::Range;

use wgpu::{util::BufferInitDescriptor, PrimitiveTopology};

use crate::core::components::maths::coordinates::Coordinates;
use crate::core::components::maths::Pivot;
use crate::graphics::rendering::shaders::gl_representations::TexturedGlVertex;
use crate::{
    graphics::components::material::Material,
    graphics::rendering::Renderable2D,
    utils::maths::Vector,
};

const INDICES: &[u16] = &[1, 0, 2];

/// Renderable 2D Triangle.
pub struct Triangle {
    pub vertices: [Coordinates; 3],
    pub uvs: Option<[Coordinates; 3]>,
    contents: [TexturedGlVertex; 3],
    pivot: Pivot
}

impl Triangle {
    /// Creates a new square using `length`.
    /// uvs are mandatory
    pub fn new(vertices: [Coordinates; 3], uvs: Option<[Coordinates; 3]>) -> Self {
        Triangle::new_with_pivot(vertices, uvs, Pivot::TopLeft)
    }

    pub fn pivot(self, pivot: Pivot) -> Self {
        Triangle::new_with_pivot(self.vertices, self.uvs, pivot)
    }

    fn new_with_pivot(
        vertices: [Coordinates; 3],
        uvs: Option<[Coordinates; 3]>,
        pivot: Pivot,
    ) -> Self {
        let uvs_ref = uvs.as_ref().expect("Uvs are currently mandatory, this need to be fixed");
        let offset = Self::compute_pivot_offset(&pivot, &vertices);
        let a = Coordinates::new(&vertices[0].x - offset.x, &vertices[0].y - offset.y);
        let b = Coordinates::new(&vertices[1].x - offset.x, &vertices[1].y - offset.y);
        let c = Coordinates::new(&vertices[2].x - offset.x, &vertices[2].y - offset.y);
        let contents = [
            TexturedGlVertex::from((&a, &uvs_ref[0])),
            TexturedGlVertex::from((&b, &uvs_ref[1])),
            TexturedGlVertex::from((&c, &uvs_ref[2])),
        ];
        Self { vertices, uvs, contents, pivot }
    }

    fn compute_pivot_offset(pivot: &Pivot,
                            vertices: &[Coordinates; 3]) -> Vector {
        match pivot {
            Pivot::TopLeft => Vector::new(0., 0.),
            Pivot::Center => Vector::new(
                (vertices[0].x + vertices[1].x + vertices[2].x).abs() / 3.,
                (vertices[0].y + vertices[1].y + vertices[2].y) / 3.,
            ),
            Pivot::Custom(x,y) => Vector::new(*x, *y)
        }
    }
}

impl Renderable2D for Triangle {
    fn vertex_buffer_descriptor(&mut self, _material: Option<&Material>) -> BufferInitDescriptor {
        BufferInitDescriptor {
            label: Some("Triangle Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.contents),
            usage: wgpu::BufferUsages::VERTEX,
        }
    }

    fn indexes_buffer_descriptor(&self) -> BufferInitDescriptor {
        BufferInitDescriptor {
            label: Some("Triangle Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        }
    }

    fn range(&self) -> Range<u32> {
        0..3_u32
    }

    fn topology() -> PrimitiveTopology {
        PrimitiveTopology::TriangleList
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
