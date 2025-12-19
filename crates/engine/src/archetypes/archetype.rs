use crate::components::{
    CameraComponent, HierarchyComponent, InputComponent, LightComponent, ModelComponent, Name,
    Orientation, PhysicsComponent, Position, RenderComponent, ScriptComponent, TerrainComponent,
    TextureComponent,
};
use crate::ecs::{ComponentKind, ComponentSignature};

#[derive(Debug, Clone)]
pub struct EntityComponents {
    pub id: u32,
    pub position: Position,
    pub orientation: Orientation,
    pub name: Name,
    pub render: Option<RenderComponent>,
    pub input: Option<InputComponent>,
    pub model: Option<ModelComponent>,
    pub camera: Option<CameraComponent>,
    pub light: Option<LightComponent>,
    pub texture: Option<TextureComponent>,
    pub terrain: Option<TerrainComponent>,
    pub script: Option<ScriptComponent>,
    pub physics: Option<PhysicsComponent>,
    pub hierarchy: Option<HierarchyComponent>,
}

impl EntityComponents {
    pub fn base(id: u32, position: Position, orientation: Orientation, name: Name) -> Self {
        Self {
            id,
            position,
            orientation,
            name,
            render: None,
            input: None,
            model: None,
            camera: None,
            light: None,
            texture: None,
            terrain: None,
            script: None,
            physics: None,
            hierarchy: None,
        }
    }
}

#[derive(Debug)]
pub struct Archetype {
    pub signature: ComponentSignature,
    pub entity_ids: Vec<u32>,
    pub positions: Vec<Position>,
    pub orientations: Vec<Orientation>,
    pub names: Vec<Name>,
    pub renderables: Option<Vec<RenderComponent>>,
    pub inputs: Option<Vec<InputComponent>>,
    pub models: Option<Vec<ModelComponent>>,
    pub cameras: Option<Vec<CameraComponent>>,
    pub lights: Option<Vec<LightComponent>>,
    pub textures: Option<Vec<TextureComponent>>,
    pub terrains: Option<Vec<TerrainComponent>>,
    pub scripts: Option<Vec<ScriptComponent>>,
    pub physics: Option<Vec<PhysicsComponent>>,
    pub hierarchies: Option<Vec<HierarchyComponent>>,
}

impl Archetype {
    pub fn new(signature: ComponentSignature) -> Self {
        Self {
            signature,
            entity_ids: Vec::new(),
            positions: Vec::new(),
            orientations: Vec::new(),
            names: Vec::new(),
            renderables: signature
                .contains(ComponentKind::Render)
                .then(|| Vec::new()),
            inputs: signature.contains(ComponentKind::Input).then(|| Vec::new()),
            models: signature.contains(ComponentKind::Model).then(|| Vec::new()),
            cameras: signature
                .contains(ComponentKind::Camera)
                .then(|| Vec::new()),
            lights: signature.contains(ComponentKind::Light).then(|| Vec::new()),
            textures: signature
                .contains(ComponentKind::Texture)
                .then(|| Vec::new()),
            terrains: signature
                .contains(ComponentKind::Terrain)
                .then(|| Vec::new()),
            scripts: signature
                .contains(ComponentKind::Script)
                .then(|| Vec::new()),
            physics: signature
                .contains(ComponentKind::Physics)
                .then(|| Vec::new()),
            hierarchies: signature
                .contains(ComponentKind::Hierarchy)
                .then(|| Vec::new()),
        }
    }

    pub fn len(&self) -> usize {
        self.entity_ids.len()
    }

    pub fn push_entity(&mut self, components: EntityComponents) -> usize {
        let index = self.entity_ids.len();
        let EntityComponents {
            id,
            position,
            orientation,
            name,
            render,
            input,
            model,
            camera,
            light,
            texture,
            terrain,
            script,
            physics: physics_component,
            hierarchy,
        } = components;

        self.entity_ids.push(id);
        self.positions.push(position);
        self.orientations.push(orientation);
        self.names.push(name);
        if let Some(renderables) = self.renderables.as_mut() {
            renderables.push(render.expect("render component missing for signature"));
        } else {
            debug_assert!(render.is_none());
        }
        if let Some(inputs) = self.inputs.as_mut() {
            inputs.push(input.expect("input component missing for signature"));
        } else {
            debug_assert!(input.is_none());
        }
        if let Some(models) = self.models.as_mut() {
            models.push(model.expect("model component missing for signature"));
        } else {
            debug_assert!(model.is_none());
        }
        if let Some(cameras) = self.cameras.as_mut() {
            cameras.push(camera.expect("camera component missing for signature"));
        } else {
            debug_assert!(camera.is_none());
        }
        if let Some(lights) = self.lights.as_mut() {
            lights.push(light.expect("light component missing for signature"));
        } else {
            debug_assert!(light.is_none());
        }
        if let Some(textures) = self.textures.as_mut() {
            textures.push(texture.expect("texture component missing for signature"));
        } else {
            debug_assert!(texture.is_none());
        }
        if let Some(terrains) = self.terrains.as_mut() {
            terrains.push(terrain.expect("terrain component missing for signature"));
        } else {
            debug_assert!(terrain.is_none());
        }
        if let Some(scripts) = self.scripts.as_mut() {
            scripts.push(script.expect("script component missing for signature"));
        } else {
            debug_assert!(script.is_none());
        }
        if let Some(physics) = self.physics.as_mut() {
            physics.push(physics_component.expect("physics component missing for signature"));
        } else {
            debug_assert!(physics_component.is_none());
        }
        if let Some(hierarchies) = self.hierarchies.as_mut() {
            hierarchies.push(hierarchy.expect("hierarchy component missing for signature"));
        } else {
            debug_assert!(hierarchy.is_none());
        }
        index
    }

    pub fn swap_remove_entity(&mut self, index: usize) -> (EntityComponents, Option<u32>) {
        let id = self.entity_ids.swap_remove(index);
        let position = self.positions.swap_remove(index);
        let orientation = self.orientations.swap_remove(index);
        let name = self.names.swap_remove(index);

        let render = self
            .renderables
            .as_mut()
            .map(|column| column.swap_remove(index));
        let input = self.inputs.as_mut().map(|column| column.swap_remove(index));
        let model = self.models.as_mut().map(|column| column.swap_remove(index));
        let camera = self
            .cameras
            .as_mut()
            .map(|column| column.swap_remove(index));
        let light = self.lights.as_mut().map(|column| column.swap_remove(index));
        let texture = self
            .textures
            .as_mut()
            .map(|column| column.swap_remove(index));
        let terrain = self
            .terrains
            .as_mut()
            .map(|column| column.swap_remove(index));
        let script = self
            .scripts
            .as_mut()
            .map(|column| column.swap_remove(index));
        let physics = self
            .physics
            .as_mut()
            .map(|column| column.swap_remove(index));
        let hierarchy = self
            .hierarchies
            .as_mut()
            .map(|column| column.swap_remove(index));

        let swapped_id = if index < self.entity_ids.len() {
            Some(self.entity_ids[index])
        } else {
            None
        };

        (
            EntityComponents {
                id,
                position,
                orientation,
                name,
                render,
                input,
                model,
                camera,
                light,
                texture,
                terrain,
                script,
                physics,
                hierarchy,
            },
            swapped_id,
        )
    }
}
