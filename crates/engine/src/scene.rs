use std::collections::HashMap;
use std::fmt;

use crate::components::{
    CameraComponent, InputComponent, LightComponent, ModelComponent, Name, Orientation,
    PhysicsBodyType, PhysicsComponent, Position, RenderComponent, ScriptComponent,
    TerrainComponent, TextureComponent,
};
use crate::ecs::ECS;

#[derive(Debug, Clone)]
pub struct SceneDefinition {
    pub settings: SceneSettings,
    pub entities: Vec<EntityDefinition>,
}

impl Default for SceneDefinition {
    fn default() -> Self {
        Self {
            settings: SceneSettings::default(),
            entities: Vec::new(),
        }
    }
}

impl SceneDefinition {
    pub fn new(settings: SceneSettings) -> Self {
        Self {
            settings,
            entities: Vec::new(),
        }
    }

    pub fn with_entities(mut self, entities: Vec<EntityDefinition>) -> Self {
        self.entities = entities;
        self
    }

    pub fn add_entity(&mut self, entity: EntityDefinition) {
        self.entities.push(entity);
    }
}

#[derive(Debug, Default)]
pub struct SceneLibrary {
    scenes: HashMap<String, SceneDefinition>,
}

impl SceneLibrary {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(
        &mut self,
        id: impl Into<String>,
        scene: SceneDefinition,
    ) -> Option<SceneDefinition> {
        self.scenes.insert(id.into(), scene)
    }

    pub fn with_scene(mut self, id: impl Into<String>, scene: SceneDefinition) -> Self {
        self.insert(id, scene);
        self
    }

    pub fn get(&self, id: &str) -> Option<&SceneDefinition> {
        self.scenes.get(id)
    }

    pub fn contains(&self, id: &str) -> bool {
        self.scenes.contains_key(id)
    }
}

#[derive(Debug)]
pub enum SceneLookupError {
    NotFound(String),
}

impl fmt::Display for SceneLookupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SceneLookupError::NotFound(id) => write!(f, "Scene '{}' not found", id),
        }
    }
}

impl std::error::Error for SceneLookupError {}

#[derive(Debug, Clone)]
pub struct EntityDefinition {
    pub name: String,
    pub position: Position,
    pub orientation: Orientation,
    pub tags: Vec<String>,
    pub components: ComponentDefinition,
}

impl EntityDefinition {
    pub fn new(name: impl Into<String>, position: Position) -> Self {
        Self {
            name: name.into(),
            position,
            orientation: Orientation::identity(),
            tags: Vec::new(),
            components: ComponentDefinition::default(),
        }
    }

    pub fn with_tags<I, S>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.tags.extend(tags.into_iter().map(Into::into));
        self
    }

    pub fn with_components(mut self, components: ComponentDefinition) -> Self {
        self.components = components;
        self
    }

    pub fn with_orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct ComponentDefinition {
    pub render: Option<RenderComponentDefinition>,
    pub model: Option<ModelComponentDefinition>,
    pub camera: Option<CameraComponentDefinition>,
    pub input: Option<InputComponentDefinition>,
    pub light: Option<LightComponentDefinition>,
    pub texture: Option<TextureComponentDefinition>,
    pub terrain: Option<TerrainComponentDefinition>,
    pub script: Option<ScriptComponentDefinition>,
    pub physics: Option<PhysicsComponentDefinition>,
}

#[derive(Debug, Clone)]
pub struct RenderComponentDefinition {
    pub color: [f32; 3],
    pub size: f32,
}

#[derive(Debug, Clone)]
pub struct ModelComponentDefinition {
    pub asset: String,
}

#[derive(Debug, Clone, Default)]
pub struct CameraComponentDefinition {
    pub yaw: Option<f32>,
    pub pitch: Option<f32>,
    pub move_speed: Option<f32>,
    pub look_sensitivity: Option<f32>,
}

#[derive(Debug, Clone, Default)]
pub struct InputComponentDefinition {
    pub speed: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct LightComponentDefinition {
    pub direction: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
    pub point_radius: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct TextureComponentDefinition {
    pub asset: String,
}

#[derive(Debug, Clone)]
pub struct TerrainComponentDefinition {
    pub size: f32,
    pub height: f32,
    pub color: [f32; 3],
    pub texture: Option<String>,
    pub model_asset: String,
}

impl Default for TerrainComponentDefinition {
    fn default() -> Self {
        Self {
            size: default_terrain_size(),
            height: default_terrain_height(),
            color: default_terrain_color(),
            texture: None,
            model_asset: default_terrain_model_asset(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PhysicsComponentDefinition {
    pub body_type: PhysicsBodyType,
    pub half_extents: Option<[f32; 3]>,
    pub restitution: f32,
    pub friction: f32,
}

impl Default for PhysicsComponentDefinition {
    fn default() -> Self {
        Self {
            body_type: PhysicsBodyType::Dynamic,
            half_extents: Some(default_physics_half_extents()),
            restitution: 0.2,
            friction: 0.7,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScriptComponentDefinition {
    pub name: String,
    pub params: HashMap<String, String>,
}

impl ScriptComponentDefinition {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            params: HashMap::new(),
        }
    }

    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct SceneSettings {
    pub background_top: [f32; 3],
    pub background_bottom: [f32; 3],
    pub fog_color: [f32; 3],
    pub fog_density: f32,
    pub background_sound: Option<String>,
}

impl Default for SceneSettings {
    fn default() -> Self {
        Self {
            background_top: default_background_top(),
            background_bottom: default_background_bottom(),
            fog_color: default_fog_color(),
            fog_density: default_fog_density(),
            background_sound: None,
        }
    }
}

fn default_terrain_size() -> f32 {
    20.0
}

fn default_terrain_height() -> f32 {
    0.2
}

fn default_terrain_color() -> [f32; 3] {
    [0.2, 0.5, 0.2]
}

fn default_terrain_model_asset() -> String {
    "assets/terrain_plane.obj".to_string()
}

fn default_background_top() -> [f32; 3] {
    [0.18, 0.26, 0.42]
}

fn default_background_bottom() -> [f32; 3] {
    [0.03, 0.03, 0.08]
}

fn default_fog_color() -> [f32; 3] {
    [0.18, 0.22, 0.28]
}

fn default_fog_density() -> f32 {
    0.45
}

fn default_physics_half_extents() -> [f32; 3] {
    [0.5, 0.5, 0.5]
}

pub fn apply_scene_definition(scene: &SceneDefinition, ecs: &mut ECS) -> SceneSettings {
    for entity in &scene.entities {
        let position = entity.position;
        let base_height = position.y;
        let entity_id = ecs.add_entity(position, entity.orientation, Name(entity.name.clone()));

        for tag in &entity.tags {
            ecs.tag_manager.add_tag(entity_id, tag);
        }

        let components = &entity.components;
        let mut render_cfg = components.render.clone();
        let mut model_cfg = components.model.clone();
        let mut texture_cfg = components.texture.clone();

        if let Some(terrain_cfg) = components.terrain.as_ref() {
            if render_cfg.is_none() {
                render_cfg = Some(RenderComponentDefinition {
                    color: terrain_cfg.color,
                    size: terrain_cfg.size,
                });
            }

            if model_cfg.is_none() {
                model_cfg = Some(ModelComponentDefinition {
                    asset: terrain_cfg.model_asset.clone(),
                });
            }

            if texture_cfg.is_none() {
                if let Some(texture_asset) = &terrain_cfg.texture {
                    texture_cfg = Some(TextureComponentDefinition {
                        asset: texture_asset.clone(),
                    });
                }
            }

            ecs.add_terrain_component(
                entity_id,
                TerrainComponent::new(
                    terrain_cfg.size,
                    terrain_cfg.height,
                    terrain_cfg.color,
                    terrain_cfg.texture.clone(),
                    terrain_cfg.model_asset.clone(),
                ),
            );
        }

        if let Some(physics_cfg) = components.physics.as_ref() {
            let resolved_half_extents = resolve_physics_half_extents(
                physics_cfg.half_extents,
                render_cfg.as_ref(),
                components.terrain.as_ref(),
            );
            let mut physics = PhysicsComponent::new(physics_cfg.body_type, resolved_half_extents);
            physics.restitution = physics_cfg.restitution;
            physics.friction = physics_cfg.friction;
            ecs.add_physics_component(entity_id, physics);
        }

        if let Some(render_cfg) = render_cfg {
            ecs.add_render_component(
                entity_id,
                RenderComponent::new(render_cfg.color, render_cfg.size),
            );
        }

        if let Some(model_cfg) = model_cfg {
            ecs.add_model_component(entity_id, ModelComponent::new(model_cfg.asset));
        }

        if let Some(camera_cfg) = components.camera.as_ref() {
            let mut camera = CameraComponent::new(
                camera_cfg.yaw.unwrap_or(0.0),
                camera_cfg.pitch.unwrap_or(0.0),
            );
            if let Some(speed) = camera_cfg.move_speed {
                camera.move_speed = speed;
            }
            if let Some(look) = camera_cfg.look_sensitivity {
                camera.look_sensitivity = look;
            }
            ecs.add_camera_component(entity_id, camera);
        }

        if let Some(input_cfg) = components.input.as_ref() {
            let speed = input_cfg.speed.unwrap_or(0.05);
            ecs.add_input_component(entity_id, InputComponent::new(speed));
        }

        if let Some(light_cfg) = components.light.as_ref() {
            let component = if let Some(radius) = light_cfg.point_radius {
                LightComponent::point(radius, light_cfg.color, light_cfg.intensity)
            } else {
                LightComponent::directional(
                    light_cfg.direction,
                    light_cfg.color,
                    light_cfg.intensity,
                )
            };
            ecs.add_light_component(entity_id, component);
        }

        if let Some(texture_cfg) = texture_cfg {
            ecs.add_texture_component(entity_id, TextureComponent::new(texture_cfg.asset));
        }

        if let Some(script_cfg) = components.script.as_ref() {
            ecs.add_script_component(
                entity_id,
                ScriptComponent::with_params(
                    script_cfg.name.clone(),
                    base_height,
                    script_cfg.params.clone(),
                ),
            );
        }
    }

    scene.settings.clone()
}

fn resolve_physics_half_extents(
    explicit: Option<[f32; 3]>,
    render: Option<&RenderComponentDefinition>,
    terrain: Option<&TerrainComponentDefinition>,
) -> [f32; 3] {
    if let Some(half_extents) = explicit {
        return half_extents;
    }
    if let Some(terrain) = terrain {
        return [terrain.size * 0.5, terrain.height * 0.5, terrain.size * 0.5];
    }
    if let Some(render) = render {
        let half = render.size * 0.5;
        return [half, half, half];
    }
    default_physics_half_extents()
}
