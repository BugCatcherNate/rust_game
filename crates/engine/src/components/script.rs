use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct ScriptComponent {
    pub name: String,
    pub base_height: f32,
    pub params: HashMap<String, String>,
}

impl ScriptComponent {
    #[allow(dead_code)]
    pub fn new(name: impl Into<String>, base_height: f32) -> Self {
        Self {
            name: name.into(),
            base_height,
            params: HashMap::new(),
        }
    }

    pub fn with_params(
        name: impl Into<String>,
        base_height: f32,
        params: HashMap<String, String>,
    ) -> Self {
        Self {
            name: name.into(),
            base_height,
            params,
        }
    }
}
