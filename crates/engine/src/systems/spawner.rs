use std::collections::HashMap;

use log::warn;

use crate::components::{HierarchyComponent, Orientation, Position};
use crate::ecs::ECS;
use crate::scene::{EntityDefinition, SceneDefinition};

pub struct SpawnerSystem;

impl SpawnerSystem {
    pub fn spawn_on_load(scene: &SceneDefinition, ecs: &mut ECS) {
        let definition_by_name = build_definition_by_name(scene);
        let children_by_parent = build_children_by_parent(scene);

        for spawner in scene
            .entities
            .iter()
            .filter(|entity| entity.components.spawner.is_some())
        {
            let Some(spawner_cfg) = spawner.components.spawner.as_ref() else {
                continue;
            };
            if !spawner_cfg.spawn_on_load {
                continue;
            }
            let Some(template_root) = definition_by_name.get(&spawner_cfg.template) else {
                warn!(
                    "Spawner '{}' references missing template '{}'",
                    spawner.name, spawner_cfg.template
                );
                continue;
            };

            spawn_template_recursive(
                ecs,
                template_root,
                spawner.position,
                spawner.orientation,
                None,
                &children_by_parent,
                &spawner.name,
            );
        }
    }

    pub fn update(scene: &SceneDefinition, ecs: &mut ECS) {
        const SPAWNER_DT: f32 = 1.0 / 60.0;
        let definition_by_name = build_definition_by_name(scene);
        let children_by_parent = build_children_by_parent(scene);
        let mut requests = Vec::new();

        for archetype in &mut ecs.archetypes {
            let Some(spawners) = archetype.spawners.as_mut() else {
                continue;
            };
            for index in 0..spawners.len() {
                let spawner = &mut spawners[index];
                if spawner.interval <= 0.0 {
                    continue;
                }
                spawner.timer += SPAWNER_DT;
                while spawner.timer >= spawner.interval {
                    spawner.timer -= spawner.interval;
                    requests.push(SpawnRequest {
                        template: spawner.template.clone(),
                        position: archetype.positions[index],
                        orientation: archetype.orientations[index],
                        spawner_name: archetype.names[index].0.clone(),
                    });
                }
            }
        }

        for request in requests {
            let Some(template_root) = definition_by_name.get(&request.template) else {
                warn!(
                    "Spawner '{}' references missing template '{}'",
                    request.spawner_name, request.template
                );
                continue;
            };
            spawn_template_recursive(
                ecs,
                template_root,
                request.position,
                request.orientation,
                None,
                &children_by_parent,
                &request.spawner_name,
            );
        }
    }
}

fn build_definition_by_name(scene: &SceneDefinition) -> HashMap<String, &EntityDefinition> {
    let mut definition_by_name = HashMap::new();
    for entity in &scene.entities {
        definition_by_name.insert(entity.name.clone(), entity);
    }
    definition_by_name
}

fn build_children_by_parent(
    scene: &SceneDefinition,
) -> HashMap<String, Vec<&EntityDefinition>> {
    let mut children_by_parent: HashMap<String, Vec<&EntityDefinition>> = HashMap::new();
    for entity in &scene.entities {
        if let Some(parent) = entity.parent.as_ref() {
            children_by_parent
                .entry(parent.clone())
                .or_default()
                .push(entity);
        }
    }
    children_by_parent
}

struct SpawnRequest {
    template: String,
    position: Position,
    orientation: Orientation,
    spawner_name: String,
}

fn spawn_template_recursive(
    ecs: &mut ECS,
    template_def: &EntityDefinition,
    position: Position,
    orientation: Orientation,
    parent: Option<(u32, Position, Orientation)>,
    children_by_parent: &HashMap<String, Vec<&EntityDefinition>>,
    spawner_name: &str,
) {
    let spawned_name = unique_entity_name(ecs, &template_def.name, spawner_name);
    let mut spawned_def = template_def.clone();
    spawned_def.name = spawned_name;
    spawned_def.position = position;
    spawned_def.orientation = orientation;
    spawned_def.parent = None;
    spawned_def.template = false;

    let entity_id = spawn_entity_from_definition(ecs, &spawned_def);
    if let Some((parent_id, parent_position, parent_orientation)) = parent {
        let hierarchy = HierarchyComponent::from_world_transforms(
            parent_id,
            parent_position,
            parent_orientation,
            position,
            orientation,
        );
        ecs.add_hierarchy_component(entity_id, hierarchy);
    }

    let Some(children) = children_by_parent.get(&template_def.name) else {
        return;
    };
    for child_def in children {
        let hierarchy = HierarchyComponent::from_world_transforms(
            0,
            template_def.position,
            template_def.orientation,
            child_def.position,
            child_def.orientation,
        );
        let (child_position, child_orientation) =
            hierarchy.compose_with_parent(position, orientation);
        spawn_template_recursive(
            ecs,
            child_def,
            child_position,
            child_orientation,
            Some((entity_id, position, orientation)),
            children_by_parent,
            spawner_name,
        );
    }
}

fn unique_entity_name(ecs: &ECS, desired: &str, spawner_name: &str) -> String {
    if ecs.find_entity_id_by_name(desired).is_none() {
        return desired.to_string();
    }
    let base = format!("{}_{}", spawner_name, desired);
    if ecs.find_entity_id_by_name(&base).is_none() {
        return base;
    }
    let mut index = 1;
    loop {
        let candidate = format!("{}_{}", base, index);
        if ecs.find_entity_id_by_name(&candidate).is_none() {
            return candidate;
        }
        index += 1;
    }
}

fn spawn_entity_from_definition(ecs: &mut ECS, entity: &EntityDefinition) -> u32 {
    use crate::scene::spawn_entity_from_definition as spawn;
    spawn(ecs, entity)
}
