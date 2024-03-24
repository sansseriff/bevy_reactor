use bevy::{
    math::{Rect, Vec3},
    render::mesh::{Indices, Mesh},
};

/// A marker for the start or end of a shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum StrokeMarker {
    /// No marker.
    #[default]
    None,

    /// Arrowhead marker.
    Arrowhead,
    // Future: Diamond, Circle, Square, etc.
}

/// A builder for creating two-dimensional shapes.
#[derive(Clone, Debug, Default)]
pub struct ShapeBuilder {
    vertices: Vec<Vec3>,
    indices: Vec<u32>,
    stroke_width: f32,
}

#[derive(Clone, Debug)]
pub struct PolygonOptions {
    pub closed: bool,
    pub dash_length: f32,
    pub gap_length: f32,
    pub start_marker: StrokeMarker,
    pub end_marker: StrokeMarker,
}

impl Default for PolygonOptions {
    fn default() -> Self {
        Self {
            closed: false,
            dash_length: f32::INFINITY,
            gap_length: 0.,
            start_marker: StrokeMarker::None,
            end_marker: StrokeMarker::None,
        }
    }
}

impl ShapeBuilder {
    /// Create a new `ShapeBuilder`.
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            stroke_width: 1.0,
        }
    }

    /// Set the stroke width for the shape.
    pub fn with_stroke_width(&mut self, stroke_width: f32) -> &mut Self {
        self.stroke_width = stroke_width;
        self
    }

    /// Reserve space for vertices and indices.
    pub fn reserve(&mut self, vertices: usize, indices: usize) -> &mut Self {
        self.vertices.reserve(vertices);
        self.indices.reserve(indices);
        self
    }

    /// Add a vertex to the shape.
    pub fn push_vertex(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.vertices.push(Vec3::new(x, y, z));
        self
    }

    /// Add an index to the shape.
    pub fn push_index(&mut self, index: u32) -> &mut Self {
        self.indices.push(index);
        self
    }

    /// Copy the shape into a [`Mesh`]. This will consume the builder and return a mesh.
    pub fn build(self, mesh: &mut Mesh) {
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.vertices);
        mesh.insert_indices(Indices::U32(self.indices));
        mesh.compute_aabb();
    }

    /// Draw a stroke in the shape of a rectangle.
    ///
    /// Arguments:
    /// `rect` - The outer bounds of the rectangle.
    pub fn stroke_rect(&mut self, rect: Rect) -> &mut Self {
        self.reserve(8, 24);

        let start = self.vertices.len() as u32;
        let lw = self.stroke_width;

        self.push_vertex(rect.min.x + lw, rect.min.y + lw, 0.);
        self.push_vertex(rect.min.x, rect.min.y, 0.);

        self.push_vertex(rect.max.x - lw, rect.min.y + lw, 0.);
        self.push_vertex(rect.max.x, rect.min.y, 0.);

        self.push_vertex(rect.max.x - lw, rect.max.y - lw, 0.);
        self.push_vertex(rect.max.x, rect.max.y, 0.);

        self.push_vertex(rect.min.x + lw, rect.max.y - lw, 0.);
        self.push_vertex(rect.min.x, rect.max.y, 0.);

        self.push_index(start);
        self.push_index(start + 1);
        self.push_index(start + 2);

        self.push_index(start + 1);
        self.push_index(start + 3);
        self.push_index(start + 2);

        self.push_index(start + 2);
        self.push_index(start + 3);
        self.push_index(start + 4);

        self.push_index(start + 4);
        self.push_index(start + 3);
        self.push_index(start + 5);

        self.push_index(start + 4);
        self.push_index(start + 5);
        self.push_index(start + 6);

        self.push_index(start + 5);
        self.push_index(start + 7);
        self.push_index(start + 6);

        self.push_index(start + 6);
        self.push_index(start + 1);
        self.push_index(start);

        self.push_index(start + 6);
        self.push_index(start + 7);
        self.push_index(start + 1);

        self
    }

    /// Draw a filled rectangle.
    pub fn fill_rect(&mut self, rect: Rect) -> &mut Self {
        self.reserve(4, 6);
        let start = self.vertices.len() as u32;
        self.push_vertex(rect.min.x, rect.min.y, 0.);
        self.push_vertex(rect.max.x, rect.min.y, 0.);
        self.push_vertex(rect.max.x, rect.max.y, 0.);
        self.push_vertex(rect.min.x, rect.max.y, 0.);
        self.push_index(start);
        self.push_index(start + 1);
        self.push_index(start + 2);
        self.push_index(start);
        self.push_index(start + 2);
        self.push_index(start + 3);
        self
    }

    /// Draw a circular stroke.
    pub fn stroke_circle(&mut self, x: f32, y: f32, radius: f32, segments: u32) -> &mut Self {
        let start = self.vertices.len() as u32;
        let step = 2.0 * std::f32::consts::PI / segments as f32;
        let radius_inner = (radius - self.stroke_width).max(0.0);
        let radius_outer = radius_inner + self.stroke_width;
        for i in 0..segments {
            let angle = i as f32 * step;
            let c = angle.cos();
            let s = angle.sin();
            let x_inner = x + radius_inner * c;
            let y_inner = y + radius_inner * s;
            let x_outer = x + radius_outer * c;
            let y_outer = y + radius_outer * s;
            self.push_vertex(x_inner, y_inner, 0.);
            self.push_vertex(x_outer, y_outer, 0.);
            self.push_index(start + i * 2);
            self.push_index(start + (i + 1) % segments);
        }
        self
    }

    /// Draw a filled circle.
    pub fn fill_circle(&mut self, x: f32, y: f32, radius: f32, segments: u32) -> &mut Self {
        let start = self.vertices.len() as u32;
        let step = 2.0 * std::f32::consts::PI / segments as f32;
        self.push_vertex(0., 0., 0.);
        for i in 0..segments {
            let angle = i as f32 * step;
            let x = x + radius * angle.cos();
            let y = y + radius * angle.sin();
            self.push_vertex(x, y, 0.);
            self.push_index(start);
            self.push_index(start + i);
            self.push_index(start + (i + 1) % segments);
        }
        self
    }

    /// Draw a polygon from a list of points.
    pub fn draw_polygon(&mut self, vertices: &[Vec3], options: PolygonOptions) -> &mut Self {
        self
    }
}
