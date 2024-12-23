use crate::archetypes::Archetype;

pub struct MovementSystem;

impl MovementSystem {
    pub fn update(archetype: &mut Archetype) {
        for pos in archetype.positions.iter_mut() {
            pos.x += 0.1;
            pos.y += 0.1;
        }
    }
}

