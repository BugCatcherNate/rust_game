use crate::components::{Name, RenderComponent};
use crate::ecs::ECS;
use crate::scripts::{
    SceneSnapshotEntry, ScriptBehavior, ScriptCommand, ScriptContext, ScriptEntityContext,
    ScriptRegistry,
};
use log::error;
use std::collections::HashMap;
use std::sync::Arc;

const SCRIPT_DT: f32 = 1.0 / 60.0;

pub struct ScriptingSystem {
    time: f32,
    behaviors: HashMap<String, Box<dyn ScriptBehavior>>,
    pending_commands: Vec<ScriptCommand>,
    script_registry: Arc<ScriptRegistry>,
}

impl ScriptingSystem {
    pub fn new(script_registry: Arc<ScriptRegistry>) -> Self {
        Self {
            time: 0.0,
            behaviors: HashMap::new(),
            pending_commands: Vec::new(),
            script_registry,
        }
    }

    pub fn reset(&mut self) {
        self.time = 0.0;
        self.behaviors.clear();
        self.pending_commands.clear();
    }

    pub fn take_commands(&mut self) -> Vec<ScriptCommand> {
        std::mem::take(&mut self.pending_commands)
    }

    pub fn update(&mut self, ecs: &mut ECS) {
        self.time += SCRIPT_DT;
        let scene_snapshot = Self::build_scene_snapshot(ecs);
        let current_time = self.time;
        let mut script_commands = Vec::new();

        for archetype in &mut ecs.archetypes {
            let Some(scripts) = archetype.scripts.as_ref() else {
                continue;
            };
            let len = scripts.len();
            for index in 0..len {
                let script_component = &scripts[index];

                let Some(position) = archetype.positions.get_mut(index) else {
                    continue;
                };
                let Some(orientation) = archetype.orientations.get_mut(index) else {
                    continue;
                };

                let name: &Name = &archetype.names[index];
                let render: Option<&RenderComponent> = archetype
                    .renderables
                    .as_ref()
                    .and_then(|column| column.get(index));
                let entity_id = archetype.entity_ids[index];
                let tags = ecs.tag_manager.tags_for_entity(entity_id);

                let Some(script) = self.get_or_create_behavior(&script_component.name) else {
                    error!(
                        "Script '{}' is not registered; skipping entity {}",
                        script_component.name, name.0
                    );
                    continue;
                };

                let ctx = ScriptContext {
                    time: current_time,
                    dt: SCRIPT_DT,
                    base_height: script_component.base_height,
                    entity: ScriptEntityContext {
                        id: entity_id,
                        name: &name.0,
                    },
                    tags: &tags,
                    render,
                    scene: &scene_snapshot,
                    params: &script_component.params,
                };

                script_commands.clear();
                script.update(ctx, position, orientation, &mut script_commands);
                self.pending_commands.extend(script_commands.drain(..));
            }
        }
    }

    fn get_or_create_behavior(&mut self, name: &str) -> Option<&mut (dyn ScriptBehavior + '_)> {
        if !self.behaviors.contains_key(name) {
            let behavior = self.script_registry.create_script(name)?;
            self.behaviors.insert(name.to_string(), behavior);
        }
        if let Some(behavior) = self.behaviors.get_mut(name) {
            Some(behavior.as_mut())
        } else {
            None
        }
    }

    fn build_scene_snapshot(ecs: &ECS) -> Vec<SceneSnapshotEntry> {
        let mut entries = Vec::new();
        for archetype in &ecs.archetypes {
            for (i, position) in archetype.positions.iter().enumerate() {
                entries.push(SceneSnapshotEntry {
                    id: archetype.entity_ids[i],
                    name: archetype.names[i].0.clone(),
                    tags: ecs.tag_manager.tags_for_entity(archetype.entity_ids[i]),
                    position: *position,
                });
            }
        }
        entries
    }
}
