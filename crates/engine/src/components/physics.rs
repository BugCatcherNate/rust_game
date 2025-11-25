use rapier3d::prelude::{
    Collider, ColliderBuilder, ColliderHandle, RigidBody, RigidBodyBuilder, RigidBodyHandle, Vector,
};

use super::Position;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicsBodyType {
    Dynamic,
    Static,
    Kinematic,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhysicsComponent {
    pub body_type: PhysicsBodyType,
    pub half_extents: [f32; 3],
    pub restitution: f32,
    pub friction: f32,
    pub body_handle: Option<RigidBodyHandle>,
    pub collider_handle: Option<ColliderHandle>,
}

impl PhysicsComponent {
    pub fn new(body_type: PhysicsBodyType, half_extents: [f32; 3]) -> Self {
        Self {
            body_type,
            half_extents,
            restitution: 0.2,
            friction: 0.7,
            body_handle: None,
            collider_handle: None,
        }
    }

    pub fn dynamic_box(half_extents: [f32; 3]) -> Self {
        Self::new(PhysicsBodyType::Dynamic, half_extents)
    }

    pub fn fixed_box(half_extents: [f32; 3]) -> Self {
        Self::new(PhysicsBodyType::Static, half_extents)
    }

    pub(crate) fn build_body(&self, position: &Position) -> RigidBody {
        let mut builder = match self.body_type {
            PhysicsBodyType::Dynamic => RigidBodyBuilder::dynamic(),
            PhysicsBodyType::Static => RigidBodyBuilder::fixed(),
            PhysicsBodyType::Kinematic => RigidBodyBuilder::kinematic_position_based(),
        };
        if matches!(self.body_type, PhysicsBodyType::Dynamic) {
            builder = builder.lock_rotations();
        }
        builder = builder.translation(Vector::new(position.x, position.y, position.z));
        builder.build()
    }

    pub(crate) fn build_collider(&self) -> Collider {
        ColliderBuilder::cuboid(
            self.half_extents[0],
            self.half_extents[1],
            self.half_extents[2],
        )
        .restitution(self.restitution)
        .friction(self.friction)
        .build()
    }
}
