#[derive(Debug, Clone, PartialEq)]
pub struct TerrainComponent {
    pub size: f32,
    pub height: f32,
    pub color: [f32; 3],
    pub texture: Option<String>,
    pub model_asset: String,
}

impl TerrainComponent {
    pub fn new(
        size: f32,
        height: f32,
        color: [f32; 3],
        texture: Option<String>,
        model_asset: String,
    ) -> Self {
        Self {
            size,
            height,
            color,
            texture,
            model_asset,
        }
    }
}
