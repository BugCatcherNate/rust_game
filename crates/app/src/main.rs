use std::sync::Arc;

use glam::Quat;
use rust_game::app::{run_game, CustomSystem, GameConfig, ShotEvent, ShotEventHit};
use rust_game::components::{Orientation, ParticleBurstRequest, PhysicsBodyType, Position};
use rust_game::ecs::ECS;
use rust_game::math::normalize_vec3;
use rust_game::modules;
use rust_game::scene::{
    AttributesComponentDefinition, CameraComponentDefinition, ComponentDefinition,
    EntityDefinition, InputComponentDefinition, LightComponentDefinition, ModelComponentDefinition,
    ParticleEmitterComponentDefinition, PhysicsComponentDefinition, RenderComponentDefinition,
    SceneDefinition, SceneLibrary, SceneSettings, ScriptComponentDefinition,
    TerrainComponentDefinition,
};
use rust_game::scripts::{ScriptBehavior, ScriptCommand, ScriptContext, ScriptRegistry};

const LABYRINTH_SCENE_ID: &str = "labyrinth";

fn build_script_registry() -> Arc<ScriptRegistry> {
    let mut registry = ScriptRegistry::new();
    registry.register_script("spinner", || Box::new(SpinnerScript::default()));
    Arc::new(registry)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    modules::core::initialize();

    let scenes = SceneLibrary::new().with_scene(LABYRINTH_SCENE_ID, labyrinth_scene());
    let script_registry = build_script_registry();

    let config = GameConfig::new(LABYRINTH_SCENE_ID, scenes, script_registry)
        .with_window_title("My Rust Game")
        .with_custom_system(ShootingSystem::new());

    run_game(config)
}

fn labyrinth_scene() -> SceneDefinition {
    let mut scene = SceneDefinition::new(SceneSettings {
        background_top: [0.08, 0.11, 0.19],
        background_bottom: [0.01, 0.01, 0.03],
        fog_color: [0.04, 0.07, 0.12],
        fog_density: 0.001,
        background_sound: Some("assets/audio/forest.wav".to_string()),
    });
    scene.add_entity(explorer_entity());
    scene.add_entity(player_gun());
    scene.add_entity(target());
    scene.add_entity(fire_emitter());
    scene.add_entity(labyrinth_floor());
    scene.add_entity(sun_light());
    scene.add_entity(tree_prop());
    scene
}

fn explorer_entity() -> EntityDefinition {
    EntityDefinition::new(
        "Explorer",
        Position {
            x: 0.0,
            y: 1.6,
            z: 6.0,
        },
    )
    .with_tags(["player", "camera"])
    .with_components(ComponentDefinition {
        camera: Some(CameraComponentDefinition {
            yaw: Some(0.0),
            pitch: Some(0.0),
            move_speed: Some(0.05),
            look_sensitivity: Some(0.0025),
        }),
        input: Some(InputComponentDefinition { speed: Some(0.05) }),
        physics: Some(PhysicsComponentDefinition {
            body_type: PhysicsBodyType::Dynamic,
            half_extents: Some([0.3, 0.9, 0.3]),
            ..Default::default()
        }),
        ..Default::default()
    })
}

fn player_gun() -> EntityDefinition {
    let offset_position = Position {
        x: 0.2,
        y: 1.4,
        z: 5.5,
    };
    EntityDefinition::new("PlayerGun", offset_position)
        .with_parent("Explorer")
        .with_tags(["player_gun"])
        .with_components(ComponentDefinition {
            render: Some(RenderComponentDefinition {
                color: [0.8, 0.8, 0.2],
                size: 0.15,
            }),
            model: Some(ModelComponentDefinition {
                asset: "assets/cube.obj".to_string(),
            }),
            attributes: Some(AttributesComponentDefinition::default().with_value("ammo", 12.0).with_value("damage", 2.0)),
            ..Default::default()
        })
}

fn labyrinth_floor() -> EntityDefinition {
    let mut terrain = TerrainComponentDefinition::default();
    terrain.size = 22.0;
    terrain.height = 0.3;
    terrain.color = [0.4, 0.4, 0.4];
    terrain.texture = None;
    let terrain_size = terrain.size;
    let terrain_height = terrain.height;
    EntityDefinition::new(
        "LabyrinthFloor",
        Position {
            x: 0.0,
            y: -0.15,
            z: 0.0,
        },
    )
    .with_tags(["terrain"])
    .with_components(ComponentDefinition {
        terrain: Some(terrain),
        physics: Some(PhysicsComponentDefinition {
            body_type: PhysicsBodyType::Static,
            half_extents: Some([terrain_size * 0.5, terrain_height * 0.5, terrain_size * 0.5]),
            ..Default::default()
        }),
        ..Default::default()
    })
}

fn target() -> EntityDefinition {
    let position = Position {
        x: 0.0,
        y: 3.0,
        z: 0.0,
    };
    let mut entity = target_entity("target", position);
    entity.components.render = Some(RenderComponentDefinition {
        color: [1.0, 0.95, 0.6],
        size: 0.5,
    });
    entity.components.model = Some(ModelComponentDefinition {
        asset: "assets/cube.obj".to_string(),
    });
    entity.components.script =
        Some(ScriptComponentDefinition::new("spinner").with_param("speed", "0.75"));
    entity.components.attributes =
        Some(AttributesComponentDefinition::default().with_value("health", 4.0));

    entity
}

fn fire_emitter() -> EntityDefinition {
    EntityDefinition::new(
        "Campfire",
        Position {
            x: 0.0,
            y: 0.1,
            z: 0.0,
        },
    )
    .with_tags(["emitter", "fire"])
    .with_components(ComponentDefinition {
        particle_emitter: Some(ParticleEmitterComponentDefinition {
            rate: 48.0,
            lifetime: 0.45,
            speed: 1.3,
            spread: 0.5,
            direction: [0.0, -1.0, 0.0],
            size: 0.08,
            size_jitter: 0.04,
            color: [1.0, 0.6, 0.15],
            color_jitter: 0.2,
            model_asset: "assets/quad.obj".to_string(),
            texture_asset: None,
            max_particles: 140,
        }),
        ..Default::default()
    })
}
fn sun_light() -> EntityDefinition {
    let position = Position {
        x: 0.0,
        y: 30.0,
        z: 0.0,
    };
    let direction = normalize_vec3([0.0, -1.0, 0.0]);
    let mut entity = directional_light("Sun", position, direction, [1.0, 0.95, 0.85], 1.0);
    entity.components.render = Some(RenderComponentDefinition {
        color: [1.0, 0.95, 0.6],
        size: 0.5,
    });
    entity.components.model = Some(ModelComponentDefinition {
        asset: "assets/cube.obj".to_string(),
    });
    entity
}

fn tree_prop() -> EntityDefinition {
    EntityDefinition::new(
        "Tree",
        Position {
            x: 4.0,
            y: 0.0,
            z: -3.0,
        },
    )
    .with_tags(["prop"])
    .with_orientation(Orientation::from_yaw_pitch_roll(
        std::f32::consts::FRAC_PI_4,
        0.0,
        0.0,
    ))
    .with_components(ComponentDefinition {
        render: Some(RenderComponentDefinition {
            color: [0.3, 0.6, 0.2],
            size: 1.0,
        }),
        model: Some(ModelComponentDefinition {
            asset: "assets/tree.obj".to_string(),
        }),
        physics: Some(PhysicsComponentDefinition {
            body_type: PhysicsBodyType::Static,
            half_extents: Some([0.8, 2.0, 0.8]),
            ..Default::default()
        }),
        ..Default::default()
    })
}
fn target_entity(name: &str, position: Position) -> EntityDefinition {
    EntityDefinition::new(name, position)
        .with_tags(["target"])
        .with_components(ComponentDefinition {
            physics: Some(PhysicsComponentDefinition {
                body_type: PhysicsBodyType::Static,
                half_extents: Some([0.4, 0.4, 0.4]),
                ..Default::default()
            }),
            ..Default::default()
        })
}

#[derive(Default)]
struct SpinnerScript;

impl ScriptBehavior for SpinnerScript {
    fn update(
        &mut self,
        ctx: ScriptContext<'_>,
        _position: &mut Position,
        orientation: &mut Orientation,
        _commands: &mut Vec<ScriptCommand>,
    ) {
        let speed = ctx
            .params
            .get("speed")
            .and_then(|value| value.parse::<f32>().ok())
            .unwrap_or(1.0);
        let delta = speed * ctx.dt;
        let rotation = Quat::from_rotation_y(delta);
        orientation.set_quat(rotation * orientation.quat());
    }
}

fn directional_light(
    name: &str,
    position: Position,
    direction: [f32; 3],
    color: [f32; 3],
    intensity: f32,
) -> EntityDefinition {
    EntityDefinition::new(name, position)
        .with_tags(["light"])
        .with_components(ComponentDefinition {
            light: Some(LightComponentDefinition {
                direction,
                color,
                intensity,
                point_radius: None,
            }),
            ..Default::default()
        })
}

struct ShootingSystem {
    destroyed: usize,
    last_message: Option<String>,
    message_timer: i32,
    gun_id: Option<u32>,
    particle_seed: u32,
}

impl ShootingSystem {
    fn new() -> Self {
        Self {
            destroyed: 0,
            last_message: None,
            message_timer: 0,
            gun_id: None,
            particle_seed: 1,
        }
    }

    fn should_destroy(ecs: &ECS, entity_id: u32) -> bool {
        let tags = ecs.tag_manager.tags_for_entity(entity_id);
        !tags.iter().any(|tag| {
            tag == "terrain" || tag == "player" || tag == "camera" || tag == "player_gun"
        })
    }

    fn spawn_hit_burst(&mut self, ecs: &mut ECS, hit: &ShotEventHit) {
        const PARTICLE_COUNT: usize = 40;
        const PARTICLE_SPEED: f32 = 1.6;
        const PARTICLE_LIFETIME: f32 = 0.35;
        const PARTICLE_SIZE: f32 = 0.07;
        const PARTICLE_SIZE_JITTER: f32 = 0.02;
        const PARTICLE_COLOR_JITTER: f32 = 0.2;
        const PARTICLE_MODEL: &str = "assets/quad.obj";
        const PARTICLE_COLORS: [[f32; 3]; 3] = [
            [1.0, 0.85, 0.2],
            [1.0, 0.55, 0.1],
            [0.9, 0.2, 0.05],
        ];

        let color_index = (self.next_unit_random() * PARTICLE_COLORS.len() as f32) as usize;
        let color = PARTICLE_COLORS
            .get(color_index)
            .copied()
            .unwrap_or(PARTICLE_COLORS[0]);
        let seed = self.advance_seed();
        ecs.emit_event(ParticleBurstRequest {
            position: hit.point,
            direction: [-hit.normal[0], -hit.normal[1], -hit.normal[2]],
            count: PARTICLE_COUNT,
            speed: PARTICLE_SPEED,
            spread: 0.8,
            lifetime: PARTICLE_LIFETIME,
            size: PARTICLE_SIZE,
            size_jitter: PARTICLE_SIZE_JITTER,
            color,
            color_jitter: PARTICLE_COLOR_JITTER,
            model_asset: PARTICLE_MODEL.to_string(),
            texture_asset: Some("assets/textures/flame.png".to_string()),
            seed,
        });
    }

    fn advance_seed(&mut self) -> u32 {
        self.particle_seed = self
            .particle_seed
            .wrapping_mul(1664525)
            .wrapping_add(1013904223);
        self.particle_seed
    }

    fn next_unit_random(&mut self) -> f32 {
        let seed = self.advance_seed();
        (seed as f32 / u32::MAX as f32).clamp(0.0, 1.0)
    }
}

impl CustomSystem for ShootingSystem {
    fn scene_loaded(&mut self, ecs: &mut ECS, _scene: &str) {
        self.destroyed = 0;
        self.last_message = None;
        self.message_timer = 0;
        self.gun_id = ecs.find_entity_id_by_name("PlayerGun");
    }

    fn update(&mut self, ecs: &mut ECS, _scene: &str, _commands: &mut Vec<ScriptCommand>) {
        for event in ecs.drain_events::<ShotEvent>() {
            if let Some(hit) = event.hit {
                self.spawn_hit_burst(ecs, &hit);
                let mut processed = false;
                let damage = self
                    .gun_id
                    .and_then(|gun_id| ecs.attributes_component(gun_id))
                    .map(|attrs| attrs.get("damage").unwrap_or(1.0))
                    .unwrap_or(1.0);
                if let Some(target_attributes) = ecs.attributes_component_mut(hit.entity_id) {
                    processed = true;
                    let mut updated = target_attributes.get("health").unwrap_or(0.0);
                    updated -= damage;
                    target_attributes.set("health", updated);
                    self.last_message = Some(format!(
                        "Hit {}. Its health remaining is {}",
                        hit.entity_id, updated
                    ));
                    if updated <= 0.0 && Self::should_destroy(ecs, hit.entity_id) {
                        ecs.remove_entity(hit.entity_id);
                        self.destroyed += 1;
                        self.last_message = Some(format!(
                            "Destroyed {} at ({:.1}, {:.1}, {:.1})",
                            hit.entity_id, hit.point[0], hit.point[1], hit.point[2]
                        ));
                        continue;
                    }
                }

                if !processed {
                    if Self::should_destroy(ecs, hit.entity_id) {
                        ecs.remove_entity(hit.entity_id);
                        self.destroyed += 1;
                        self.last_message = Some(format!(
                            "Destroyed {} at ({:.1}, {:.1}, {:.1})",
                            hit.entity_id, hit.point[0], hit.point[1], hit.point[2]
                        ));
                    } else {
                        self.last_message =
                            Some(format!("Hit {} but it cannot be destroyed", hit.entity_id));
                    }
                }
            } else {
                self.last_message = Some("Shot missed...".to_string());
            }
            self.message_timer = 180;
        }
        if self.message_timer > 0 {
            self.message_timer -= 1;
            if self.message_timer == 0 {
                self.last_message = None;
            }
        }
    }

    fn hud_text(&mut self, ecs: &ECS, _scene: &str) -> Option<String> {
        let mut lines = Vec::new();
        if let Some(gun_id) = self.gun_id {
            if let Some(attributes) = ecs.attributes_component(gun_id) {
                let ammo = attributes.get("ammo").unwrap_or(0.0);
                lines.push(format!("Ammo: {}", ammo.max(0.0).floor() as i32));
            }
        }
        lines.push(format!("Targets destroyed: {}", self.destroyed));
        if let Some(message) = self.last_message.as_ref() {
            lines.push(message.clone());
        } else {
            lines.push("Left click to fire.".to_string());
        }
        Some(lines.join("\n"))
    }

    fn before_fire(&mut self, ecs: &mut ECS) -> bool {
        let Some(gun_id) = self.gun_id else {
            return true;
        };
        let Some(attributes) = ecs.attributes_component_mut(gun_id) else {
            return true;
        };
        let ammo = attributes.get("ammo").unwrap_or(0.0);
        if ammo <= 0.0 {
            self.last_message = Some("Out of ammo!".to_string());
            self.message_timer = 90;
            return false;
        }
        let updated = ammo - 1.0;
        attributes.set("ammo", updated);
        self.last_message = Some(format!(
            "Ammo remaining: {}",
            updated.max(0.0).floor() as i32
        ));
        self.message_timer = 60;
        true
    }
}
