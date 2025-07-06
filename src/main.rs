// mod modules;
// mod archetypes;
// mod components;
// mod systems;
// mod ecs;
// 
// use components::{Position, Name};
// use systems::systems::Movement;
// use systems::systems_manager::SystemsManager;
// use ecs::ECS;

// fn main() {
//     env_logger::init();
//     modules::core::initialize();
    //pollster::block_on(run());
//     let mut ecs = ECS::new();
//     let mut systems_manager = SystemsManager::new();
//     systems_manager.add_system(Box::new(Movement{}));
//     ecs.add_entity(Some(Position { x: 0.1, y: 0.2 }), Some(Name("Nathan".to_string())));
//     ecs.add_entity(Some(Position { x: 0.0, y: 0.0 }), Some(Name("Ronald".to_string())));
//     ecs.tag_manager.add_tag(0, "player");
//     ecs.tag_manager.add_tag(0, "camera");
//     ecs.tag_manager.add_tag(1, "player");
//     
//     systems_manager.update(&mut ecs.archetypes);
//     if let Some(entities) =  ecs.tag_manager.get_entities_with_tag("player") {
//         println!("Entities with player tag: {:?}", entities);
//     }
//     if let Some(entities) =  ecs.tag_manager.get_entities_with_tag("camera") {
//         println!("Entities with camera tag: {:?}", entities);
//     }
//     if let Some((position, name)) = ecs.find_entity_components(0) {
//         println!("After movement, entity {:?} is at position {:?}",name, position);
//     }
//    if let Some((position, name)) = ecs.find_entity_components(1) {
//         println!("After movement, entity {:?} is at position {:?}",name, position);
//     }
//     if let Some(entity) = ecs.find_entity(1) {
//     println!("Entity 1 exists in archetype: {:?}", entity);
// } else {
//     println!("Entity 1 not found!");
// }
// 
// 
//     ecs.remove_entity(0);
// 
//     modules::core::shutdown();
//}
//
//
mod graphics;

use graphics::vulkano::vulkano_instance::{create_vulkano_bundle};

fn main() {

    let vulkano_bundle = create_vulkano_bundle();

        println!(
        "API Version: {:?}", vulkano_bundle.instance.api_version());
}

