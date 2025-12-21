use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::components::{
    AttributesComponent, CameraComponent, HierarchyComponent, InputComponent, LightComponent,
    ModelComponent, Name, Orientation, ParticleEmitterComponent, PhysicsBodyType, PhysicsComponent,
    Position, RenderComponent, ScriptComponent, TerrainComponent, TextureComponent,
};
use crate::ecs::ECS;
use log::warn;
use serde::Deserialize;

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
pub enum SceneLoadError {
    Io(std::io::Error),
    Parse(serde_yaml::Error),
    InvalidBodyType { value: String },
}

impl fmt::Display for SceneLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SceneLoadError::Io(err) => write!(f, "Failed to read scene file: {}", err),
            SceneLoadError::Parse(err) => write!(f, "Failed to parse scene file: {}", err),
            SceneLoadError::InvalidBodyType { value } => {
                write!(f, "Unknown physics body type '{}'", value)
            }
        }
    }
}

impl std::error::Error for SceneLoadError {}

pub fn load_scene_from_yaml(path: impl AsRef<Path>) -> Result<SceneDefinition, SceneLoadError> {
    let file = File::open(path.as_ref()).map_err(SceneLoadError::Io)?;
    let reader = BufReader::new(file);
    let scene_file: SceneFile = serde_yaml::from_reader(reader).map_err(SceneLoadError::Parse)?;
    scene_file.into_scene_definition()
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
    pub parent: Option<String>,
    pub tags: Vec<String>,
    pub components: ComponentDefinition,
}

impl EntityDefinition {
    pub fn new(name: impl Into<String>, position: Position) -> Self {
        Self {
            name: name.into(),
            position,
            orientation: Orientation::identity(),
            parent: None,
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

    pub fn with_parent(mut self, parent: impl Into<String>) -> Self {
        self.parent = Some(parent.into());
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
    pub attributes: Option<AttributesComponentDefinition>,
    pub particle_emitter: Option<ParticleEmitterComponentDefinition>,
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

#[derive(Debug, Clone, Default)]
pub struct AttributesComponentDefinition {
    pub values: HashMap<String, f32>,
}

impl AttributesComponentDefinition {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn with_value(mut self, key: impl Into<String>, value: f32) -> Self {
        self.values.insert(key.into(), value);
        self
    }
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
pub struct ParticleEmitterComponentDefinition {
    pub rate: f32,
    pub lifetime: f32,
    pub speed: f32,
    pub spread: f32,
    pub direction: [f32; 3],
    pub size: f32,
    pub size_jitter: f32,
    pub color: [f32; 3],
    pub color_jitter: f32,
    pub model_asset: String,
    pub texture_asset: Option<String>,
    pub max_particles: usize,
}

impl Default for ParticleEmitterComponentDefinition {
    fn default() -> Self {
        Self {
            rate: 12.0,
            lifetime: 1.2,
            speed: 2.5,
            spread: 0.6,
            direction: [0.0, 0.0, -1.0],
            size: 0.08,
            size_jitter: 0.0,
            color: [1.0, 0.6, 0.2],
            color_jitter: 0.0,
            model_asset: "assets/cube.obj".to_string(),
            texture_asset: None,
            max_particles: 128,
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

#[derive(Debug, Deserialize)]
struct SceneFile {
    #[serde(default)]
    environment: Option<SceneSettingsConfig>,
    #[serde(default)]
    entities: Vec<EntityConfig>,
}

#[derive(Debug, Deserialize, Default)]
struct SceneSettingsConfig {
    background_top: Option<[f32; 3]>,
    background_bottom: Option<[f32; 3]>,
    fog_color: Option<[f32; 3]>,
    fog_density: Option<f32>,
    background_sound: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EntityConfig {
    name: String,
    position: PositionConfig,
    #[serde(default)]
    orientation: Option<OrientationConfig>,
    #[serde(default)]
    parent: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    components: ComponentsConfig,
}

#[derive(Debug, Deserialize)]
struct PositionConfig {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug, Deserialize)]
struct OrientationConfig {
    yaw: f32,
    pitch: f32,
    roll: f32,
}

#[derive(Debug, Deserialize, Default)]
struct ComponentsConfig {
    render: Option<RenderConfig>,
    model: Option<ModelConfig>,
    camera: Option<CameraConfig>,
    input: Option<InputConfig>,
    light: Option<LightConfig>,
    texture: Option<TextureConfig>,
    terrain: Option<TerrainConfig>,
    script: Option<ScriptConfig>,
    physics: Option<PhysicsConfig>,
    attributes: Option<AttributesConfig>,
    particle_emitter: Option<ParticleEmitterConfig>,
}

#[derive(Debug, Deserialize)]
struct RenderConfig {
    color: [f32; 3],
    size: f32,
}

#[derive(Debug, Deserialize)]
struct ModelConfig {
    asset: String,
}

#[derive(Debug, Deserialize, Default)]
struct CameraConfig {
    yaw: Option<f32>,
    pitch: Option<f32>,
    move_speed: Option<f32>,
    look_sensitivity: Option<f32>,
}

#[derive(Debug, Deserialize, Default)]
struct InputConfig {
    speed: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct LightConfig {
    direction: [f32; 3],
    color: [f32; 3],
    intensity: f32,
    point_radius: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct TextureConfig {
    asset: String,
}

#[derive(Debug, Deserialize)]
struct TerrainConfig {
    size: f32,
    height: f32,
    color: [f32; 3],
    texture: Option<String>,
    model_asset: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ScriptConfig {
    name: String,
    #[serde(default)]
    params: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct PhysicsConfig {
    body_type: Option<String>,
    half_extents: Option<[f32; 3]>,
    restitution: Option<f32>,
    friction: Option<f32>,
}

#[derive(Debug, Deserialize, Default)]
struct AttributesConfig {
    #[serde(default)]
    values: HashMap<String, f32>,
}

#[derive(Debug, Deserialize, Default)]
struct ParticleEmitterConfig {
    rate: Option<f32>,
    lifetime: Option<f32>,
    speed: Option<f32>,
    spread: Option<f32>,
    direction: Option<[f32; 3]>,
    size: Option<f32>,
    size_jitter: Option<f32>,
    color: Option<[f32; 3]>,
    color_jitter: Option<f32>,
    model_asset: Option<String>,
    texture_asset: Option<String>,
    max_particles: Option<usize>,
}

impl SceneFile {
    fn into_scene_definition(self) -> Result<SceneDefinition, SceneLoadError> {
        let mut settings = SceneSettings::default();
        if let Some(environment) = self.environment {
            if let Some(top) = environment.background_top {
                settings.background_top = top;
            }
            if let Some(bottom) = environment.background_bottom {
                settings.background_bottom = bottom;
            }
            if let Some(color) = environment.fog_color {
                settings.fog_color = color;
            }
            if let Some(density) = environment.fog_density {
                settings.fog_density = density;
            }
            if let Some(sound) = environment.background_sound {
                settings.background_sound = Some(sound);
            }
        }

        let mut scene = SceneDefinition::new(settings);
        for entity in self.entities {
            scene.add_entity(entity.into_definition()?);
        }
        Ok(scene)
    }
}

impl EntityConfig {
    fn into_definition(self) -> Result<EntityDefinition, SceneLoadError> {
        let position = Position {
            x: self.position.x,
            y: self.position.y,
            z: self.position.z,
        };
        let mut definition = EntityDefinition::new(self.name, position);
        if let Some(orientation) = self.orientation {
            definition = definition.with_orientation(Orientation::from_yaw_pitch_roll(
                orientation.yaw,
                orientation.pitch,
                orientation.roll,
            ));
        }
        if let Some(parent) = self.parent {
            definition = definition.with_parent(parent);
        }
        if !self.tags.is_empty() {
            definition = definition.with_tags(self.tags);
        }
        let components = self.components.into_definition()?;
        definition = definition.with_components(components);
        Ok(definition)
    }
}

impl ComponentsConfig {
    fn into_definition(self) -> Result<ComponentDefinition, SceneLoadError> {
        Ok(ComponentDefinition {
            render: self.render.map(|cfg| RenderComponentDefinition {
                color: cfg.color,
                size: cfg.size,
            }),
            model: self
                .model
                .map(|cfg| ModelComponentDefinition { asset: cfg.asset }),
            camera: self.camera.map(|cfg| CameraComponentDefinition {
                yaw: cfg.yaw,
                pitch: cfg.pitch,
                move_speed: cfg.move_speed,
                look_sensitivity: cfg.look_sensitivity,
            }),
            input: self
                .input
                .map(|cfg| InputComponentDefinition { speed: cfg.speed }),
            light: self.light.map(|cfg| LightComponentDefinition {
                direction: cfg.direction,
                color: cfg.color,
                intensity: cfg.intensity,
                point_radius: cfg.point_radius,
            }),
            texture: self
                .texture
                .map(|cfg| TextureComponentDefinition { asset: cfg.asset }),
            terrain: self.terrain.map(|cfg| TerrainComponentDefinition {
                size: cfg.size,
                height: cfg.height,
                color: cfg.color,
                texture: cfg.texture,
                model_asset: cfg
                    .model_asset
                    .unwrap_or_else(default_terrain_model_asset),
            }),
            script: self.script.map(|cfg| ScriptComponentDefinition {
                name: cfg.name,
                params: cfg.params,
            }),
            physics: match self.physics {
                Some(cfg) => Some(cfg.into_definition()?),
                None => None,
            },
            attributes: self.attributes.map(|cfg| AttributesComponentDefinition {
                values: cfg.values,
            }),
            particle_emitter: self.particle_emitter.map(|cfg| {
                let defaults = ParticleEmitterComponentDefinition::default();
                ParticleEmitterComponentDefinition {
                    rate: cfg.rate.unwrap_or(defaults.rate),
                    lifetime: cfg.lifetime.unwrap_or(defaults.lifetime),
                    speed: cfg.speed.unwrap_or(defaults.speed),
                    spread: cfg.spread.unwrap_or(defaults.spread),
                    direction: cfg.direction.unwrap_or(defaults.direction),
                    size: cfg.size.unwrap_or(defaults.size),
                    size_jitter: cfg.size_jitter.unwrap_or(defaults.size_jitter),
                    color: cfg.color.unwrap_or(defaults.color),
                    color_jitter: cfg.color_jitter.unwrap_or(defaults.color_jitter),
                    model_asset: cfg
                        .model_asset
                        .unwrap_or_else(|| defaults.model_asset.clone()),
                    texture_asset: cfg.texture_asset.or_else(|| defaults.texture_asset.clone()),
                    max_particles: cfg.max_particles.unwrap_or(defaults.max_particles),
                }
            }),
        })
    }
}

impl PhysicsConfig {
    fn into_definition(self) -> Result<PhysicsComponentDefinition, SceneLoadError> {
        let mut definition = PhysicsComponentDefinition::default();
        if let Some(body_type) = self.body_type {
            definition.body_type = parse_body_type(&body_type)
                .ok_or(SceneLoadError::InvalidBodyType { value: body_type })?;
        }
        if let Some(half_extents) = self.half_extents {
            definition.half_extents = Some(half_extents);
        }
        if let Some(restitution) = self.restitution {
            definition.restitution = restitution;
        }
        if let Some(friction) = self.friction {
            definition.friction = friction;
        }
        Ok(definition)
    }
}

fn parse_body_type(value: &str) -> Option<PhysicsBodyType> {
    match value.to_ascii_lowercase().as_str() {
        "dynamic" => Some(PhysicsBodyType::Dynamic),
        "static" => Some(PhysicsBodyType::Static),
        "kinematic" => Some(PhysicsBodyType::Kinematic),
        _ => None,
    }
}

pub fn apply_scene_definition(scene: &SceneDefinition, ecs: &mut ECS) -> SceneSettings {
    let mut name_to_entity = HashMap::new();
    let mut ordered_ids = Vec::with_capacity(scene.entities.len());

    for entity in &scene.entities {
        let entity_id = spawn_entity_from_definition(ecs, entity);
        name_to_entity.insert(entity.name.clone(), entity_id);
        ordered_ids.push(entity_id);
    }

    attach_entity_hierarchies(scene, ecs, &ordered_ids, &name_to_entity);

    scene.settings.clone()
}

fn spawn_entity_from_definition(ecs: &mut ECS, entity: &EntityDefinition) -> u32 {
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
            LightComponent::directional(light_cfg.direction, light_cfg.color, light_cfg.intensity)
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

    if let Some(attributes_cfg) = components.attributes.as_ref() {
        ecs.add_attributes_component(
            entity_id,
            AttributesComponent::from_values(attributes_cfg.values.clone()),
        );
    }

    if let Some(emitter_cfg) = components.particle_emitter.as_ref() {
        ecs.add_particle_emitter_component(
            entity_id,
            ParticleEmitterComponent::new(
                emitter_cfg.rate,
                emitter_cfg.lifetime,
                emitter_cfg.speed,
                emitter_cfg.spread,
                emitter_cfg.direction,
                emitter_cfg.size,
                emitter_cfg.size_jitter,
                emitter_cfg.color,
                emitter_cfg.color_jitter,
                emitter_cfg.model_asset.clone(),
                emitter_cfg.texture_asset.clone(),
                emitter_cfg.max_particles,
            ),
        );
    }

    entity_id
}

fn attach_entity_hierarchies(
    scene: &SceneDefinition,
    ecs: &mut ECS,
    ordered_ids: &[u32],
    name_to_entity: &HashMap<String, u32>,
) {
    for (index, entity) in scene.entities.iter().enumerate() {
        let Some(parent_name) = entity.parent.as_ref() else {
            continue;
        };
        let Some(&parent_id) = name_to_entity.get(parent_name) else {
            warn!(
                "Cannot attach '{}': parent '{}' not found",
                entity.name, parent_name
            );
            continue;
        };
        let child_id = ordered_ids[index];
        if child_id == parent_id {
            warn!(
                "Entity '{}' cannot be its own parent; skipping hierarchy attachment",
                entity.name
            );
            continue;
        }
        let Some((parent_position, parent_orientation, _)) = ecs.find_entity_components(parent_id)
        else {
            warn!(
                "Cannot attach '{}': parent entity {} missing components",
                entity.name, parent_id
            );
            continue;
        };
        let Some((child_position, child_orientation, _)) = ecs.find_entity_components(child_id)
        else {
            continue;
        };
        let hierarchy = HierarchyComponent::from_world_transforms(
            parent_id,
            *parent_position,
            *parent_orientation,
            *child_position,
            *child_orientation,
        );
        ecs.add_hierarchy_component(child_id, hierarchy);
    }
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
