use crate::ecs::ECS;
use crate::rendering::Renderer;

pub struct RenderPrepSystem;

impl RenderPrepSystem {
    pub fn update(renderer: &mut Renderer, ecs: &ECS, camera_entity: Option<u32>) {
        renderer.update_scene(ecs);
        renderer.update_lighting(ecs);
        if let Some(id) = camera_entity {
            if let Some(position) = ecs.find_entity_components(id).map(|(position, _)| position) {
                if let Some(camera) = ecs.camera_component(id) {
                    renderer.update_camera(position, camera);
                }
            }
        }
    }
}
