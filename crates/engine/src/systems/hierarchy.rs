use std::collections::HashSet;

use log::{debug, warn};

use crate::components::Orientation;
use crate::ecs::ECS;

pub struct HierarchySystem;

impl HierarchySystem {
    pub fn update(ecs: &mut ECS) {
        let mut resolved = HashSet::new();
        let mut visiting = HashSet::new();
        let entities_with_hierarchy = Self::collect_entities_with_hierarchy(ecs);
        for entity_id in entities_with_hierarchy {
            Self::resolve_entity(ecs, entity_id, &mut resolved, &mut visiting);
        }
    }

    fn collect_entities_with_hierarchy(ecs: &ECS) -> Vec<u32> {
        let mut entities = Vec::new();
        for archetype in &ecs.archetypes {
            if archetype.hierarchies.is_none() {
                continue;
            }
            entities.extend(archetype.entity_ids.iter().copied());
        }
        entities
    }

    fn resolve_entity(
        ecs: &mut ECS,
        entity_id: u32,
        resolved: &mut HashSet<u32>,
        visiting: &mut HashSet<u32>,
    ) {
        if resolved.contains(&entity_id) {
            return;
        }
        if !visiting.insert(entity_id) {
            warn!(
                "Hierarchy cycle detected for entity {}; detaching from parent",
                entity_id
            );
            ecs.remove_hierarchy_component(entity_id);
            resolved.insert(entity_id);
            return;
        }

        let hierarchy = match ecs.hierarchy_component(entity_id) {
            Some(component) => *component,
            None => {
                visiting.remove(&entity_id);
                resolved.insert(entity_id);
                return;
            }
        };

        if hierarchy.parent == entity_id {
            warn!(
                "Entity {} cannot be its own parent; detaching hierarchy component",
                entity_id
            );
            ecs.remove_hierarchy_component(entity_id);
            visiting.remove(&entity_id);
            resolved.insert(entity_id);
            return;
        }

        if ecs.hierarchy_component(hierarchy.parent).is_some() {
            Self::resolve_entity(ecs, hierarchy.parent, resolved, visiting);
        }

        let (parent_position, mut parent_orientation) =
            match ecs.find_entity_components(hierarchy.parent) {
                Some((position, orientation, _)) => (*position, *orientation),
                None => {
                    warn!(
                        "Parent entity {} missing for child {}; detaching hierarchy component",
                        hierarchy.parent, entity_id
                    );
                    ecs.remove_hierarchy_component(entity_id);
                    visiting.remove(&entity_id);
                    resolved.insert(entity_id);
                    return;
                }
            };
        if let Some(camera) = ecs.camera_component(hierarchy.parent) {
            parent_orientation = Orientation::from_yaw_pitch_roll(-camera.yaw, camera.pitch, 0.0);
        }

        let (world_position, world_orientation) =
            hierarchy.compose_with_parent(parent_position, parent_orientation);

        if let Some(position) = ecs.position_component_mut(entity_id) {
            *position = world_position;
        }
        if let Some(orientation) = ecs.orientation_component_mut(entity_id) {
            *orientation = world_orientation;
        }
        if let Some((_, _, name)) = ecs.find_entity_components(entity_id) {
            if name.0 == "PlayerGun" {
                debug!("Gun orientation updated: {:?}", world_orientation);
            }
        }

        visiting.remove(&entity_id);
        resolved.insert(entity_id);
    }
}
