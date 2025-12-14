use std::collections::HashSet;

use winit::keyboard::KeyCode;

use crate::components::InputComponent;
use crate::ecs::ECS;

pub struct InputSystem;

impl InputSystem {
    pub fn update_entity_from_keys(
        ecs: &mut ECS,
        entity_id: u32,
        pressed_keys: &HashSet<KeyCode>,
        just_pressed: &HashSet<KeyCode>,
        free_flight: bool,
    ) {
        let camera_yaw = ecs.camera_component(entity_id).map(|camera| camera.yaw);
        let direction = Self::direction_from_keys(pressed_keys, camera_yaw, free_flight);
        let jump_requested = !free_flight && just_pressed.contains(&KeyCode::Space);
        if let Some(input) = ecs.input_component_mut(entity_id) {
            input.set_direction(direction);
            if jump_requested {
                input.request_jump();
            }
        } else {
            let mut component = InputComponent::new(0.05);
            component.set_direction(direction);
            if jump_requested {
                component.request_jump();
            }
            ecs.add_input_component(entity_id, component);
        }
    }

    fn direction_from_keys(
        pressed_keys: &HashSet<KeyCode>,
        camera_yaw: Option<f32>,
        free_flight: bool,
    ) -> [f32; 3] {
        let mut x: f32 = 0.0;
        let mut y: f32 = 0.0;
        let mut z: f32 = 0.0;

        if pressed_keys.contains(&KeyCode::KeyA) || pressed_keys.contains(&KeyCode::ArrowLeft) {
            x -= 1.0;
        }
        if pressed_keys.contains(&KeyCode::KeyD) || pressed_keys.contains(&KeyCode::ArrowRight) {
            x += 1.0;
        }
        if pressed_keys.contains(&KeyCode::KeyW) || pressed_keys.contains(&KeyCode::ArrowUp) {
            z += 1.0;
        }
        if pressed_keys.contains(&KeyCode::KeyS) || pressed_keys.contains(&KeyCode::ArrowDown) {
            z -= 1.0;
        }
        if pressed_keys.contains(&KeyCode::ShiftLeft) || pressed_keys.contains(&KeyCode::ShiftRight)
        {
            y -= 1.0;
        }
        if free_flight
            && (pressed_keys.contains(&KeyCode::Space) || pressed_keys.contains(&KeyCode::KeyE))
        {
            y += 1.0;
        }

        if x == 0.0 && y == 0.0 && z == 0.0 {
            return [0.0, 0.0, 0.0];
        }

        let mut horiz = [x, z];
        let horiz_len = (horiz[0] * horiz[0] + horiz[1] * horiz[1]).sqrt();
        if horiz_len > 0.0 {
            horiz[0] /= horiz_len;
            horiz[1] /= horiz_len;
        }

        let mut direction = [horiz[0], y, horiz[1]];

        if let Some(yaw) = camera_yaw {
            let sin_yaw = yaw.sin();
            let cos_yaw = yaw.cos();

            let forward = [sin_yaw, 0.0, -cos_yaw];
            let right = [cos_yaw, 0.0, sin_yaw];

            let world_dir = [
                forward[0] * horiz[1] + right[0] * horiz[0],
                direction[1],
                forward[2] * horiz[1] + right[2] * horiz[0],
            ];

            direction = world_dir;
        }

        let len = (direction[0] * direction[0]
            + direction[1] * direction[1]
            + direction[2] * direction[2])
            .sqrt();
        if len > 0.0 {
            [direction[0] / len, direction[1] / len, direction[2] / len]
        } else {
            [0.0, 0.0, 0.0]
        }
    }
}
