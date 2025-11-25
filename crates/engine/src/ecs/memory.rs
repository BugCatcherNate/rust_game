use std::mem;

use crate::components::{
    CameraComponent, InputComponent, LightComponent, ModelComponent, Name, PhysicsComponent,
    Position, RenderComponent, ScriptComponent, TerrainComponent, TextureComponent,
};

use super::ECS;

#[derive(Debug, Clone)]
pub struct ComponentMemoryUsage {
    pub label: &'static str,
    pub estimated_bytes: usize,
}

#[derive(Debug, Clone)]
pub struct EntityMemoryUsage {
    pub id: u32,
    pub name: String,
    pub estimated_bytes: usize,
    pub components: Vec<ComponentMemoryUsage>,
}

impl ECS {
    pub fn entity_memory_usage(&self) -> Vec<EntityMemoryUsage> {
        let mut reports = Vec::new();
        for archetype in &self.archetypes {
            for index in 0..archetype.entity_ids.len() {
                let mut components = Vec::new();
                let mut total = 0usize;

                total += push_usage::<Position>("Position", true, &mut components);
                total += push_usage::<Name>("Name", true, &mut components);

                total += push_usage::<RenderComponent>(
                    "Render",
                    archetype
                        .renderables
                        .as_ref()
                        .and_then(|column| column.get(index))
                        .is_some(),
                    &mut components,
                );
                total += push_usage::<InputComponent>(
                    "Input",
                    archetype
                        .inputs
                        .as_ref()
                        .and_then(|column| column.get(index))
                        .is_some(),
                    &mut components,
                );
                total += push_usage::<ModelComponent>(
                    "Model",
                    archetype
                        .models
                        .as_ref()
                        .and_then(|column| column.get(index))
                        .is_some(),
                    &mut components,
                );
                total += push_usage::<CameraComponent>(
                    "Camera",
                    archetype
                        .cameras
                        .as_ref()
                        .and_then(|column| column.get(index))
                        .is_some(),
                    &mut components,
                );
                total += push_usage::<LightComponent>(
                    "Light",
                    archetype
                        .lights
                        .as_ref()
                        .and_then(|column| column.get(index))
                        .is_some(),
                    &mut components,
                );
                total += push_usage::<TextureComponent>(
                    "Texture",
                    archetype
                        .textures
                        .as_ref()
                        .and_then(|column| column.get(index))
                        .is_some(),
                    &mut components,
                );
                total += push_usage::<TerrainComponent>(
                    "Terrain",
                    archetype
                        .terrains
                        .as_ref()
                        .and_then(|column| column.get(index))
                        .is_some(),
                    &mut components,
                );
                total += push_usage::<ScriptComponent>(
                    "Script",
                    archetype
                        .scripts
                        .as_ref()
                        .and_then(|column| column.get(index))
                        .is_some(),
                    &mut components,
                );
                total += push_usage::<PhysicsComponent>(
                    "Physics",
                    archetype
                        .physics
                        .as_ref()
                        .and_then(|column| column.get(index))
                        .is_some(),
                    &mut components,
                );

                reports.push(EntityMemoryUsage {
                    id: archetype.entity_ids[index],
                    name: archetype.names[index].0.clone(),
                    estimated_bytes: total,
                    components,
                });
            }
        }
        reports
    }

    pub fn total_memory_usage(&self) -> usize {
        self.entity_memory_usage()
            .into_iter()
            .map(|report| report.estimated_bytes)
            .sum()
    }
}

fn push_usage<T>(
    label: &'static str,
    present: bool,
    components: &mut Vec<ComponentMemoryUsage>,
) -> usize {
    if present {
        let size = mem::size_of::<T>();
        components.push(ComponentMemoryUsage {
            label,
            estimated_bytes: size,
        });
        size
    } else {
        0
    }
}
