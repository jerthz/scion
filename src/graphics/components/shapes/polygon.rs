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

/// Renderable 2D Polygon made of outlines.
#[derive(Debug)]
pub struct Polygon {
    pub vertices: Vec<Coordinates>,
    contents: Vec<TexturedGlVertex>,
    indices: Vec<u16>,
    pivot: Pivot,
    dirty: bool,
}

impl Polygon {
    /// Creates a new polygon using `vertices`.
    /// Rendering is done with strip, meaning if you give 3 vertices (a,b,c) then it will create a to b and b to c and c to a.
    pub fn new(vertices: Vec<Coordinates>) -> Self {
        Polygon::new_with_pivot(vertices, Pivot::TopLeft)
    }
    pub fn pivot(self, pivot: Pivot) -> Self {
        Polygon::new_with_pivot(self.vertices, pivot)
    }

    fn new_with_pivot(vertices: Vec<Coordinates>, pivot: Pivot) -> Self {
        let offset = Self::compute_pivot_offset(&pivot, &vertices);
        let mut contents: Vec<TexturedGlVertex> = vertices
            .iter()
            .map(|c| TexturedGlVertex::from((&Coordinates::new(c.x - offset.x, c.y - offset.y), &Coordinates::new(0., 0.))))
            .collect();
        let last_vertex = vertices.first().map(|c| TexturedGlVertex::from((&Coordinates::new(c.x - offset.x, c.y - offset.y), &Coordinates::new(0., 0.)))).unwrap();
        contents.push(last_vertex);
        let indices = (0..contents.len() as u16).collect();
        Self { vertices, contents, indices, pivot, dirty: true }
    }

    /// Retrieves the vertices list
    pub fn get_vertices(&self) -> &Vec<Coordinates> {
        &self.vertices
    }

    /// Append x to the nth vertice
    pub fn append_x(&mut self, index: usize, x: f32) {
        self.vertices.get_mut(index).unwrap().x += x;
        self.compute_contents();
    }

    /// Append y to the nth vertice
    pub fn append_y(&mut self, index: usize, y: f32) {
        self.vertices.get_mut(index).unwrap().y += y;
        self.compute_contents();
    }

    fn compute_contents(&mut self) {
        self.contents = self
            .vertices
            .iter()
            .map(|c| TexturedGlVertex::from((c, &Coordinates::new(0., 0.))))
            .collect();
        self.dirty = true;
    }

    fn compute_pivot_offset(pivot: &Pivot, vertices: &Vec<Coordinates>) -> Vector {
        match pivot {
            Pivot::TopLeft => Vector::new(0., 0.),
            Pivot::Center => {
                let centroid = crate::utils::maths::centroid_polygon(vertices);
                Vector::new(centroid.x, centroid.y)
            },
            Pivot::Custom(x,y) => Vector::new(*x, *y)
        }
    }
}

impl Renderable2D for Polygon {
    fn vertex_buffer_descriptor(&mut self, _material: Option<&Material>) -> BufferInitDescriptor {
        BufferInitDescriptor {
            label: Some("Polygon Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.contents),
            usage: wgpu::BufferUsages::VERTEX,
        }
    }

    fn indexes_buffer_descriptor(&self) -> BufferInitDescriptor {
        BufferInitDescriptor {
            label: Some("Polygon Index Buffer"),
            contents: bytemuck::cast_slice(self.indices.as_slice()),
            usage: wgpu::BufferUsages::INDEX,
        }
    }

    fn range(&self) -> Range<u32> {
        0..(self.indices.len()) as u32
    }

    fn topology() -> PrimitiveTopology {
        PrimitiveTopology::LineStrip
    }

    fn dirty(&self) -> bool {
        self.dirty
    }

    fn set_dirty(&mut self, is_dirty: bool) {
        self.dirty = is_dirty;
    }

    fn get_pivot_offset(&self, _material: Option<&Material>) -> Vector {
        Self::compute_pivot_offset(&self.pivot, &self.vertices)
    }
    fn get_pivot(&self) -> Pivot {
        self.pivot
    }
}
