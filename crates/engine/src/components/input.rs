#[derive(Debug, Clone, PartialEq)]
pub struct InputComponent {
    pub direction: [f32; 3],
    pub speed: f32,
}

impl Default for InputComponent {
    fn default() -> Self {
        Self {
            direction: [0.0, 0.0, 0.0],
            speed: 0.05,
        }
    }
}

impl InputComponent {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            ..Self::default()
        }
    }

    pub fn set_direction(&mut self, direction: [f32; 3]) {
        self.direction = direction;
    }
}
