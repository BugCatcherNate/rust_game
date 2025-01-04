mod modules;
mod archetypes;
mod components;
mod systems;
mod ecs;
mod graphics;

use components::{Position, Name};
use systems::MovementSystem;
use ecs::ECS;
use graphics::game_window::run;
use pollster;

fn main() {
    env_logger::init();
    modules::core::initialize();
    pollster::block_on(run());
    let mut ecs = ECS::new();
    ecs.add_entity(Position { x: 0.0, y: 0.0 }, Name("Nathan".to_string()));
    ecs.add_entity(Position { x: 0.0, y: 0.0 }, Name("Ronald".to_string()));
    ecs.tag_manager.add_tag(0, "player");
    ecs.tag_manager.add_tag(0, "camera");
    ecs.tag_manager.add_tag(1, "player");

    // Update systems
    for archetype in &mut ecs.archetypes {
        MovementSystem::update(archetype);
    }
    if let Some(entities) =  ecs.tag_manager.get_entities_with_tag("player") {
        println!("Entities with player tag: {:?}", entities);
    }
    if let Some(entities) =  ecs.tag_manager.get_entities_with_tag("camera") {
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
