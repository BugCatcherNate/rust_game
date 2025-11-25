use std::collections::{HashMap, HashSet};

use rapier3d::prelude::*;

use crate::archetypes::Archetype;
use crate::components::{PhysicsBodyType, PhysicsComponent, Position};
use crate::ecs::ECS;

pub struct PhysicsSystem {
    pipeline: PhysicsPipeline,
    gravity: Vector<Real>,
    integration_parameters: IntegrationParameters,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    bodies: RigidBodySet,
    colliders: ColliderSet,
    ccd_solver: CCDSolver,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    query_pipeline: QueryPipeline,
    entity_to_body: HashMap<u32, RigidBodyHandle>,
}

impl PhysicsSystem {
    const JUMP_SPEED: f32 = 5.5;
    pub fn new() -> Self {
        Self {
            pipeline: PhysicsPipeline::new(),
            gravity: vector![0.0, -9.81, 0.0],
            integration_parameters: IntegrationParameters {
                dt: 1.0 / 60.0,
                ..IntegrationParameters::default()
            },
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            ccd_solver: CCDSolver::new(),
            impulse_joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            query_pipeline: QueryPipeline::new(),
            entity_to_body: HashMap::new(),
        }
    }

    pub fn rebuild_from_ecs(&mut self, ecs: &mut ECS) {
        self.clear_world();
        self.ensure_bodies_for_ecs(ecs);
        self.sync_positions_from_bodies(ecs);
        self.update_ground_contacts(ecs);
    }

    pub fn update(&mut self, ecs: &mut ECS) {
        let active_entities = Self::collect_active_entities(&ecs.archetypes);
        self.remove_stale_bodies(&active_entities);
        self.ensure_bodies_for_ecs(ecs);
        self.apply_input_velocity(ecs);
        self.step_world();
        self.sync_positions_from_bodies(ecs);
        self.update_ground_contacts(ecs);
    }

    fn clear_world(&mut self) {
        self.island_manager = IslandManager::new();
        self.broad_phase = DefaultBroadPhase::new();
        self.narrow_phase = NarrowPhase::new();
        self.bodies = RigidBodySet::new();
        self.colliders = ColliderSet::new();
        self.ccd_solver = CCDSolver::new();
        self.impulse_joints = ImpulseJointSet::new();
        self.multibody_joints = MultibodyJointSet::new();
        self.query_pipeline = QueryPipeline::new();
        self.entity_to_body.clear();
    }

    fn collect_active_entities(archetypes: &[Archetype]) -> HashSet<u32> {
        let mut active = HashSet::new();
        for archetype in archetypes {
            if archetype.physics.is_none() {
                continue;
            }
            for entity_id in &archetype.entity_ids {
                active.insert(*entity_id);
            }
        }
        active
    }

    fn remove_stale_bodies(&mut self, active_entities: &HashSet<u32>) {
        let stale: Vec<u32> = self
            .entity_to_body
            .keys()
            .copied()
            .filter(|entity| !active_entities.contains(entity))
            .collect();

        for entity_id in stale {
            if let Some(handle) = self.entity_to_body.remove(&entity_id) {
                if self.bodies.contains(handle) {
                    self.bodies.remove(
                        handle,
                        &mut self.island_manager,
                        &mut self.colliders,
                        &mut self.impulse_joints,
                        &mut self.multibody_joints,
                        true,
                    );
                }
            }
        }
    }

    fn ensure_bodies_for_ecs(&mut self, ecs: &mut ECS) {
        for archetype in &mut ecs.archetypes {
            let Some(_) = archetype.physics.as_mut() else {
                continue;
            };

            let len = archetype.len();
            for index in 0..len {
                let entity_id = archetype.entity_ids[index];
                let position = archetype.positions[index].clone();
                let Some(physics) = archetype
                    .physics
                    .as_mut()
                    .and_then(|vec| vec.get_mut(index))
                else {
                    continue;
                };
                self.ensure_body(entity_id, &position, physics);
            }
        }
    }

    fn ensure_body(&mut self, entity_id: u32, position: &Position, physics: &mut PhysicsComponent) {
        if let Some(handle) = physics.body_handle {
            if self.bodies.contains(handle) {
                self.entity_to_body.insert(entity_id, handle);
                return;
            }
            physics.body_handle = None;
            physics.collider_handle = None;
        }

        let body_handle = self.bodies.insert(physics.build_body(position));
        let collider = physics.build_collider();
        let collider_handle =
            self.colliders
                .insert_with_parent(collider, body_handle, &mut self.bodies);

        physics.body_handle = Some(body_handle);
        physics.collider_handle = Some(collider_handle);
        self.entity_to_body.insert(entity_id, body_handle);
    }

    fn apply_input_velocity(&mut self, ecs: &mut ECS) {
        for archetype in &mut ecs.archetypes {
            let len = archetype.len();
            let (Some(physics), Some(inputs)) =
                (archetype.physics.as_mut(), archetype.inputs.as_mut())
            else {
                continue;
            };

            for index in 0..len {
                let component = &mut physics[index];
                let Some(body_handle) = component.body_handle else {
                    continue;
                };
                let Some(body) = self.bodies.get_mut(body_handle) else {
                    continue;
                };
                let input = &mut inputs[index];

                match component.body_type {
                    PhysicsBodyType::Dynamic => {
                        let jump = component.grounded && input.take_jump_request();
                        let dir = input.direction;
                        let speed_per_sec = input.speed / self.integration_parameters.dt;
                        let current_y = body.linvel().y;
                        let target_y = if jump {
                            Self::JUMP_SPEED
                        } else if dir[1].abs() > f32::EPSILON {
                            dir[1] * speed_per_sec
                        } else {
                            current_y
                        };
                        let velocity = Vector::new(
                            dir[0] * speed_per_sec,
                            target_y,
                            dir[2] * speed_per_sec,
                        );
                        body.set_linvel(velocity, true);
                    }
                    PhysicsBodyType::Kinematic => {
                        let dir = inputs[index].direction;
                        if dir == [0.0, 0.0, 0.0] {
                            continue;
                        }
                        let current = body.translation();
                        let step = Vector::new(
                            dir[0] * inputs[index].speed,
                            dir[1] * inputs[index].speed,
                            dir[2] * inputs[index].speed,
                        );
                        body.set_next_kinematic_translation(current + step);
                    }
                    PhysicsBodyType::Static => {}
                }
            }
        }
    }

    fn update_ground_contacts(&mut self, ecs: &mut ECS) {
        for archetype in &mut ecs.archetypes {
            let Some(physics_vec) = archetype.physics.as_mut() else {
                continue;
            };
            for component in physics_vec.iter_mut() {
                let Some(collider_handle) = component.collider_handle else {
                    component.grounded = false;
                    continue;
                };
                let mut grounded = false;
                for pair in self.narrow_phase.contact_pairs_with(collider_handle) {
                    if !pair.has_any_active_contact {
                        continue;
                    }
                    for manifold in &pair.manifolds {
                        let mut normal = manifold.data.normal;
                        if pair.collider2 == collider_handle {
                            normal = -normal;
                        }
                        if normal.y > 0.5 {
                            grounded = true;
                            break;
                        }
                    }
                    if grounded {
                        break;
                    }
                }
                component.grounded = grounded;
            }
        }
    }

    fn step_world(&mut self) {
        self.pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &(),
            &(),
        );
    }

    fn sync_positions_from_bodies(&mut self, ecs: &mut ECS) {
        for archetype in &mut ecs.archetypes {
            let Some(physics) = archetype.physics.as_ref() else {
                continue;
            };

            for index in 0..archetype.len() {
                let Some(body_handle) = physics[index].body_handle else {
                    continue;
                };
                if let Some(body) = self.bodies.get(body_handle) {
                    let translation = body.translation();
                    let pos = &mut archetype.positions[index];
                    pos.x = translation.x;
                    pos.y = translation.y;
                    pos.z = translation.z;
                }
            }
        }
    }
}
