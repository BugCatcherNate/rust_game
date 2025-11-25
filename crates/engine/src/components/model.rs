#[derive(Debug, Clone, PartialEq)]
pub struct ModelComponent {
    pub asset_path: String,
}

impl ModelComponent {
    pub fn new(asset_path: impl Into<String>) -> Self {
        Self {
            asset_path: asset_path.into(),
        }
    }
}
