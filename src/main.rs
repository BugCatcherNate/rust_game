mod modules;
mod archetypes;
mod components;
mod systems;
mod ecs;

use components::{Position, Name};
use systems::MovementSystem;
use ecs::ECS;

fn main() {
    env_logger::init();
    modules::core::initialize();
    let mut ecs = ECS::new();
   ecs.add_entity(1, Position { x: 0.0, y: 0.0 }, Name("Nathan".to_string()));
   ecs.add_entity(2, Position { x: 0.0, y: 0.0 }, Name("Ronald".to_string()));

    // Update systems
    for archetype in &mut ecs.archetypes {
        MovementSystem::update(archetype);
    }
    if let Some((position, name)) = ecs.find_entity_components(1) {
        println!("After movement, entity {:?} is at position {:?}",name, position);
    }
   if let Some((position, name)) = ecs.find_entity_components(2) {
        println!("After movement, entity {:?} is at position {:?}",name, position);
    }

    modules::core::shutdown();
}
