use crate::archetypes::Archetype;
use crate::components::{Position, Name};
use crate::ecs::entity_manager::EntityManager;
use std::collections::HashMap;
use log::debug;

pub struct ECS {
    pub archetypes: Vec<Archetype>,
    pub entity_to_location: HashMap<u32, (usize, usize)>,
    pub entity_manager: EntityManager,
}

impl ECS {
    pub fn new() -> Self {
        Self {
            archetypes: Vec::new(),
            entity_to_location: HashMap::new(),
            entity_manager: EntityManager::new(),
        }
    }

    pub fn add_entity(
        &mut self,
        position: Position,
        name: Name,
    ) {
        let id = self.entity_manager.create_entity();
        if self.archetypes.is_empty() {
            self.archetypes.push(Archetype::new());
        }
        let archetype_index = 0; // Examp
        let archetype = &mut self.archetypes[archetype_index];
        let index_within_archetype = archetype.entity_ids.len();

        // Add entity data
        archetype.add_entity(id, position,name);
        self.entity_to_location.insert(id, (archetype_index, index_within_archetype));
        debug!("Entity {} created. Current entity count: {}", id, self.entity_to_location.len()); 
    }

    pub fn find_entity(&self, id: u32) -> Option<&Archetype> {
        if let Some(&(archetype_index, _)) = self.entity_to_location.get(&id) {
            self.archetypes.get(archetype_index)
        } else {
            None
        }
    }

    pub fn find_entity_components(
        &self,
        id: u32,
    ) -> Option<(&Position, &Name)> {
        if let Some(&(archetype_index, index_within_archetype)) = self.entity_to_location.get(&id) {
            let archetype = &self.archetypes[archetype_index];
            Some((
                &archetype.positions[index_within_archetype],
                &archetype.names[index_within_archetype],
            ))
        } else {
            None
        }
    }

    pub fn remove_entity(&mut self, id:u32) {
                if let Some((archetype_index, index_within_archetype)) = self.entity_to_location.remove(&id) {
            let archetype = &mut self.archetypes[archetype_index];
            
            // Remove components associated with this entity
            archetype.entity_ids.swap_remove(index_within_archetype);
            archetype.positions.swap_remove(index_within_archetype);
            archetype.names.swap_remove(index_within_archetype);
            // Recycle the ID
            self.entity_manager.destroy_entity(id);
            debug!("Entity {} deleted. Current entity count: {}", id, self.entity_to_location.len());
        }
    }

    }

