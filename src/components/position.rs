#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl PartialEq<&Position> for Position {
    fn eq(&self, other: &&Position) -> bool {
        self.x == other.x && self.y == other.y
    }
}
