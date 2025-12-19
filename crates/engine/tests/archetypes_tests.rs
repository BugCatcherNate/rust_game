use rust_game::archetypes::{Archetype, EntityComponents};
use rust_game::components::{
    CameraComponent, InputComponent, LightComponent, ModelComponent, Name, Orientation, Position,
    RenderComponent, ScriptComponent, TerrainComponent, TextureComponent,
};
use rust_game::ecs::{ComponentKind, ComponentSignature};

fn signature_with(kinds: &[ComponentKind]) -> ComponentSignature {
    kinds
        .iter()
        .copied()
        .fold(ComponentSignature::empty(), |sig, kind| sig.with(kind))
}

fn make_bundle(id: u32, position: Position, name: Name) -> EntityComponents {
    EntityComponents::base(id, position, Orientation::identity(), name)
}

#[test]
fn empty_signature_archetype_has_no_optional_columns() {
    let archetype = Archetype::new(ComponentSignature::empty());
    assert!(archetype.entity_ids.is_empty());
    assert!(archetype.positions.is_empty());
    assert!(archetype.names.is_empty());
    assert!(archetype.renderables.is_none());
    assert!(archetype.inputs.is_none());
    assert!(archetype.models.is_none());
    assert!(archetype.cameras.is_none());
    assert!(archetype.lights.is_none());
    assert!(archetype.textures.is_none());
    assert!(archetype.terrains.is_none());
    assert!(archetype.scripts.is_none());
    assert!(archetype.hierarchies.is_none());
}

#[test]
fn push_entity_populates_expected_components() {
    let signature = signature_with(&[
        ComponentKind::Render,
        ComponentKind::Model,
        ComponentKind::Texture,
    ]);
    let mut archetype = Archetype::new(signature);

    let mut bundle = make_bundle(
        1,
        Position {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        Name("Entity".into()),
    );
    bundle.render = Some(RenderComponent::new([1.0, 0.0, 0.0], 1.0));
    bundle.model = Some(ModelComponent::new("assets/cube.obj"));
    bundle.texture = Some(TextureComponent::new("assets/textures/blue.png"));

    let index = archetype.push_entity(bundle);
    assert_eq!(index, 0);
    assert_eq!(archetype.entity_ids.len(), 1);
    let render = archetype
        .renderables
        .as_ref()
        .and_then(|column| column.get(0))
        .expect("render missing");
    assert_eq!(render.color, [1.0, 0.0, 0.0]);
    assert_eq!(
        archetype
            .models
            .as_ref()
            .and_then(|column| column.get(0))
            .expect("model missing")
            .asset_path,
        "assets/cube.obj"
    );
    assert_eq!(
        archetype
            .textures
            .as_ref()
            .and_then(|column| column.get(0))
            .expect("texture missing")
            .asset_path,
        "assets/textures/blue.png"
    );
}

#[test]
fn swap_remove_entity_returns_component_bundle() {
    let signature = signature_with(&[
        ComponentKind::Input,
        ComponentKind::Camera,
        ComponentKind::Light,
        ComponentKind::Script,
        ComponentKind::Terrain,
    ]);
    let mut archetype = Archetype::new(signature);

    let mut bundle_a = make_bundle(
        10,
        Position {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        Name("A".into()),
    );
    bundle_a.input = Some(InputComponent::new(0.1));
    bundle_a.camera = Some(CameraComponent::new(0.0, 0.0));
    bundle_a.light = Some(LightComponent::directional(
        [0.0, -1.0, 0.0],
        [1.0, 1.0, 1.0],
        1.0,
    ));
    bundle_a.script = Some(ScriptComponent::new("test", 0.0));
    bundle_a.terrain = Some(TerrainComponent::new(
        10.0,
        0.2,
        [0.2, 0.5, 0.2],
        None,
        "assets/terrain_plane.obj".into(),
    ));

    let mut bundle_b = bundle_a.clone();
    bundle_b.id = 11;
    bundle_b.name = Name("B".into());
    bundle_b.position.x = 1.0;

    archetype.push_entity(bundle_a);
    archetype.push_entity(bundle_b);

    let (removed, swapped) = archetype.swap_remove_entity(0);
    assert_eq!(removed.id, 10);
    assert!(swapped.is_some());
    assert_eq!(removed.name.0, "A");
    assert!(removed.input.is_some());
    assert!(removed.camera.is_some());
    assert!(removed.light.is_some());
    assert!(removed.script.is_some());
    assert!(removed.terrain.is_some());
    assert_eq!(archetype.entity_ids.len(), 1);
    assert_eq!(archetype.entity_ids[0], 11);
}
