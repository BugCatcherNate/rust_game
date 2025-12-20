use std::collections::HashMap;

use glam::{EulerRot, Mat3, Quat, Vec3};

use crate::components::{
    ModelComponent, Name, Orientation, ParticleBurstRequest, ParticleComponent,
    ParticleEmitterComponent, Position, RenderComponent, TextureComponent,
};
use crate::ecs::ECS;

pub struct ParticleSystem;

struct ParticleSpawn {
    emitter_id: u32,
    position: Position,
    velocity: [f32; 3],
    lifetime: f32,
    size: f32,
    color: [f32; 3],
    model_asset: String,
    texture_asset: Option<String>,
}

impl ParticleSystem {
    const DT: f32 = 1.0 / 60.0;

    pub fn update(ecs: &mut ECS, camera_entity: Option<u32>) {
        let camera_pos = camera_entity
            .and_then(|id| ecs.find_entity_components(id).map(|(pos, _, _)| *pos))
            .map(|pos| Vec3::from_array(pos.as_array()));
        let mut to_remove = Vec::new();
        let mut counts: HashMap<u32, usize> = HashMap::new();
        let burst_requests = ecs.drain_events::<ParticleBurstRequest>();

        for archetype in &mut ecs.archetypes {
            let len = archetype.entity_ids.len();
            let Some(particles) = archetype.particles.as_mut() else {
                continue;
            };
            for index in 0..len {
                let particle = &mut particles[index];
                particle.age += Self::DT;
                let velocity = Vec3::from_array(particle.velocity);
                let pos = &mut archetype.positions[index];
                pos.x += velocity.x * Self::DT;
                pos.y += velocity.y * Self::DT;
                pos.z += velocity.z * Self::DT;
                if let Some(camera_pos) = camera_pos {
                    let particle_pos = Vec3::new(pos.x, pos.y, pos.z);
                    archetype.orientations[index] =
                        Orientation::from_quat(Self::billboard_quat(particle_pos, camera_pos));
                }

                if particle.age >= particle.lifetime {
                    to_remove.push(archetype.entity_ids[index]);
                } else {
                    *counts.entry(particle.emitter_id).or_insert(0) += 1;
                }
            }
        }

        for entity_id in to_remove {
            ecs.remove_entity(entity_id);
        }

        let mut spawns = Vec::new();
        for request in burst_requests {
            let mut seed = request.seed;
            for _ in 0..request.count {
                let velocity = Self::random_burst_velocity(
                    request.direction,
                    request.spread,
                    request.speed,
                    &mut seed,
                );
                let color =
                    Self::jitter_color_values(request.color, request.color_jitter, &mut seed);
                let size = Self::jitter_size_value(request.size, request.size_jitter, &mut seed);
                spawns.push(ParticleSpawn {
                    emitter_id: 0,
                    position: Position::from_array(request.position),
                    velocity,
                    lifetime: request.lifetime,
                    size,
                    color,
                    model_asset: request.model_asset.clone(),
                    texture_asset: request.texture_asset.clone(),
                });
            }
        }
        for archetype in &mut ecs.archetypes {
            let len = archetype.entity_ids.len();
            let Some(emitters) = archetype.particle_emitters.as_mut() else {
                continue;
            };
            for index in 0..len {
                let entity_id = archetype.entity_ids[index];
                let emitter = &mut emitters[index];
                let current = *counts.get(&entity_id).unwrap_or(&0);
                if current >= emitter.max_particles {
                    continue;
                }

                emitter.spawn_accumulator += emitter.rate * Self::DT;
                let mut spawn_count = emitter.spawn_accumulator.floor() as usize;
                if spawn_count == 0 {
                    continue;
                }
                emitter.spawn_accumulator -= spawn_count as f32;
                let available = emitter.max_particles - current;
                if spawn_count > available {
                    spawn_count = available;
                }

                let position = archetype.positions[index];
                let orientation = archetype.orientations[index];
                for _ in 0..spawn_count {
                    let velocity = Self::random_velocity(emitter, orientation);
                    let color = Self::jitter_color(emitter);
                    let size = Self::jitter_size(emitter);
                    spawns.push(ParticleSpawn {
                        emitter_id: entity_id,
                        position,
                        velocity,
                        lifetime: emitter.lifetime,
                        size,
                        color,
                        model_asset: emitter.model_asset.clone(),
                        texture_asset: emitter.texture_asset.clone(),
                    });
                }
            }
        }

        for spawn in spawns {
            let particle_id = ecs.add_entity(
                spawn.position,
                Orientation::identity(),
                Name("Particle".to_string()),
            );
            ecs.add_render_component(
                particle_id,
                RenderComponent::new(spawn.color, spawn.size),
            );
            ecs.add_model_component(particle_id, ModelComponent::new(spawn.model_asset));
            if let Some(texture_asset) = spawn.texture_asset {
                ecs.add_texture_component(particle_id, TextureComponent::new(texture_asset));
            }
            ecs.add_particle_component(
                particle_id,
                ParticleComponent::new(spawn.emitter_id, spawn.velocity, spawn.lifetime),
            );
            ecs.tag_manager.add_tag(particle_id, "particle");
        }
    }

    fn random_velocity(
        emitter: &mut ParticleEmitterComponent,
        orientation: Orientation,
    ) -> [f32; 3] {
        let yaw = (emitter.next_unit_random() * 2.0 - 1.0) * emitter.spread;
        let pitch = (emitter.next_unit_random() * 2.0 - 1.0) * emitter.spread;
        let rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
        let local_dir = rotation * Vec3::new(0.0, 0.0, -1.0);
        let mut forward = Vec3::from_array(emitter.direction);
        if forward.length_squared() <= f32::EPSILON {
            forward = Vec3::new(0.0, 0.0, -1.0);
        }
        let forward = (orientation.quat() * forward).normalize_or_zero();
        let up_ref = if forward.dot(Vec3::Y).abs() > 0.95 {
            Vec3::X
        } else {
            Vec3::Y
        };
        let right = forward.cross(up_ref).normalize_or_zero();
        let up = right.cross(forward).normalize_or_zero();
        let world_dir = (right * local_dir.x + up * local_dir.y + forward * local_dir.z)
            .normalize_or_zero();
        (world_dir * emitter.speed).to_array()
    }

    fn jitter_color(emitter: &mut ParticleEmitterComponent) -> [f32; 3] {
        Self::jitter_color_values(emitter.color, emitter.color_jitter, &mut emitter.seed)
    }

    fn jitter_size(emitter: &mut ParticleEmitterComponent) -> f32 {
        Self::jitter_size_value(emitter.size, emitter.size_jitter, &mut emitter.seed)
    }

    fn jitter_color_values(color: [f32; 3], jitter: f32, seed: &mut u32) -> [f32; 3] {
        if jitter <= 0.0 {
            return color;
        }
        let r = color[0] + (Self::next_unit_random(seed) * 2.0 - 1.0) * jitter;
        let g = color[1] + (Self::next_unit_random(seed) * 2.0 - 1.0) * jitter;
        let b = color[2] + (Self::next_unit_random(seed) * 2.0 - 1.0) * jitter;
        [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)]
    }

    fn jitter_size_value(size: f32, jitter: f32, seed: &mut u32) -> f32 {
        if jitter <= 0.0 {
            return size;
        }
        let value = size + (Self::next_unit_random(seed) * 2.0 - 1.0) * jitter;
        value.max(0.01)
    }

    fn next_unit_random(seed: &mut u32) -> f32 {
        *seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (*seed as f32 / u32::MAX as f32).clamp(0.0, 1.0)
    }

    fn random_burst_velocity(
        direction: [f32; 3],
        spread: f32,
        speed: f32,
        seed: &mut u32,
    ) -> [f32; 3] {
        let yaw = (Self::next_unit_random(seed) * 2.0 - 1.0) * spread;
        let pitch = (Self::next_unit_random(seed) * 2.0 - 1.0) * spread;
        let rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
        let local_dir = rotation * Vec3::new(0.0, 0.0, -1.0);
        let mut forward = Vec3::from_array(direction);
        if forward.length_squared() <= f32::EPSILON {
            forward = Vec3::new(0.0, 0.0, -1.0);
        }
        let forward = forward.normalize_or_zero();
        let up_ref = if forward.dot(Vec3::Y).abs() > 0.95 {
            Vec3::X
        } else {
            Vec3::Y
        };
        let right = forward.cross(up_ref).normalize_or_zero();
        let up = right.cross(forward).normalize_or_zero();
        let world_dir = (right * local_dir.x + up * local_dir.y + forward * local_dir.z)
            .normalize_or_zero();
        (world_dir * speed).to_array()
    }

    fn billboard_quat(position: Vec3, camera_pos: Vec3) -> Quat {
        let mut forward = (camera_pos - position).normalize_or_zero();
        if forward.length_squared() <= f32::EPSILON {
            forward = Vec3::Z;
        }
        let up_ref = if forward.dot(Vec3::Y).abs() > 0.95 {
            Vec3::X
        } else {
            Vec3::Y
        };
        let right = up_ref.cross(forward).normalize_or_zero();
        let up = forward.cross(right).normalize_or_zero();
        let basis = Mat3::from_cols(right, up, forward);
        Quat::from_mat3(&basis)
    }
}
