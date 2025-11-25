#[derive(Debug, Clone, PartialEq)]
pub struct CameraComponent {
    pub yaw: f32,
    pub pitch: f32,
    pub move_speed: f32,
    pub look_sensitivity: f32,
}

impl CameraComponent {
    pub fn new(yaw: f32, pitch: f32) -> Self {
        Self {
            yaw,
            pitch,
            move_speed: 0.05,
            look_sensitivity: 0.0025,
        }
    }
}
