use rust_game::ecs::ECS;
use rust_game::components::{Position, Name};

#[test]
fn test_add_entity() {
    let mut ecs = ECS::new();

    // Add an entity
    let position = Position { x: 1.0, y: 2.0 };
    let name = Name("Test Entity".to_string());
    ecs.add_entity(position.clone(), name.clone());

    // Verify that the entity was added
    assert_eq!(ecs.entity_to_location.len(), 1);
    let id = ecs.entity_manager.next_entity_id - 1;
    assert!(ecs.entity_to_location.contains_key(&id));

    // Check the entity's components
    let components = ecs.find_entity_components(id).unwrap();
    assert_eq!(components.0, &position);
    assert_eq!(components.1, &name);
}

#[test]
fn test_find_entity() {
    let mut ecs = ECS::new();

    // Add an entity
    let position = Position { x: 5.0, y: 10.0 };
    let name = Name("Finder".to_string());
    ecs.add_entity(position.clone(), name.clone());

    // Find the entity
    let id = ecs.entity_manager.next_entity_id - 1;
    let archetype = ecs.find_entity(id);

    assert!(archetype.is_some());
}

#[test]
fn test_find_entity_components() {
    let mut ecs = ECS::new();

    // Add an entity
    let position = Position { x: 3.0, y: 4.0 };
    let name = Name("Component Checker".to_string());
    ecs.add_entity(position.clone(), name.clone());

    // Find the components
    let id = ecs.entity_manager.next_entity_id - 1;
    let components = ecs.find_entity_components(id);

    assert!(components.is_some());
    let (pos, n) = components.unwrap();
    assert_eq!(pos, &position);
    assert_eq!(n, &name);
}

#[test]
fn test_remove_entity() {
    let mut ecs = ECS::new();

    // Add an entity
    let position = Position { x: 7.0, y: 8.0 };
    let name = Name("Removable".to_string());
    ecs.add_entity(position.clone(), name.clone());

    // Verify the entity exists in the archetype before removal
    let id = ecs.entity_manager.next_entity_id - 1;
    let location = ecs.entity_to_location.get(&id).unwrap();
    let (archetype_index, _) = *location;
    let archetype = &ecs.archetypes[archetype_index];

    assert!(archetype.entity_ids.contains(&id));

    // Remove the entity
    ecs.remove_entity(id);

    // Verify the entity is removed from the ECS
    assert_eq!(ecs.entity_to_location.len(), 0);
    assert!(ecs.find_entity(id).is_none());
    assert!(ecs.find_entity_components(id).is_none());

    // Verify the entity is removed from the Archetype
    let archetype = &ecs.archetypes[archetype_index];
    assert!(!archetype.entity_ids.contains(&id));
    assert_eq!(archetype.entity_ids.len(), archetype.positions.len());
    assert_eq!(archetype.entity_ids.len(), archetype.names.len());
}

#[test]
fn test_reuse_entity_id() {
    let mut ecs = ECS::new();

    // Add and remove an entity
    let position = Position { x: 2.0, y: 2.0 };
    let name = Name("Reusable".to_string());
    ecs.add_entity(position.clone(), name.clone());
    let id = ecs.entity_manager.next_entity_id - 1;
    ecs.remove_entity(id);

    // Add a new entity and check ID reuse
    let new_position = Position { x: 9.0, y: 9.0 };
    let new_name = Name("Reused".to_string());
    ecs.add_entity(new_position.clone(), new_name.clone());
    let new_id = ecs.entity_manager.next_entity_id - 1;

    assert_eq!(id, new_id);
    assert!(ecs.find_entity_components(new_id).is_some());
}

