use crate::components::Orientation;
use crate::ecs::ECS;

pub struct CameraSystem;

impl CameraSystem {
    pub fn apply_mouse_delta(ecs: &mut ECS, entity_id: u32, delta: (f64, f64)) {
        let mut yaw_pitch = None;
        if let Some(camera) = ecs.camera_component_mut(entity_id) {
            let (delta_x, delta_y) = delta;
            let sensitivity = camera.look_sensitivity;
            camera.yaw += delta_x as f32 * sensitivity;
            camera.pitch = (camera.pitch - delta_y as f32 * sensitivity).clamp(
                -std::f32::consts::FRAC_PI_2 + 0.01,
                std::f32::consts::FRAC_PI_2 - 0.01,
            );
            yaw_pitch = Some((camera.yaw, camera.pitch));
        }
        if let Some((yaw, pitch)) = yaw_pitch {
            if let Some(orientation) = ecs.orientation_component_mut(entity_id) {
                *orientation = Orientation::from_yaw_pitch_roll(yaw, pitch, 0.0);
            }
        }
    }
}
