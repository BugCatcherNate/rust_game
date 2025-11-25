use crate::archetypes::Archetype;
use crate::ecs::ECS;
pub struct MovementSystem;

impl MovementSystem {
    pub fn update(ecs: &mut ECS) {
        for archetype in &mut ecs.archetypes {
            Self::update_archetype(archetype);
        }
    }

    fn update_archetype(archetype: &mut Archetype) {
        if archetype.physics.is_some() {
            return;
        }

        let len = archetype.len();

        let inputs = archetype.inputs.as_ref();
        for index in 0..len {
            if let Some(input) = inputs.and_then(|vec| vec.get(index)) {
                let pos = &mut archetype.positions[index];
                pos.x += input.direction[0] * input.speed;
                pos.y += input.direction[1] * input.speed;
                pos.z += input.direction[2] * input.speed;
            }
        }
    }
}
