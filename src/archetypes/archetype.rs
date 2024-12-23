use crate::components::{Position, Name};

#[derive(Debug)]
pub struct Archetype {
    pub entity_ids: Vec<u32>,
    pub positions: Vec<Position>,
    pub names: Vec<Name>,
}

impl Archetype {
    pub fn new() -> Self {
        Self {
            entity_ids: Vec::new(),
            positions: Vec::new(),
            names: Vec::new(),
        }
    }

    pub fn add_entity(&mut self, id: u32, position: Position, name: Name) {
        self.entity_ids.push(id);
        self.positions.push(position);
        self.names.push(name);
    }
}

