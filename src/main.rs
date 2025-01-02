mod modules;
mod archetypes;
mod components;
mod systems;
mod ecs;

use components::{Position, Name};
use systems::MovementSystem;
use ecs::ECS;
use ecs::tag_manager::TagManager;

fn main() {
    env_logger::init();
    modules::core::initialize();
    let mut ecs = ECS::new();
    let mut tm = TagManager::new();
    ecs.add_entity(Position { x: 0.0, y: 0.0 }, Name("Nathan".to_string()));
    ecs.add_entity(Position { x: 0.0, y: 0.0 }, Name("Ronald".to_string()));
    tm.add_tag(0, "player");
    tm.add_tag(0, "camera");
    tm.add_tag(1, "player");

    // Update systems
    for archetype in &mut ecs.archetypes {
        MovementSystem::update(archetype);
    }
    if let Some(entities) =  tm.get_entities_with_tag("player") {
        println!("Entities with player tag: {:?}", entities);
    }
    if let Some(entities) =  tm.get_entities_with_tag("camera") {
        println!("Entities with camera tag: {:?}", entities);
    }
    if let Some((position, name)) = ecs.find_entity_components(1) {
        println!("After movement, entity {:?} is at position {:?}",name, position);
    }
   if let Some((position, name)) = ecs.find_entity_components(2) {
        println!("After movement, entity {:?} is at position {:?}",name, position);
    }


    ecs.remove_entity(0);

    modules::core::shutdown();
}
