#[derive(Debug, Clone)]
pub struct ParticleEmitterComponent {
    pub rate: f32,
    pub lifetime: f32,
    pub speed: f32,
    pub spread: f32,
    pub direction: [f32; 3],
    pub size: f32,
    pub size_jitter: f32,
    pub color: [f32; 3],
    pub color_jitter: f32,
    pub model_asset: String,
    pub texture_asset: Option<String>,
    pub max_particles: usize,
    pub spawn_accumulator: f32,
    pub seed: u32,
}

impl ParticleEmitterComponent {
    pub fn new(
        rate: f32,
        lifetime: f32,
        speed: f32,
        spread: f32,
        direction: [f32; 3],
        size: f32,
        size_jitter: f32,
        color: [f32; 3],
        color_jitter: f32,
        model_asset: String,
        texture_asset: Option<String>,
        max_particles: usize,
    ) -> Self {
        Self {
            rate,
            lifetime,
            speed,
            spread,
            direction,
            size,
            size_jitter,
            color,
            color_jitter,
            model_asset,
            texture_asset,
            max_particles,
            spawn_accumulator: 0.0,
            seed: 1,
        }
    }

    pub fn next_unit_random(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.seed as f32 / u32::MAX as f32).clamp(0.0, 1.0)
    }
}

#[derive(Debug, Clone)]
pub struct ParticleComponent {
    pub emitter_id: u32,
    pub velocity: [f32; 3],
    pub age: f32,
    pub lifetime: f32,
}

impl ParticleComponent {
    pub fn new(emitter_id: u32, velocity: [f32; 3], lifetime: f32) -> Self {
        Self {
            emitter_id,
            velocity,
            age: 0.0,
            lifetime,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParticleBurstRequest {
    pub position: [f32; 3],
    pub direction: [f32; 3],
    pub count: usize,
    pub speed: f32,
    pub spread: f32,
    pub lifetime: f32,
    pub size: f32,
    pub size_jitter: f32,
    pub color: [f32; 3],
    pub color_jitter: f32,
    pub model_asset: String,
    pub texture_asset: Option<String>,
    pub seed: u32,
}
