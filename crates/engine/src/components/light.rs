#[derive(Debug, Clone, PartialEq)]
pub struct LightComponent {
    pub direction: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
}

impl LightComponent {
    pub fn directional(direction: [f32; 3], color: [f32; 3], intensity: f32) -> Self {
        let len_sq =
            direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2];
        let inv_len = if len_sq > 0.0 {
            1.0 / len_sq.sqrt()
        } else {
            1.0
        };
        Self {
            direction: [
                direction[0] * inv_len,
                direction[1] * inv_len,
                direction[2] * inv_len,
            ],
            color,
            intensity,
        }
    }
}
