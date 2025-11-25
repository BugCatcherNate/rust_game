#[derive(Debug, Clone, PartialEq)]
pub struct TextureComponent {
    pub asset_path: String,
}

impl TextureComponent {
    pub fn new(asset_path: impl Into<String>) -> Self {
        Self {
            asset_path: asset_path.into(),
        }
    }
}
