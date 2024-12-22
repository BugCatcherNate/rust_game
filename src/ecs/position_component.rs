pub struct PositionComponent{
    pub position: [f32; 3],
}

impl PositionComponent{
    pub fn new(position: [f32; 3]) -> Self {
        Self{
            position,
        }

    }


}
