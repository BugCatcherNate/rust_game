use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct AttributesComponent {
    values: HashMap<String, f32>,
}

impl AttributesComponent {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn from_values(values: HashMap<String, f32>) -> Self {
        Self { values }
    }

    pub fn set(&mut self, key: impl Into<String>, value: f32) {
        self.values.insert(key.into(), value);
    }

    pub fn remove(&mut self, key: &str) -> Option<f32> {
        self.values.remove(key)
    }

    pub fn get(&self, key: &str) -> Option<f32> {
        self.values.get(key).copied()
    }

    pub fn contains(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    pub fn values(&self) -> &HashMap<String, f32> {
        &self.values
    }
}

impl Default for AttributesComponent {
    fn default() -> Self {
        Self::new()
    }
}
