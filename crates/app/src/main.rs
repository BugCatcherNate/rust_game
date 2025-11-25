use std::collections::HashSet;
use std::sync::Arc;

use rust_game::app::{run_game, CustomSystem, GameConfig};
use rust_game::components::{PhysicsBodyType, Position};
use rust_game::ecs::ECS;
use rust_game::modules;
use rust_game::scene::{
    CameraComponentDefinition, ComponentDefinition, EntityDefinition, InputComponentDefinition,
    LightComponentDefinition, ModelComponentDefinition, PhysicsComponentDefinition,
    RenderComponentDefinition, SceneDefinition, SceneLibrary, SceneSettings,
    TerrainComponentDefinition, TextureComponentDefinition,
};
use rust_game::scripts::{ScriptCommand, ScriptRegistry};

const LABYRINTH_SCENE_ID: &str = "labyrinth";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    modules::core::initialize();

    let scenes = SceneLibrary::new().with_scene(LABYRINTH_SCENE_ID, labyrinth_scene());

    let config = GameConfig::new(LABYRINTH_SCENE_ID, scenes, Arc::new(ScriptRegistry::new()))
        .with_window_title("My Rust Game")
        .with_custom_system(CollectibleSystem::new())
        .with_custom_system(LabyrinthSystem::new());

    run_game(config)
}

fn labyrinth_scene() -> SceneDefinition {
    let mut scene = SceneDefinition::new(SceneSettings {
        background_top: [0.08, 0.11, 0.19],
        background_bottom: [0.01, 0.01, 0.03],
        fog_color: [0.04, 0.07, 0.12],
        fog_density: 0.32,
        background_sound: Some("assets/audio/background.wav".to_string()),
    });
    scene.add_entity(explorer_entity());
    scene.add_entity(labyrinth_floor());
    scene.add_entity(exit_obelisk());
    scene.entities.extend(key_entities());
    scene.add_entity(hazard_crystal());
    scene.entities.extend(patrol_orbs());
    scene.entities.extend(light_sources());
    scene.entities.extend(wall_entities());
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

fn labyrinth_floor() -> EntityDefinition {
    let mut terrain = TerrainComponentDefinition::default();
    terrain.size = 22.0;
    terrain.height = 0.3;
    terrain.color = [0.12, 0.12, 0.12];
    terrain.texture = Some("assets/textures/ground.png".to_string());
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
            half_extents: Some([
                terrain_size * 0.5,
                terrain_height * 0.5,
                terrain_size * 0.5,
            ]),
            ..Default::default()
        }),
        ..Default::default()
    })
}

fn exit_obelisk() -> EntityDefinition {
    cube_with_texture(
        "ExitObelisk",
        Position {
            x: 0.0,
            y: 0.5,
            z: -7.0,
        },
        &["exit"],
        [0.2, 0.8, 1.0],
        0.8,
        "assets/textures/blue.png",
    )
    .with_physics(auto_box_physics(PhysicsBodyType::Static))
}

fn key_entities() -> Vec<EntityDefinition> {
    vec![
        cube_with_texture(
            "KeyAlpha",
            Position {
                x: 6.0,
                y: 0.4,
                z: 4.0,
            },
            &["collectible", "key"],
            [1.0, 0.8, 0.1],
            0.5,
            "assets/textures/blue.png",
        ),
        cube_with_texture(
            "KeyBeta",
            Position {
                x: -6.0,
                y: 0.4,
                z: 0.0,
            },
            &["collectible", "key"],
            [0.9, 0.3, 0.8],
            0.5,
            "assets/textures/red.png",
        ),
        cube_with_texture(
            "KeyGamma",
            Position {
                x: 2.0,
                y: 0.4,
                z: -6.0,
            },
            &["collectible", "key"],
            [0.3, 0.9, 0.6],
            0.5,
            "assets/textures/ground.png",
        ),
    ]
}

fn hazard_crystal() -> EntityDefinition {
    cube_with_texture(
        "HazardCrystal",
        Position {
            x: 0.0,
            y: 0.3,
            z: 0.0,
        },
        &["prop"],
        [1.0, 0.25, 0.25],
        0.6,
        "assets/textures/red.png",
    )
}

fn patrol_orbs() -> Vec<EntityDefinition> {
    vec![
        patrol_orb(
            "PatrolOrbNorth",
            Position {
                x: -3.0,
                y: 1.0,
                z: 3.0,
            },
            [0.4, 0.7, 1.0],
        ),
        patrol_orb(
            "PatrolOrbSouth",
            Position {
                x: 3.0,
                y: 1.0,
                z: -3.5,
            },
            [0.7, 0.4, 1.0],
        ),
    ]
}

fn patrol_orb(name: &str, position: Position, color: [f32; 3]) -> EntityDefinition {
    EntityDefinition::new(name, position).with_components(ComponentDefinition {
        render: Some(RenderComponentDefinition { color, size: 0.35 }),
        model: Some(ModelComponentDefinition {
            asset: "assets/cube.obj".to_string(),
        }),
        texture: Some(TextureComponentDefinition {
            asset: "assets/textures/blue.png".to_string(),
        }),
        light: Some(LightComponentDefinition {
            direction: [0.0, -1.0, 0.0],
            color: [
                (color[0] + 0.5).min(1.0),
                (color[1] + 0.5).min(1.0),
                (color[2] + 0.5).min(1.0),
            ],
            intensity: 10.0,
            point_radius: Some(4.0),
        }),
        ..Default::default()
    })
}

fn light_sources() -> Vec<EntityDefinition> {
    vec![
        directional_light(
            "Sun",
            Position {
                x: 5.0,
                y: 8.0,
                z: 5.0,
            },
            [-0.3, -1.0, -0.2],
            [1.0, 0.95, 0.85],
            3.5,
        ),
        directional_light(
            "AccentLight",
            Position {
                x: -4.0,
                y: 6.0,
                z: -4.0,
            },
            [0.1, -1.0, 0.2],
            [0.5, 0.7, 1.0],
            2.0,
        ),
    ]
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

fn wall_entities() -> Vec<EntityDefinition> {
    const POSITIONS: &[(i32, i32)] = &[
        (-6, 6),
        (-4, 6),
        (2, 6),
        (4, 6),
        (6, 6),
        (-6, 4),
        (-4, 4),
        (-6, 2),
        (4, 2),
        (6, 2),
        (0, 0),
        (2, 0),
        (4, 0),
        (6, 0),
        (-6, -2),
        (-4, -2),
        (6, -2),
        (-6, -4),
        (-4, -4),
        (-2, -4),
        (6, -4),
        (-6, -6),
        (-4, -6),
        (-2, -6),
        (4, -6),
        (6, -6),
    ];
    POSITIONS
        .iter()
        .map(|&(x, z)| {
            cube_with_texture(
                &format!("Wall_X{}_Z{}", x, z),
                Position {
                    x: x as f32,
                    y: 0.6,
                    z: z as f32,
                },
                &["wall"],
                [0.28, 0.27, 0.25],
                1.2,
                "assets/textures/ground.png",
            )
            .with_physics(box_physics(1.2, PhysicsBodyType::Static))
        })
        .collect()
}

fn cube_with_texture(
    name: impl Into<String>,
    position: Position,
    tags: &[&str],
    color: [f32; 3],
    size: f32,
    texture: &str,
) -> EntityDefinition {
    EntityDefinition::new(name, position)
        .with_tags(tags.iter().copied())
        .with_components(ComponentDefinition {
            render: Some(RenderComponentDefinition { color, size }),
            model: Some(ModelComponentDefinition {
                asset: "assets/cube.obj".to_string(),
            }),
            texture: Some(TextureComponentDefinition {
                asset: texture.to_string(),
            }),
            ..Default::default()
        })
}

trait PhysicsExt {
    fn with_physics(self, physics: PhysicsComponentDefinition) -> Self;
}

impl PhysicsExt for EntityDefinition {
    fn with_physics(mut self, physics: PhysicsComponentDefinition) -> Self {
        self.components.physics = Some(physics);
        self
    }
}

fn box_physics(size: f32, body_type: PhysicsBodyType) -> PhysicsComponentDefinition {
    PhysicsComponentDefinition {
        body_type,
        half_extents: Some([size * 0.5, size * 0.5, size * 0.5]),
        ..Default::default()
    }
}

fn auto_box_physics(body_type: PhysicsBodyType) -> PhysicsComponentDefinition {
    PhysicsComponentDefinition {
        body_type,
        half_extents: None,
        ..Default::default()
    }
}

struct LabyrinthSystem {
    exit_tag: &'static str,
    key_tag: &'static str,
    exit_radius_sq: f32,
    keys: Vec<String>,
    collected: HashSet<String>,
    exit_unlocked: bool,
    escaped: bool,
    message: Option<String>,
}

impl LabyrinthSystem {
    fn new() -> Self {
        Self {
            exit_tag: "exit",
            key_tag: "key",
            exit_radius_sq: 1.0,
            keys: Vec::new(),
            collected: HashSet::new(),
            exit_unlocked: false,
            escaped: false,
            message: None,
        }
    }

    fn rebuild_keys(&mut self, ecs: &ECS) {
        self.keys.clear();
        if let Some(entities) = ecs.tag_manager.get_entities_with_tag(self.key_tag) {
            for id in entities {
                if let Some((_pos, name)) = ecs.find_entity_components(*id) {
                    self.keys.push(name.0.clone());
                }
            }
        }
        self.collected.clear();
        self.exit_unlocked = self.keys.is_empty();
        self.escaped = false;
        self.message = None;
    }

    fn sync_key_progress(&mut self, ecs: &ECS) {
        for name in self.keys.iter() {
            if self.collected.contains(name) {
                continue;
            }
            if ecs.find_entity_id_by_name(name).is_none() {
                self.collected.insert(name.clone());
                self.message = Some(format!("Recovered {name}"));
            }
        }
        if !self.exit_unlocked && self.collected.len() == self.keys.len() {
            self.exit_unlocked = true;
            self.message = Some("All keys recovered. Exit unlocked!".to_string());
        }
    }

    fn player_position(&self, ecs: &ECS) -> Option<Position> {
        let player_id = ecs
            .tag_manager
            .get_entities_with_tag("player")
            .and_then(|set| set.iter().next().copied())?;
        ecs.find_entity_components(player_id)
            .map(|(position, _)| position.clone())
    }

    fn exit_position(&self, ecs: &ECS) -> Option<Position> {
        let exit_id = ecs
            .tag_manager
            .get_entities_with_tag(self.exit_tag)
            .and_then(|set| set.iter().next().copied())?;
        ecs.find_entity_components(exit_id)
            .map(|(position, _)| position.clone())
    }
}

impl CustomSystem for LabyrinthSystem {
    fn scene_loaded(&mut self, ecs: &mut ECS, _scene: &str) {
        self.rebuild_keys(ecs);
    }

    fn update(&mut self, ecs: &mut ECS, _current_scene: &str, _commands: &mut Vec<ScriptCommand>) {
        if self.escaped {
            return;
        }
        let Some(player_pos) = self.player_position(ecs) else {
            return;
        };
        self.sync_key_progress(ecs);
        if !self.exit_unlocked {
            return;
        }
        let Some(exit_pos) = self.exit_position(ecs) else {
            return;
        };
        let dx = player_pos.x - exit_pos.x;
        let dy = player_pos.y - exit_pos.y;
        let dz = player_pos.z - exit_pos.z;
        if dx * dx + dy * dy + dz * dz <= self.exit_radius_sq {
            self.escaped = true;
            self.message = Some("You escaped the labyrinth!".to_string());
        }
    }

    fn hud_text(&mut self, _ecs: &ECS, _scene: &str) -> Option<String> {
        let total = self.keys.len();
        if total == 0 && self.message.is_none() {
            return None;
        }
        let remaining = total.saturating_sub(self.collected.len());
        let mut lines = vec![format!("Keys: {}/{}", self.collected.len(), total)];
        if self.escaped {
            lines.push("Objective complete: exit reached.".to_string());
        } else if self.exit_unlocked {
            lines.push("Exit unlocked. Find the glowing monolith.".to_string());
        } else {
            lines.push(format!("{remaining} key(s) remain hidden."));
        }
        if let Some(msg) = self.message.take() {
            lines.push(msg);
        }
        Some(lines.join("\n"))
    }
}

struct CollectibleSystem {
    tag: &'static str,
    radius_sq: f32,
    collectibles: Vec<String>,
    collected: HashSet<String>,
    message: Option<String>,
}

impl CollectibleSystem {
    fn new() -> Self {
        Self {
            tag: "collectible",
            radius_sq: 1.0,
            collectibles: Vec::new(),
            collected: HashSet::new(),
            message: None,
        }
    }

    fn rebuild(&mut self, ecs: &ECS) {
        self.collectibles.clear();
        if let Some(entities) = ecs.tag_manager.get_entities_with_tag(self.tag) {
            for id in entities {
                if let Some((_pos, name)) = ecs.find_entity_components(*id) {
                    self.collectibles.push(name.0.clone());
                }
            }
        }
        self.collected.clear();
        self.message = None;
    }

    fn player_position(&self, ecs: &ECS) -> Option<Position> {
        let player_id = ecs
            .tag_manager
            .get_entities_with_tag("player")
            .and_then(|set| set.iter().next().copied())?;
        ecs.find_entity_components(player_id)
            .map(|(position, _)| position.clone())
    }

    fn entity_position(&self, ecs: &ECS, name: &str) -> Option<Position> {
        let entity_id = ecs.find_entity_id_by_name(name)?;
        ecs.find_entity_components(entity_id)
            .map(|(position, _)| position.clone())
    }
}

impl CustomSystem for CollectibleSystem {
    fn scene_loaded(&mut self, ecs: &mut ECS, _scene: &str) {
        self.rebuild(ecs);
    }

    fn update(&mut self, ecs: &mut ECS, _current_scene: &str, _commands: &mut Vec<ScriptCommand>) {
        let Some(player_pos) = self.player_position(ecs) else {
            return;
        };
        self.message = None;
        for name in self.collectibles.iter().cloned() {
            if self.collected.contains(&name) {
                continue;
            }
            let Some(item_pos) = self.entity_position(ecs, &name) else {
                continue;
            };
            let dx = player_pos.x - item_pos.x;
            let dy = player_pos.y - item_pos.y;
            let dz = player_pos.z - item_pos.z;
            if dx * dx + dy * dy + dz * dz <= self.radius_sq {
                if let Some(entity_id) = ecs.find_entity_id_by_name(&name) {
                    ecs.remove_entity(entity_id);
                }
                self.collected.insert(name.clone());
                self.message = Some(format!("Collected {name}!"));
            }
        }
    }

    fn hud_text(&mut self, _ecs: &ECS, _scene: &str) -> Option<String> {
        if self.collectibles.is_empty() {
            return None;
        }
        let mut lines = vec![format!(
            "Collectibles: {}/{}",
            self.collected.len(),
            self.collectibles.len()
        )];
        if self.collected.len() == self.collectibles.len() {
            lines.push("All collectibles gathered! Explore freely.".to_string());
        } else if let Some(msg) = &self.message {
            lines.push(msg.clone());
        }
        Some(lines.join("\n"))
    }
}
