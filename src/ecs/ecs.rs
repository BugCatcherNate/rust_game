use crate::ecs::entity::Entity;

pub struct ECS {
    entities: Vec<Entity>,
}

impl ECS {
    pub fn new() -> Self {

        ECS {
            entities: Vec::new(),
        }

    }


    
    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    pub fn get_entity(&self, id: u32) -> Option<&Entity> {
        self.entities.iter().find(|e| e.id == id)
    }

    pub fn get_entity_mut(&mut self, id: u32) -> Option<&mut Entity> {
        self.entities.iter_mut().find(|e| e.id == id)
    }




}
