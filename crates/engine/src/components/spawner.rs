#[derive(Debug, Clone, PartialEq)]
pub struct SpawnerComponent {
    pub template: String,
    pub interval: f32,
    pub spawn_on_load: bool,
    pub timer: f32,
}

impl SpawnerComponent {
    pub fn new(template: String, interval: f32, spawn_on_load: bool) -> Self {
        Self {
            template,
            interval,
            spawn_on_load,
            timer: 0.0,
        }
    }
}
