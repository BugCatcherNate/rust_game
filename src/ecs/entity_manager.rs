use std::collections::HashSet;

pub struct EntityManager {
    pub next_entity_id: u32,
    pub recycled_ids: HashSet<u32>,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            next_entity_id: 0,
            recycled_ids: HashSet::new(),
        }
    }

    pub fn create_entity(&mut self) -> u32 {
        if let Some(&id) = self.recycled_ids.iter().next() {
            self.recycled_ids.remove(&id);
            id
        } else {
            let id = self.next_entity_id;
            self.next_entity_id += 1;
            id
        }
    }

    pub fn destroy_entity(&mut self, id: u32) {
        self.recycled_ids.insert(id);
    }
}

