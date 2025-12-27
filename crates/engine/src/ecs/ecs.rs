use crate::archetypes::{Archetype, EntityComponents};
use crate::components::{
    AttributesComponent, CameraComponent, HierarchyComponent, InputComponent, LightComponent,
    ModelComponent, Name, Orientation, ParticleComponent, ParticleEmitterComponent,
    PhysicsComponent, Position, RenderComponent, ScriptComponent, SpawnerComponent,
    TerrainComponent, TextureComponent,
};
use crate::ecs::entity_manager::EntityManager;
use crate::ecs::event_bus::EventBus;
use crate::ecs::tag_manager::TagManager;
use crate::ecs::{ComponentKind, ComponentSignature};
use log::debug;
use std::any::Any;
use std::collections::HashMap;

pub struct ECS {
    pub archetypes: Vec<Archetype>,
    pub entity_to_location: HashMap<u32, (usize, usize)>,
    signature_to_index: HashMap<ComponentSignature, usize>,
    pub entity_manager: EntityManager,
    pub tag_manager: TagManager,
    event_bus: EventBus,
}

impl ECS {
    pub fn new() -> Self {
        Self {
            archetypes: Vec::new(),
            entity_to_location: HashMap::new(),
            signature_to_index: HashMap::new(),
            entity_manager: EntityManager::new(),
            tag_manager: TagManager::new(),
            event_bus: EventBus::new(),
        }
    }

    pub fn event_bus(&self) -> &EventBus {
        &self.event_bus
    }

    pub fn event_bus_mut(&mut self) -> &mut EventBus {
        &mut self.event_bus
    }

    pub fn emit_event<E>(&mut self, event: E)
    where
        E: 'static + Send,
    {
        self.event_bus.publish(event);
    }

    pub fn emit_boxed_event(&mut self, event: Box<dyn Any + Send>) {
        self.event_bus.publish_boxed(event);
    }

    pub fn drain_events<E>(&mut self) -> Vec<E>
    where
        E: 'static + Send,
    {
        self.event_bus.drain::<E>()
    }

    pub fn add_entity(&mut self, position: Position, orientation: Orientation, name: Name) -> u32 {
        let id = self.entity_manager.create_entity();
        let archetype_index = self.get_or_create_archetype(ComponentSignature::empty());
        let index_within_archetype = self.archetypes[archetype_index]
            .push_entity(EntityComponents::base(id, position, orientation, name));
        self.entity_to_location
            .insert(id, (archetype_index, index_within_archetype));
        debug!(
            "Entity {} created. Current entity count: {}",
            id,
            self.entity_to_location.len()
        );
        id
    }

    pub fn find_entity(&self, id: u32) -> Option<&Archetype> {
        if let Some(&(archetype_index, _)) = self.entity_to_location.get(&id) {
            self.archetypes.get(archetype_index)
        } else {
            None
        }
    }

    pub fn find_entity_components(&self, id: u32) -> Option<(&Position, &Orientation, &Name)> {
        if let Some(&(archetype_index, index_within_archetype)) = self.entity_to_location.get(&id) {
            let archetype = &self.archetypes[archetype_index];
            Some((
                &archetype.positions[index_within_archetype],
                &archetype.orientations[index_within_archetype],
                &archetype.names[index_within_archetype],
            ))
        } else {
            None
        }
    }

    pub fn add_render_component(&mut self, id: u32, component: RenderComponent) {
        let update_value = component.clone();
        self.add_or_replace_component(
            id,
            ComponentKind::Render,
            move |bundle| bundle.render = Some(component),
            move |archetype, index| {
                if let Some(renderables) = archetype.renderables.as_mut() {
                    renderables[index] = update_value;
                }
            },
        );
    }

    pub fn add_input_component(&mut self, id: u32, component: InputComponent) {
        let update_value = component.clone();
        self.add_or_replace_component(
            id,
            ComponentKind::Input,
            move |bundle| bundle.input = Some(component),
            move |archetype, index| {
                if let Some(inputs) = archetype.inputs.as_mut() {
                    inputs[index] = update_value;
                }
            },
        );
    }

    pub fn add_model_component(&mut self, id: u32, component: ModelComponent) {
        let update_value = component.clone();
        self.add_or_replace_component(
            id,
            ComponentKind::Model,
            move |bundle| bundle.model = Some(component),
            move |archetype, index| {
                if let Some(models) = archetype.models.as_mut() {
                    models[index] = update_value;
                }
            },
        );
    }

    pub fn add_camera_component(&mut self, id: u32, component: CameraComponent) {
        let update_value = component.clone();
        self.add_or_replace_component(
            id,
            ComponentKind::Camera,
            move |bundle| bundle.camera = Some(component),
            move |archetype, index| {
                if let Some(cameras) = archetype.cameras.as_mut() {
                    cameras[index] = update_value;
                }
            },
        );
    }

    pub fn add_light_component(&mut self, id: u32, component: LightComponent) {
        let update_value = component.clone();
        self.add_or_replace_component(
            id,
            ComponentKind::Light,
            move |bundle| bundle.light = Some(component),
            move |archetype, index| {
                if let Some(lights) = archetype.lights.as_mut() {
                    lights[index] = update_value;
                }
            },
        );
    }

    pub fn add_texture_component(&mut self, id: u32, component: TextureComponent) {
        let update_value = component.clone();
        self.add_or_replace_component(
            id,
            ComponentKind::Texture,
            move |bundle| bundle.texture = Some(component),
            move |archetype, index| {
                if let Some(textures) = archetype.textures.as_mut() {
                    textures[index] = update_value;
                }
            },
        );
    }

    pub fn add_terrain_component(&mut self, id: u32, component: TerrainComponent) {
        let update_value = component.clone();
        self.add_or_replace_component(
            id,
            ComponentKind::Terrain,
            move |bundle| bundle.terrain = Some(component),
            move |archetype, index| {
                if let Some(terrains) = archetype.terrains.as_mut() {
                    terrains[index] = update_value;
                }
            },
        );
    }

    pub fn add_script_component(&mut self, id: u32, component: ScriptComponent) {
        let update_value = component.clone();
        self.add_or_replace_component(
            id,
            ComponentKind::Script,
            move |bundle| bundle.script = Some(component),
            move |archetype, index| {
                if let Some(scripts) = archetype.scripts.as_mut() {
                    scripts[index] = update_value;
                }
            },
        );
    }

    pub fn add_physics_component(&mut self, id: u32, component: PhysicsComponent) {
        let update_value = component.clone();
        self.add_or_replace_component(
            id,
            ComponentKind::Physics,
            move |bundle| bundle.physics = Some(component),
            move |archetype, index| {
                if let Some(physics) = archetype.physics.as_mut() {
                    physics[index] = update_value;
                }
            },
        );
    }

    pub fn add_hierarchy_component(&mut self, id: u32, component: HierarchyComponent) {
        let update_value = component;
        self.add_or_replace_component(
            id,
            ComponentKind::Hierarchy,
            move |bundle| bundle.hierarchy = Some(component),
            move |archetype, index| {
                if let Some(hierarchies) = archetype.hierarchies.as_mut() {
                    hierarchies[index] = update_value;
                }
            },
        );
    }

    pub fn add_attributes_component(&mut self, id: u32, component: AttributesComponent) {
        let update_value = component.clone();
        self.add_or_replace_component(
            id,
            ComponentKind::Attributes,
            move |bundle| bundle.attributes = Some(component),
            move |archetype, index| {
                if let Some(attributes) = archetype.attributes.as_mut() {
                    attributes[index] = update_value.clone();
                }
            },
        );
    }

    pub fn add_particle_emitter_component(&mut self, id: u32, component: ParticleEmitterComponent) {
        let update_value = component.clone();
        self.add_or_replace_component(
            id,
            ComponentKind::ParticleEmitter,
            move |bundle| bundle.particle_emitters = Some(component),
            move |archetype, index| {
                if let Some(emitters) = archetype.particle_emitters.as_mut() {
                    emitters[index] = update_value;
                }
            },
        );
    }

    pub fn add_particle_component(&mut self, id: u32, component: ParticleComponent) {
        let update_value = component.clone();
        self.add_or_replace_component(
            id,
            ComponentKind::Particle,
            move |bundle| bundle.particles = Some(component),
            move |archetype, index| {
                if let Some(particles) = archetype.particles.as_mut() {
                    particles[index] = update_value;
                }
            },
        );
    }

    pub fn add_spawner_component(&mut self, id: u32, component: SpawnerComponent) {
        let update_value = component.clone();
        self.add_or_replace_component(
            id,
            ComponentKind::Spawner,
            move |bundle| bundle.spawner = Some(component),
            move |archetype, index| {
                if let Some(spawners) = archetype.spawners.as_mut() {
                    spawners[index] = update_value;
                }
            },
        );
    }

    pub fn remove_render_component(&mut self, id: u32) {
        self.remove_component(id, ComponentKind::Render, |bundle| bundle.render = None);
    }

    pub fn remove_input_component(&mut self, id: u32) {
        self.remove_component(id, ComponentKind::Input, |bundle| bundle.input = None);
    }

    pub fn remove_model_component(&mut self, id: u32) {
        self.remove_component(id, ComponentKind::Model, |bundle| bundle.model = None);
    }

    pub fn remove_camera_component(&mut self, id: u32) {
        self.remove_component(id, ComponentKind::Camera, |bundle| bundle.camera = None);
    }

    pub fn remove_light_component(&mut self, id: u32) {
        self.remove_component(id, ComponentKind::Light, |bundle| bundle.light = None);
    }

    pub fn remove_texture_component(&mut self, id: u32) {
        self.remove_component(id, ComponentKind::Texture, |bundle| bundle.texture = None);
    }

    pub fn remove_terrain_component(&mut self, id: u32) {
        self.remove_component(id, ComponentKind::Terrain, |bundle| bundle.terrain = None);
    }

    pub fn remove_script_component(&mut self, id: u32) {
        self.remove_component(id, ComponentKind::Script, |bundle| bundle.script = None);
    }

    pub fn remove_physics_component(&mut self, id: u32) {
        self.remove_component(id, ComponentKind::Physics, |bundle| bundle.physics = None);
    }

    pub fn remove_hierarchy_component(&mut self, id: u32) {
        self.remove_component(id, ComponentKind::Hierarchy, |bundle| {
            bundle.hierarchy = None
        });
    }

    pub fn remove_attributes_component(&mut self, id: u32) {
        self.remove_component(id, ComponentKind::Attributes, |bundle| {
            bundle.attributes = None
        });
    }

    pub fn remove_particle_emitter_component(&mut self, id: u32) {
        self.remove_component(id, ComponentKind::ParticleEmitter, |bundle| {
            bundle.particle_emitters = None
        });
    }

    pub fn remove_particle_component(&mut self, id: u32) {
        self.remove_component(id, ComponentKind::Particle, |bundle| bundle.particles = None);
    }

    pub fn remove_spawner_component(&mut self, id: u32) {
        self.remove_component(id, ComponentKind::Spawner, |bundle| bundle.spawner = None);
    }

    pub fn input_component_mut(&mut self, id: u32) -> Option<&mut InputComponent> {
        if let Some(&(archetype_index, index_within_archetype)) = self.entity_to_location.get(&id) {
            if let Some(archetype) = self.archetypes.get_mut(archetype_index) {
                if let Some(inputs) = archetype.inputs.as_mut() {
                    return inputs.get_mut(index_within_archetype);
                }
            }
        }
        None
    }

    pub fn position_component_mut(&mut self, id: u32) -> Option<&mut Position> {
        if let Some(&(archetype_index, index_within_archetype)) = self.entity_to_location.get(&id) {
            if let Some(archetype) = self.archetypes.get_mut(archetype_index) {
                return archetype.positions.get_mut(index_within_archetype);
            }
        }
        None
    }

    pub fn orientation_component_mut(&mut self, id: u32) -> Option<&mut Orientation> {
        if let Some(&(archetype_index, index_within_archetype)) = self.entity_to_location.get(&id) {
            if let Some(archetype) = self.archetypes.get_mut(archetype_index) {
                return archetype.orientations.get_mut(index_within_archetype);
            }
        }
        None
    }

    pub fn orientation_component(&self, id: u32) -> Option<&Orientation> {
        if let Some(&(archetype_index, index_within_archetype)) = self.entity_to_location.get(&id) {
            if let Some(archetype) = self.archetypes.get(archetype_index) {
                return archetype.orientations.get(index_within_archetype);
            }
        }
        None
    }

    pub fn camera_component_mut(&mut self, id: u32) -> Option<&mut CameraComponent> {
        if let Some(&(archetype_index, index_within_archetype)) = self.entity_to_location.get(&id) {
            if let Some(archetype) = self.archetypes.get_mut(archetype_index) {
                if let Some(cameras) = archetype.cameras.as_mut() {
                    return cameras.get_mut(index_within_archetype);
                }
            }
        }
        None
    }

    pub fn light_component_mut(&mut self, id: u32) -> Option<&mut LightComponent> {
        if let Some(&(archetype_index, index_within_archetype)) = self.entity_to_location.get(&id) {
            if let Some(archetype) = self.archetypes.get_mut(archetype_index) {
                if let Some(lights) = archetype.lights.as_mut() {
                    return lights.get_mut(index_within_archetype);
                }
            }
        }
        None
    }

    pub fn camera_component(&self, id: u32) -> Option<&CameraComponent> {
        if let Some(&(archetype_index, index_within_archetype)) = self.entity_to_location.get(&id) {
            if let Some(archetype) = self.archetypes.get(archetype_index) {
                if let Some(cameras) = archetype.cameras.as_ref() {
                    return cameras.get(index_within_archetype);
                }
            }
        }
        None
    }

    pub fn attributes_component_mut(&mut self, id: u32) -> Option<&mut AttributesComponent> {
        if let Some(&(archetype_index, index_within_archetype)) = self.entity_to_location.get(&id) {
            if let Some(archetype) = self.archetypes.get_mut(archetype_index) {
                if let Some(attributes) = archetype.attributes.as_mut() {
                    return attributes.get_mut(index_within_archetype);
                }
            }
        }
        None
    }

    pub fn attributes_component(&self, id: u32) -> Option<&AttributesComponent> {
        if let Some(&(archetype_index, index_within_archetype)) = self.entity_to_location.get(&id) {
            if let Some(archetype) = self.archetypes.get(archetype_index) {
                if let Some(attributes) = archetype.attributes.as_ref() {
                    return attributes.get(index_within_archetype);
                }
            }
        }
        None
    }

    pub fn physics_component(&self, id: u32) -> Option<&PhysicsComponent> {
        if let Some(&(archetype_index, index_within_archetype)) = self.entity_to_location.get(&id) {
            if let Some(archetype) = self.archetypes.get(archetype_index) {
                if let Some(physics) = archetype.physics.as_ref() {
                    return physics.get(index_within_archetype);
                }
            }
        }
        None
    }

    pub fn hierarchy_component(&self, id: u32) -> Option<&HierarchyComponent> {
        if let Some(&(archetype_index, index_within_archetype)) = self.entity_to_location.get(&id) {
            if let Some(archetype) = self.archetypes.get(archetype_index) {
                if let Some(hierarchies) = archetype.hierarchies.as_ref() {
                    return hierarchies.get(index_within_archetype);
                }
            }
        }
        None
    }

    pub fn hierarchy_component_mut(&mut self, id: u32) -> Option<&mut HierarchyComponent> {
        if let Some(&(archetype_index, index_within_archetype)) = self.entity_to_location.get(&id) {
            if let Some(archetype) = self.archetypes.get_mut(archetype_index) {
                if let Some(hierarchies) = archetype.hierarchies.as_mut() {
                    return hierarchies.get_mut(index_within_archetype);
                }
            }
        }
        None
    }

    pub fn find_entity_id_by_name(&self, name: &str) -> Option<u32> {
        for archetype in &self.archetypes {
            for (index, entity_name) in archetype.names.iter().enumerate() {
                if entity_name.0 == name {
                    return Some(archetype.entity_ids[index]);
                }
            }
        }
        None
    }

    pub fn model_component(&self, id: u32) -> Option<&ModelComponent> {
        if let Some(&(archetype_index, index_within_archetype)) = self.entity_to_location.get(&id) {
            if let Some(archetype) = self.archetypes.get(archetype_index) {
                if let Some(models) = archetype.models.as_ref() {
                    return models.get(index_within_archetype);
                }
            }
        }
        None
    }

    pub fn spawner_component(&self, id: u32) -> Option<&SpawnerComponent> {
        if let Some(&(archetype_index, index_within_archetype)) = self.entity_to_location.get(&id) {
            if let Some(archetype) = self.archetypes.get(archetype_index) {
                if let Some(spawners) = archetype.spawners.as_ref() {
                    return spawners.get(index_within_archetype);
                }
            }
        }
        None
    }

    pub fn texture_component(&self, id: u32) -> Option<&TextureComponent> {
        if let Some(&(archetype_index, index_within_archetype)) = self.entity_to_location.get(&id) {
            if let Some(archetype) = self.archetypes.get(archetype_index) {
                if let Some(textures) = archetype.textures.as_ref() {
                    return textures.get(index_within_archetype);
                }
            }
        }
        None
    }

    pub fn light_components(&self) -> impl Iterator<Item = (&Position, &LightComponent)> {
        self.archetypes.iter().flat_map(|archetype| {
            archetype
                .lights
                .as_ref()
                .map(|lights| archetype.positions.iter().zip(lights.iter()))
                .into_iter()
                .flatten()
        })
    }

    pub fn terrain_components(&self) -> impl Iterator<Item = &TerrainComponent> {
        self.archetypes.iter().flat_map(|archetype| {
            archetype
                .terrains
                .as_ref()
                .map(|terrains| terrains.iter())
                .into_iter()
                .flatten()
        })
    }

    pub fn remove_entity(&mut self, id: u32) {
        if let Some((archetype_index, index_within_archetype)) = self.entity_to_location.remove(&id)
        {
            let (_components, swapped_id) =
                self.archetypes[archetype_index].swap_remove_entity(index_within_archetype);
            if let Some(swapped) = swapped_id {
                self.entity_to_location
                    .insert(swapped, (archetype_index, index_within_archetype));
            }
            self.entity_manager.destroy_entity(id);
            debug!(
                "Entity {} deleted. Current entity count: {}",
                id,
                self.entity_to_location.len()
            );
            self.detach_children_of(id);
        }
    }

    fn detach_children_of(&mut self, parent_id: u32) {
        let mut children = Vec::new();
        for archetype in &self.archetypes {
            let Some(hierarchies) = archetype.hierarchies.as_ref() else {
                continue;
            };
            for (index, component) in hierarchies.iter().enumerate() {
                if component.parent == parent_id {
                    children.push(archetype.entity_ids[index]);
                }
            }
        }
        for child in children {
            self.remove_hierarchy_component(child);
        }
    }

    fn get_or_create_archetype(&mut self, signature: ComponentSignature) -> usize {
        if let Some(&index) = self.signature_to_index.get(&signature) {
            return index;
        }
        let index = self.archetypes.len();
        self.archetypes.push(Archetype::new(signature));
        self.signature_to_index.insert(signature, index);
        index
    }

    fn add_or_replace_component<FNew, FExisting>(
        &mut self,
        id: u32,
        kind: ComponentKind,
        add: FNew,
        update_existing: FExisting,
    ) where
        FNew: FnOnce(&mut EntityComponents),
        FExisting: FnOnce(&mut Archetype, usize),
    {
        let Some(&(archetype_index, index_within_archetype)) = self.entity_to_location.get(&id)
        else {
            return;
        };

        let current_signature = self.archetypes[archetype_index].signature;
        if current_signature.contains(kind) {
            update_existing(
                &mut self.archetypes[archetype_index],
                index_within_archetype,
            );
            return;
        }

        let (mut components, swapped_id) =
            self.archetypes[archetype_index].swap_remove_entity(index_within_archetype);
        if let Some(swapped) = swapped_id {
            self.entity_to_location
                .insert(swapped, (archetype_index, index_within_archetype));
        }

        add(&mut components);
        let target_signature = current_signature.with(kind);
        let target_index = self.get_or_create_archetype(target_signature);
        let new_position = self.archetypes[target_index].push_entity(components);
        self.entity_to_location
            .insert(id, (target_index, new_position));
    }

    fn remove_component<FStrip>(&mut self, id: u32, kind: ComponentKind, strip: FStrip)
    where
        FStrip: FnOnce(&mut EntityComponents),
    {
        let Some(&(archetype_index, index_within_archetype)) = self.entity_to_location.get(&id)
        else {
            return;
        };

        let current_signature = self.archetypes[archetype_index].signature;
        if !current_signature.contains(kind) {
            return;
        }

        let (mut components, swapped_id) =
            self.archetypes[archetype_index].swap_remove_entity(index_within_archetype);
        if let Some(swapped) = swapped_id {
            self.entity_to_location
                .insert(swapped, (archetype_index, index_within_archetype));
        }

        strip(&mut components);
        let target_signature = current_signature.without(kind);
        let target_index = self.get_or_create_archetype(target_signature);
        let new_position = self.archetypes[target_index].push_entity(components);
        self.entity_to_location
            .insert(id, (target_index, new_position));
    }
}
