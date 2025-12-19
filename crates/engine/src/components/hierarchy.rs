use glam::Vec3;

use super::{Orientation, Position};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct HierarchyComponent {
    pub parent: u32,
    pub local_position: Position,
    pub local_orientation: Orientation,
}

impl HierarchyComponent {
    pub const fn new(
        parent: u32,
        local_position: Position,
        local_orientation: Orientation,
    ) -> Self {
        Self {
            parent,
            local_position,
            local_orientation,
        }
    }

    pub fn from_world_transforms(
        parent: u32,
        parent_position: Position,
        parent_orientation: Orientation,
        child_position: Position,
        child_orientation: Orientation,
    ) -> Self {
        let parent_quat = parent_orientation.quat();
        let parent_vec = vec_from_position(parent_position);
        let child_vec = vec_from_position(child_position);
        let local_translation = parent_quat.conjugate() * (child_vec - parent_vec);
        let local_orientation =
            Orientation::from_quat(parent_quat.conjugate() * child_orientation.quat());
        Self {
            parent,
            local_position: Position::new(
                local_translation.x,
                local_translation.y,
                local_translation.z,
            ),
            local_orientation,
        }
    }

    pub fn compose_with_parent(
        &self,
        parent_position: Position,
        parent_orientation: Orientation,
    ) -> (Position, Orientation) {
        let parent_quat = parent_orientation.quat();
        let rotated_offset = parent_quat * vec_from_position(self.local_position);
        let world_position = Position::new(
            parent_position.x + rotated_offset.x,
            parent_position.y + rotated_offset.y,
            parent_position.z + rotated_offset.z,
        );
        let world_orientation = Orientation::from_quat(parent_quat * self.local_orientation.quat());
        (world_position, world_orientation)
    }

    pub fn set_local_position(&mut self, position: Position) {
        self.local_position = position;
    }

    pub fn set_local_orientation(&mut self, orientation: Orientation) {
        self.local_orientation = orientation;
    }
}

fn vec_from_position(position: Position) -> Vec3 {
    Vec3::new(position.x, position.y, position.z)
}
