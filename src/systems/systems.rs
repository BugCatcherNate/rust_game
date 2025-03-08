use crate::archetypes::Archetype;
use std::any::Any;

pub trait System {
    fn update(&self, arch: &mut Archetype);
    fn as_any(&self) -> &dyn Any;
}

pub struct Movement;

impl System for Movement {
    fn update(&self, archetype: &mut Archetype) {
        for pos in archetype.positions.iter_mut() {
            pos.as_mut().map(|position| {
                position.x += 0.1;
                position.y += 0.1;
            });
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
