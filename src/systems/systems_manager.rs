use crate::systems::systems::System; 
use crate::archetypes::archetype::Archetype;
use std::any::Any;

pub struct SystemsManager {
    systems: Vec<Box<dyn System>>,
}

impl SystemsManager {
    pub fn new()-> Self {
        SystemsManager {
            systems: Vec::new(),
        }
    }

    pub fn add_system(&mut self, system: Box<dyn System>){
        self.systems.push(system);

    }

    pub fn remove_system<T: Any>(&mut self) {
        if let Some(system) = self.systems.iter().position(|system| system.as_any().is::<T>()){
            self.systems.remove(system);
        }
    }

    pub fn update(&self, archetypes: &mut Vec<Archetype>) {
        for system in &self.systems {
            for archetype in archetypes.iter_mut(){
                system.update(archetype);
        }
    }
    } 
}





