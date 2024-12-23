use crate::archetypes::Archetype;
use crate::components::{Position, Name};
use std::collections::HashMap;

pub struct ECS {
    pub archetypes: Vec<Archetype>,
    pub entity_to_location: HashMap<u32, (usize, usize)>,
}

impl ECS {
    pub fn new() -> Self {
        Self {
            archetypes: Vec::new(),
            entity_to_location: HashMap::new(),
        }
    }

    pub fn add_entity(
        &mut self,
        id: u32,
        position: Position,
        name: Name,
    ) {
        if self.archetypes.is_empty() {
            self.archetypes.push(Archetype::new());
        }
        let archetype_index = 0; // Examp
        let archetype = &mut self.archetypes[archetype_index];
        let index_within_archetype = archetype.entity_ids.len();

        // Add entity data
        archetype.add_entity(id, position,name);
        self.entity_to_location.insert(id, (archetype_index, index_within_archetype));
 
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
}

