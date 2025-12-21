use std::sync::Arc;

use glam::Quat;
use rust_game::app::{run_game, CustomSystem, GameConfig, ScriptBinding, ShotEvent, ShotEventHit};
use rust_game::components::{Orientation, ParticleBurstRequest, Position};
use rust_game::ecs::ECS;
use rust_game::modules;
use rust_game::scene::{
    load_scene_from_yaml, ComponentDefinition, EntityDefinition, PhysicsComponentDefinition,
    RenderComponentDefinition, SceneDefinition, SceneLibrary,
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

    let mut scene = load_scene_from_yaml("assets/labyrinth_scene.yml")?;
    spawn_random_trees(&mut scene);
    let scenes = SceneLibrary::new().with_scene(LABYRINTH_SCENE_ID, scene);
    let script_registry = build_script_registry();

    let config = GameConfig::new(LABYRINTH_SCENE_ID, scenes, script_registry)
        .with_window_title("My Rust Game")
        .with_script_binding(
            ScriptBinding::new("target", "spinner").with_param("speed", "0.75"),
        )
        .with_custom_system(ShootingSystem::new());

    run_game(config)
}

fn spawn_random_trees(scene: &mut SceneDefinition) {
    let terrain_size = find_terrain_size(scene).unwrap_or(22.0);
    let half = terrain_size * 0.5;
    let mut rng = LcgRng::new(42);
    for index in 0..50 {
        let x = rng.next_range(-half, half);
        let z = rng.next_range(-half, half);
        let yaw = rng.next_range(0.0, std::f32::consts::TAU);
        let name = format!("Tree_{:02}", index + 1);
        let tree = EntityDefinition::new(
            name,
            Position {
                x,
                y: 0.0,
                z,
            },
        )
        .with_tags(["tree"])
        .with_orientation(Orientation::from_yaw_pitch_roll(yaw, 0.0, 0.0))
        .with_components(ComponentDefinition {
            render: Some(RenderComponentDefinition {
                color: [0.3, 0.6, 0.2],
                size: 1.0,
            }),
            model: Some(rust_game::scene::ModelComponentDefinition {
                asset: "assets/tree.obj".to_string(),
            }),
            physics: Some(PhysicsComponentDefinition {
                body_type: rust_game::components::PhysicsBodyType::Static,
                half_extents: Some([0.1, 2.0, 0.1]),
                ..Default::default()
            }),
            ..Default::default()
        });
        scene.add_entity(tree);
    }
}

fn find_terrain_size(scene: &SceneDefinition) -> Option<f32> {
    scene
        .entities
        .iter()
        .find_map(|entity| entity.components.terrain.as_ref().map(|terrain| terrain.size))
}

struct LcgRng {
    state: u32,
}

impl LcgRng {
    const A: u32 = 1664525;
    const C: u32 = 1013904223;

    fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(Self::A).wrapping_add(Self::C);
        self.state
    }

    fn next_f32(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }

    fn next_range(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.next_f32()
    }
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
            tag == "terrain" || tag == "player" || tag == "camera" || tag == "player_gun" || tag == "tree"
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
