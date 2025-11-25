#[derive(Debug, Clone, PartialEq)]
pub struct InputComponent {
    pub direction: [f32; 3],
    pub speed: f32,
    pub jump_requested: bool,
}

impl Default for InputComponent {
    fn default() -> Self {
        Self {
            direction: [0.0, 0.0, 0.0],
            speed: 0.05,
            jump_requested: false,
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

    pub fn request_jump(&mut self) {
        self.jump_requested = true;
    }

    pub fn take_jump_request(&mut self) -> bool {
        if self.jump_requested {
            self.jump_requested = false;
            true
        } else {
            false
        }
    }
}
