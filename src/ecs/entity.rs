use log::{debug};
use std::collections::HashMap;
use std::any::Any;

pub struct Entity {
    pub id: u32,
    components: HashMap<String, Box<dyn Any>>,
}

impl Entity {

   pub fn new(id: u32) -> Self {

        debug!("Entity {} created", id);
        Self {
            id,
            components: HashMap::new(),
        }
    }

       // Adds a component to the entity
    pub fn add_component<T: 'static>(&mut self, component: T) {
        let key = std::any::type_name::<T>().to_string();
        self.components.insert(key, Box::new(component));
    }

    // Retrieves a reference to a component if it exists
    pub fn get_component<T: 'static>(&self) -> Option<&T> {
        let key = std::any::type_name::<T>().to_string();
        self.components.get(&key).and_then(|boxed| boxed.downcast_ref::<T>())
    }

}
