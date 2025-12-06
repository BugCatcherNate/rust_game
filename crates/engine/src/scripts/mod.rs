use std::any::Any;
use std::collections::HashMap;

use crate::components::{Position, RenderComponent};
use crate::ecs::ComponentKind;
use crate::rendering::DebugLine;

#[allow(dead_code)]
pub struct ScriptEntityContext<'a> {
    pub id: u32,
    pub name: &'a str,
}

#[allow(dead_code)]
pub struct ScriptContext<'a> {
    pub time: f32,
    pub dt: f32,
    pub base_height: f32,
    pub entity: ScriptEntityContext<'a>,
    pub tags: &'a [String],
    pub render: Option<&'a RenderComponent>,
    pub scene: &'a [SceneSnapshotEntry],
    pub params: &'a HashMap<String, String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SceneSnapshotEntry {
    pub id: u32,
    pub name: String,
    pub tags: Vec<String>,
    pub position: Position,
}

#[derive(Debug)]
pub enum ScriptCommand {
    LoadScene(String),
    RemoveComponent {
        entity_id: u32,
        component: ComponentKind,
    },
    EmitEvent(Box<dyn Any + Send>),
    DebugLine(DebugLine),
}

pub trait ScriptBehavior {
    fn update(
        &mut self,
        ctx: ScriptContext<'_>,
        position: &mut Position,
        commands: &mut Vec<ScriptCommand>,
    );
}

type ScriptFactory = dyn Fn() -> Box<dyn ScriptBehavior> + Send + Sync;

#[derive(Default)]
pub struct ScriptRegistry {
    constructors: HashMap<String, Box<ScriptFactory>>,
}

impl ScriptRegistry {
    pub fn new() -> Self {
        Self {
            constructors: HashMap::new(),
        }
    }

    pub fn register_script<F>(&mut self, name: &str, constructor: F)
    where
        F: Fn() -> Box<dyn ScriptBehavior> + Send + Sync + 'static,
    {
        self.constructors
            .insert(name.to_string(), Box::new(constructor));
    }

    pub fn create_script(&self, name: &str) -> Option<Box<dyn ScriptBehavior>> {
        self.constructors.get(name).map(|constructor| constructor())
    }
}
