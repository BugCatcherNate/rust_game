use rust_game::components::{
    CameraComponent, InputComponent, LightComponent, ModelComponent, Name, Position,
    RenderComponent, ScriptComponent, TerrainComponent,
};
use rust_game::ecs::ECS;

#[test]
fn test_add_entity() {
    let mut ecs = ECS::new();

    // Add an entity
    let position = Position {
        x: 1.0,
        y: 2.0,
        z: 0.0,
    };
    let name = Name("Test Entity".to_string());
    let id = ecs.add_entity(position.clone(), name.clone());

    // Verify that the entity was added
    assert_eq!(ecs.entity_to_location.len(), 1);
    assert!(ecs.entity_to_location.contains_key(&id));

    // Check the entity's components
    let components = ecs.find_entity_components(id).unwrap();
    assert_eq!(components.0, &position);
    assert_eq!(components.1, &name);
}

#[test]
fn test_add_render_component() {
    let mut ecs = ECS::new();

    let position = Position {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let name = Name("Renderable".to_string());
    let id = ecs.add_entity(position, name);

    let render_component = RenderComponent::new([1.0, 0.0, 0.0], 1.0);
    ecs.add_render_component(id, render_component.clone());

    let (archetype_index, index_within_archetype) = ecs.entity_to_location[&id];
    let archetype = &ecs.archetypes[archetype_index];
    let column = archetype
        .renderables
        .as_ref()
        .expect("render column missing");
    assert_eq!(column[index_within_archetype], render_component);
}

#[test]
fn test_add_input_component() {
    let mut ecs = ECS::new();

    let position = Position {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let name = Name("Controllable".to_string());
    let id = ecs.add_entity(position, name);

    let mut input_component = InputComponent::new(0.2);
    input_component.set_direction([0.0, 0.0, 1.0]);
    ecs.add_input_component(id, input_component.clone());

    let (archetype_index, index_within_archetype) = ecs.entity_to_location[&id];
    let archetype = &ecs.archetypes[archetype_index];
    let column = archetype.inputs.as_ref().expect("input column missing");
    assert_eq!(column[index_within_archetype], input_component);
}

#[test]
fn test_add_model_component() {
    let mut ecs = ECS::new();

    let position = Position {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let name = Name("Model Holder".to_string());
    let id = ecs.add_entity(position, name);

    let model_component = ModelComponent::new("assets/cube.obj");
    ecs.add_model_component(id, model_component.clone());

    let (archetype_index, index_within_archetype) = ecs.entity_to_location[&id];
    let archetype = &ecs.archetypes[archetype_index];
    let column = archetype.models.as_ref().expect("model column missing");
    assert_eq!(column[index_within_archetype], model_component);
}

#[test]
fn test_add_camera_component() {
    let mut ecs = ECS::new();

    let position = Position {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let name = Name("Camera".to_string());
    let id = ecs.add_entity(position, name);

    let camera_component = CameraComponent::new(0.0, 0.0);
    ecs.add_camera_component(id, camera_component.clone());

    let (archetype_index, index_within_archetype) = ecs.entity_to_location[&id];
    let archetype = &ecs.archetypes[archetype_index];
    let column = archetype.cameras.as_ref().expect("camera column missing");
    assert_eq!(column[index_within_archetype], camera_component);
}

#[test]
fn test_add_terrain_component() {
    let mut ecs = ECS::new();

    let position = Position {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let name = Name("Terrain Holder".to_string());
    let id = ecs.add_entity(position, name);

    let terrain_component = TerrainComponent::new(
        12.0,
        0.3,
        [0.2, 0.4, 0.2],
        None,
        "assets/terrain_plane.obj".to_string(),
    );
    ecs.add_terrain_component(id, terrain_component.clone());

    let (archetype_index, index_within_archetype) = ecs.entity_to_location[&id];
    let archetype = &ecs.archetypes[archetype_index];
    let column = archetype.terrains.as_ref().expect("terrain column missing");
    assert_eq!(column[index_within_archetype], terrain_component);
}

#[test]
fn test_add_script_component() {
    let mut ecs = ECS::new();

    let position = Position {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };
    let name = Name("Scripted".to_string());
    let id = ecs.add_entity(position, name);

    let script = ScriptComponent::new("test_behavior", 1.0);
    ecs.add_script_component(id, script.clone());

    let (archetype_index, index_within_archetype) = ecs.entity_to_location[&id];
    let archetype = &ecs.archetypes[archetype_index];
    let column = archetype.scripts.as_ref().expect("script column missing");
    assert_eq!(column[index_within_archetype], script);
}

#[test]
fn test_remove_render_component() {
    let mut ecs = ECS::new();

    let position = Position {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let name = Name("Renderable".to_string());
    let id = ecs.add_entity(position, name);

    ecs.add_render_component(id, RenderComponent::new([1.0, 0.0, 0.0], 1.0));
    let (before_index, before_slot) = ecs.entity_to_location[&id];
    assert!(ecs.archetypes[before_index].renderables.is_some());
    assert!(
        ecs.archetypes[before_index].renderables.as_ref().unwrap()[before_slot].color
            == [1.0, 0.0, 0.0]
    );

    ecs.remove_render_component(id);
    let (after_index, after_slot) = ecs.entity_to_location[&id];
    assert_ne!(before_index, after_index);
    assert!(ecs.archetypes[after_index].renderables.is_none());
    assert_eq!(ecs.archetypes[after_index].positions[after_slot].x, 0.0);
}

#[test]
fn test_remove_one_of_multiple_components_preserves_others() {
    let mut ecs = ECS::new();

    let id = ecs.add_entity(
        Position {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        Name("Complex".into()),
    );

    ecs.add_render_component(id, RenderComponent::new([0.5, 0.5, 0.5], 1.0));
    ecs.add_model_component(id, ModelComponent::new("assets/cube.obj"));
    let render_before = {
        let (index, slot) = ecs.entity_to_location[&id];
        ecs.archetypes[index].renderables.as_ref().unwrap()[slot].clone()
    };

    ecs.remove_model_component(id);

    let (after_index, after_slot) = ecs.entity_to_location[&id];
    assert!(ecs.archetypes[after_index].models.is_none());
    let render_column = ecs.archetypes[after_index]
        .renderables
        .as_ref()
        .expect("render column missing after removal");
    assert_eq!(render_column[after_slot], render_before);
}
#[test]
fn add_tag_using_ecs() {
    let mut ecs = ECS::new();
    let position = Position {
        x: 5.0,
        y: 10.0,
        z: 0.0,
    };
    let name = Name("Finder".to_string());
    let id = ecs.add_entity(position.clone(), name.clone());
    ecs.tag_manager.add_tag(id, "player");
    let entities = ecs.tag_manager.get_entities_with_tag("player").unwrap();
    assert!(entities.contains(&id));
}
#[test]
fn test_find_entity() {
    let mut ecs = ECS::new();

    // Add an entity
    let position = Position {
        x: 5.0,
        y: 10.0,
        z: 0.0,
    };
    let name = Name("Finder".to_string());
    let id = ecs.add_entity(position.clone(), name.clone());

    // Find the entity
    let archetype = ecs.find_entity(id);

    assert!(archetype.is_some());
}

#[test]
fn test_find_entity_components() {
    let mut ecs = ECS::new();

    // Add an entity
    let position = Position {
        x: 3.0,
        y: 4.0,
        z: 0.0,
    };
    let name = Name("Component Checker".to_string());
    let id = ecs.add_entity(position.clone(), name.clone());

    // Find the components
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
    let position = Position {
        x: 7.0,
        y: 8.0,
        z: 0.0,
    };
    let name = Name("Removable".to_string());
    let id = ecs.add_entity(position.clone(), name.clone());

    // Verify the entity exists in the archetype before removal
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
    let len = archetype.entity_ids.len();
    assert_eq!(len, archetype.positions.len());
    assert_eq!(len, archetype.names.len());
    let optional_lengths = [
        archetype.renderables.as_ref().map(|c| c.len()),
        archetype.inputs.as_ref().map(|c| c.len()),
        archetype.models.as_ref().map(|c| c.len()),
        archetype.cameras.as_ref().map(|c| c.len()),
        archetype.lights.as_ref().map(|c| c.len()),
        archetype.textures.as_ref().map(|c| c.len()),
        archetype.terrains.as_ref().map(|c| c.len()),
        archetype.scripts.as_ref().map(|c| c.len()),
    ];
    for column_len in optional_lengths {
        if let Some(column_len) = column_len {
            assert_eq!(len, column_len);
        }
    }
}

#[test]
fn test_reuse_entity_id() {
    let mut ecs = ECS::new();

    // Add and remove an entity
    let position = Position {
        x: 2.0,
        y: 2.0,
        z: 0.0,
    };
    let name = Name("Reusable".to_string());
    let id = ecs.add_entity(position.clone(), name.clone());
    ecs.remove_entity(id);

    // Add a new entity and check ID reuse
    let new_position = Position {
        x: 9.0,
        y: 9.0,
        z: 0.0,
    };
    let new_name = Name("Reused".to_string());
    let new_id = ecs.add_entity(new_position.clone(), new_name.clone());

    assert_eq!(id, new_id);
    assert!(ecs.find_entity_components(new_id).is_some());
}
#[test]
fn light_components_iterates_lights() {
    let mut ecs = ECS::new();
    let entity = ecs.add_entity(
        Position {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        Name("Light".into()),
    );
    ecs.add_light_component(
        entity,
        LightComponent::directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0], 1.0),
    );

    let lights: Vec<_> = ecs.light_components().collect();
    assert_eq!(lights.len(), 1);
    let (pos, light) = lights[0];
    assert_eq!(pos.x, 0.0);
    assert_eq!(light.intensity, 1.0);
}

#[test]
fn terrain_components_iterates_terrains() {
    let mut ecs = ECS::new();
    let entity = ecs.add_entity(
        Position {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        Name("Ground".into()),
    );
    ecs.add_terrain_component(
        entity,
        TerrainComponent::new(
            20.0,
            0.4,
            [0.3, 0.55, 0.2],
            Some("assets/textures/blue.png".into()),
            "assets/terrain_plane.obj".into(),
        ),
    );

    let mut iter = ecs.terrain_components();
    let terrain = iter.next().expect("missing terrain component");
    assert_eq!(terrain.size, 20.0);
    assert!(iter.next().is_none());
}
