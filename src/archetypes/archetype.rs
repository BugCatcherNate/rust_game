use crate::components::{Position, Name, Model};

#[derive(Debug, Clone)]
pub struct Archetype {
    pub entity_ids: Vec<u32>,
    pub positions: Vec<Option<Position>>,
    pub names: Vec<Option<Name>>,
    pub models: Vec<Option<Model>>,
}

impl Archetype {
    pub fn new() -> Self {
        Self {
            entity_ids: Vec::new(),
            positions: Vec::new(),
            names: Vec::new(),
            models: Vec::new(),
        }
    }

    pub fn add_entity(
        &mut self,
        id: u32,
        position: Option<Position>,
        name: Option<Name>,
        model: Option<Model>,
    ) {
        self.entity_ids.push(id);
        self.positions.push(position);
        self.names.push(name);
        self.models.push(model);
    }
}

