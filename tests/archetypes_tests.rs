use rust_game::archetypes::Archetype;
use rust_game::components::model::{Model, Vertex};
use rust_game::components::{Name, Position};

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

    archetype.add_entity(entity_id, Some(position.clone()), Some(name.clone()), None);

    assert_eq!(archetype.entity_ids.len(), 1);
    assert_eq!(archetype.positions.len(), 1);
    assert_eq!(archetype.names.len(), 1);

    assert_eq!(archetype.entity_ids[0], entity_id);
    assert_eq!(archetype.positions[0].as_ref().unwrap(), &position);
    assert_eq!(archetype.names[0].as_ref().unwrap(), &name);
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
        archetype.add_entity(*id, Some(position.clone()), Some(name.clone()), None);
    }

    assert_eq!(archetype.entity_ids.len(), entities.len());
    assert_eq!(archetype.positions.len(), entities.len());
    assert_eq!(archetype.names.len(), entities.len());

    for (i, (id, position, name)) in entities.iter().enumerate() {
        assert_eq!(archetype.entity_ids[i], *id);
        assert_eq!(archetype.positions[i].as_ref().unwrap(), &position);
        assert_eq!(archetype.names[i].as_ref().unwrap(), &name);
    }
}

#[test]
fn test_model_creation() {
    // Create a model with no vertices
    let model = Model::new();
    assert!(model.vertices.is_empty());
}

#[test]
fn test_add_vertex() {
    let mut model = Model::new();

    let vertex = Vertex::new([1.0, 2.0, 3.0], [0.1, 0.1]);
    model.add_vertex(vertex.clone());

    assert_eq!(model.vertices.len(), 1);
    assert_eq!(model.vertices[0], vertex);
}

#[test]
fn test_add_multiple_vertices() {
    let mut model = Model::new();

    let vertices = vec![
        Vertex::new([1.0, 2.0, 3.0], [0.1, 0.1]),
        Vertex::new([4.0, 5.0, 6.0], [0.2, 0.2]),
    ];

    for vertex in &vertices {
        model.add_vertex(vertex.clone());
    }

    assert_eq!(model.vertices.len(), vertices.len());

    for (i, vertex) in vertices.iter().enumerate() {
        assert_eq!(model.vertices[i], *vertex);
    }
}

#[test]
fn test_clear_model() {
    let mut model = Model::new();

    let vertex = Vertex::new([1.0, 2.0, 3.0], [0.1, 0.1]);
    model.add_vertex(vertex.clone());

    assert_eq!(model.vertices.len(), 1);

    model.clear();

    assert!(model.vertices.is_empty());
}

#[test]
fn test_position_name_and_model() {
    let mut archetype = Archetype::new();

    let position = Position { x: 1.0, y: 2.0 };
    let name = Name("Test Entity".to_string());
    let entity_id = 42;

    // Assuming you have model with vertices
    let model = Model::new();
    let vertex = Vertex::new([1.0, 2.0, 3.0], [0.1, 0.1]);
    let mut model = model;
    model.add_vertex(vertex.clone());

    archetype.add_entity(
        entity_id,
        Some(position.clone()),
        Some(name.clone()),
        Some(model.clone()),
    );
    // Check that position, name, and model exist in archetype
    assert_eq!(archetype.entity_ids.len(), 1);
    assert_eq!(archetype.positions.len(), 1);
    assert_eq!(archetype.names.len(), 1);

    let pos = archetype.positions.get(0).unwrap();
    let n = archetype.names.get(0).unwrap();
    let m = archetype.models.get(0).unwrap(); // Assuming you add models to the archetype

    assert_eq!(pos.as_ref().unwrap(), &position);
    assert_eq!(n.as_ref().unwrap(), &name);
    assert_eq!(m.as_ref().unwrap().vertices.len(), 1); // Unwrap Model to access vertices
    assert_eq!(m.as_ref().unwrap().vertices[0], vertex);
}
