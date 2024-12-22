mod modules;
mod ecs;
use ecs::position_component::PositionComponent;
use ecs::name_component::NameComponent;
use ecs::ecs::ECS;
use ecs::entity::Entity;
use std::thread;
use std::time::Duration;

fn main() {
    env_logger::init();
    modules::core::initialize();
    let mut ecs = ECS::new();
    let mut player = Entity::new(1);
    let mut player_position = PositionComponent::new([0.0, 0.0, 0.0]);
    let mut player_name = NameComponent::new("nathan".to_string());
    player.add_component(player_position);
    player.add_component(player_name);
    ecs.add_entity(player);
    while true {
        if let Some(entity) = ecs.get_entity(1) {
        println!("Player: {} at position {:?}",entity.get_component::<NameComponent>().unwrap().name, entity.get_component::<PositionComponent>().unwrap().position);
        }
        thread::sleep(Duration::from_millis(500));
    }
    modules::core::shutdown();
}
