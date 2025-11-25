#[derive(Debug, Clone, PartialEq)]
pub struct RenderComponent {
    pub color: [f32; 3],
    pub size: f32,
}

impl RenderComponent {
    pub fn new(color: [f32; 3], size: f32) -> Self {
        Self { color, size }
    }
}
