use std::sync::Arc;

use glam::Quat;
use rust_game::app::{run_game, CustomSystem, GameConfig, ShotEvent};
use rust_game::components::{Orientation, PhysicsBodyType, Position};
use rust_game::ecs::ECS;
use rust_game::math::normalize_vec3;
use rust_game::modules;
use rust_game::scene::{
    CameraComponentDefinition, ComponentDefinition, EntityDefinition, InputComponentDefinition,
    LightComponentDefinition, ModelComponentDefinition, PhysicsComponentDefinition,
    RenderComponentDefinition, SceneDefinition, SceneLibrary, SceneSettings,
    ScriptComponentDefinition, TerrainComponentDefinition,
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
        background_sound: Some("assets/audio/background.wav".to_string()),
    });
    scene.add_entity(explorer_entity());
    scene.add_entity(player_gun());
    scene.add_entity(target());
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
    let mut entity = target_entity("Sun", position);
    entity.components.render = Some(RenderComponentDefinition {
        color: [1.0, 0.95, 0.6],
        size: 0.5,
    });
    entity.components.model = Some(ModelComponentDefinition {
        asset: "assets/cube.obj".to_string(),
    });
    entity.components.script =
        Some(ScriptComponentDefinition::new("spinner").with_param("speed", "0.75"));
    entity
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
}

impl ShootingSystem {
    fn new() -> Self {
        Self {
            destroyed: 0,
            last_message: None,
            message_timer: 0,
        }
    }

    fn should_destroy(ecs: &ECS, entity_id: u32) -> bool {
        let tags = ecs.tag_manager.tags_for_entity(entity_id);
        !tags
            .iter()
            .any(|tag| tag == "terrain" || tag == "player" || tag == "camera" || tag == "player_gun")
    }
}

impl CustomSystem for ShootingSystem {
    fn scene_loaded(&mut self, _ecs: &mut ECS, _scene: &str) {
        self.destroyed = 0;
        self.last_message = None;
        self.message_timer = 0;
    }

    fn update(&mut self, ecs: &mut ECS, _scene: &str, _commands: &mut Vec<ScriptCommand>) {
        for event in ecs.drain_events::<ShotEvent>() {
            if let Some(hit) = event.hit {
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

    fn hud_text(&mut self, _ecs: &ECS, _scene: &str) -> Option<String> {
        let mut lines = vec![format!("Targets destroyed: {}", self.destroyed)];
        if let Some(message) = self.last_message.as_ref() {
            lines.push(message.clone());
        } else {
            lines.push("Left click to fire.".to_string());
        }
        Some(lines.join("\n"))
    }
}
