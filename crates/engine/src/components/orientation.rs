use glam::{EulerRot, Quat};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Orientation {
    quat: Quat,
}

impl Orientation {
    pub const fn identity() -> Self {
        Self {
            quat: Quat::IDENTITY,
        }
    }

    pub fn from_quat(quat: Quat) -> Self {
        Self {
            quat: Self::normalize_or_identity(quat),
        }
    }

    pub fn from_yaw_pitch_roll(yaw: f32, pitch: f32, roll: f32) -> Self {
        let quat = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
        Self::from_quat(quat)
    }

    pub fn as_array(&self) -> [f32; 4] {
        self.quat.to_array()
    }

    pub fn quat(&self) -> Quat {
        self.quat
    }

    pub fn set_quat(&mut self, quat: Quat) {
        self.quat = Self::normalize_or_identity(quat);
    }

    fn normalize_or_identity(quat: Quat) -> Quat {
        if quat.length_squared() <= f32::EPSILON {
            Quat::IDENTITY
        } else {
            quat.normalize()
        }
    }
}

impl Default for Orientation {
    fn default() -> Self {
        Self::identity()
    }
}

impl From<Quat> for Orientation {
    fn from(value: Quat) -> Self {
        Self::from_quat(value)
    }
}
