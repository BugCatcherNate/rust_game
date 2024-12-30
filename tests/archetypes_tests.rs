use rust_game::archetypes::Archetype;
use rust_game::components::{Position, Name};
#[test]
fn test_new_archetype() {
    let archetype = Archetype::new();
    assert!(archetype.entity_ids.is_empty());
    assert!(archetype.positions.is_empty());
    assert!(archetype.names.is_empty());
}

#[test]
fn test_add_entity() {
    let mut archetype = Archetype::new();
    
    let position = Position { x: 1.0, y: 2.0 };
    let name = Name("Test Entity".to_string());
    let entity_id = 42;

    archetype.add_entity(entity_id, position.clone(), name.clone());

    assert_eq!(archetype.entity_ids.len(), 1);
    assert_eq!(archetype.positions.len(), 1);
    assert_eq!(archetype.names.len(), 1);

    assert_eq!(archetype.entity_ids[0], entity_id);
    assert_eq!(archetype.positions[0], position);
    assert_eq!(archetype.names[0], name);
}

#[test]
fn test_add_multiple_entities() {
    let mut archetype = Archetype::new();

    let entities = vec![
        (1, Position { x: 1.0, y: 1.0 }, Name("Entity1".to_string())),
        (2, Position { x: 2.0, y: 2.0 }, Name("Entity2".to_string())),
        (3, Position { x: 3.0, y: 3.0 }, Name("Entity3".to_string())),
    ];

    for (id, position, name) in &entities {
        archetype.add_entity(*id, position.clone(), name.clone());
    }

    assert_eq!(archetype.entity_ids.len(), entities.len());
    assert_eq!(archetype.positions.len(), entities.len());
    assert_eq!(archetype.names.len(), entities.len());

    for (i, (id, position, name)) in entities.iter().enumerate() {
        assert_eq!(archetype.entity_ids[i], *id);
        assert_eq!(archetype.positions[i], *position);
        assert_eq!(archetype.names[i], *name);
    }
}
