#[derive(Debug, Clone, PartialEq)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex {
    pub fn new(position: [f32; 3], tex_coords: [f32; 2]) -> Self {
        Self { position, tex_coords }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub vertices: Vec<Vertex>,
}

impl Model {
    /// Create a new empty model
    pub fn new() -> Self {
        Self { vertices: Vec::new() }
    }

    /// Create a model from a set of vertices
    pub fn from_vertices(vertices: Vec<Vertex>) -> Self {
        Self { vertices }
    }

    /// Add a vertex dynamically
    pub fn add_vertex(&mut self, vertex: Vertex) {
        self.vertices.push(vertex);
    }

    /// Clear all vertices
    pub fn clear(&mut self) {
        self.vertices.clear();
    }
}

