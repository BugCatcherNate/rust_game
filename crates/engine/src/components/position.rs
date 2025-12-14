use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub const fn from_array(coords: [f32; 3]) -> Self {
        Self {
            x: coords[0],
            y: coords[1],
            z: coords[2],
        }
    }

    pub const fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub fn as_array(&self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }

    pub fn distance_to(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::zero()
    }
}

impl From<[f32; 3]> for Position {
    fn from(coords: [f32; 3]) -> Self {
        Self::from_array(coords)
    }
}

impl From<Position> for [f32; 3] {
    fn from(position: Position) -> Self {
        position.as_array()
    }
}

impl Add for Position {
    type Output = Position;

    fn add(self, rhs: Self) -> Self::Output {
        Position::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl AddAssign for Position {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub for Position {
    type Output = Position;

    fn sub(self, rhs: Self) -> Self::Output {
        Position::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl SubAssign for Position {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

#[cfg(test)]
mod tests {
    use super::Position;

    #[test]
    fn constructors_and_conversions_work() {
        let p = Position::new(1.0, 2.0, 3.0);
        assert_eq!(p.as_array(), [1.0, 2.0, 3.0]);

        let from_array = Position::from_array([4.0, 5.0, 6.0]);
        assert_eq!(
            from_array,
            Position {
                x: 4.0,
                y: 5.0,
                z: 6.0
            }
        );

        let array: [f32; 3] = p.into();
        assert_eq!(array, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn math_ops_behave_as_expected() {
        let mut a = Position::new(1.0, 1.0, 1.0);
        let b = Position::new(2.0, 3.0, 4.0);

        assert_eq!(a + b, Position::new(3.0, 4.0, 5.0));
        assert_eq!(b - a, Position::new(1.0, 2.0, 3.0));

        a += b;
        assert_eq!(a, Position::new(3.0, 4.0, 5.0));

        a -= Position::new(1.0, 1.0, 1.0);
        assert_eq!(a, Position::new(2.0, 3.0, 4.0));
    }

    #[test]
    fn distance_calculations_match_expectations() {
        let origin = Position::zero();
        let point = Position::new(3.0, 4.0, 12.0);
        assert_eq!(origin.distance_to(&point), 13.0);
    }
}
