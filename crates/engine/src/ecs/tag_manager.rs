use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct TagManager {
    pub tag_to_entities: HashMap<String, HashSet<u32>>,
}

impl TagManager {
    pub fn new() -> Self {
        Self {
            tag_to_entities: HashMap::new(),
        }
    }

    pub fn add_tag(&mut self, entity: u32, tag: &str) {
        self.tag_to_entities
            .entry(tag.to_string())
            .or_insert_with(HashSet::new)
            .insert(entity);
    }

    pub fn remove_tag(&mut self, entity: u32, tag: &str) {
        if let Some(entities) = self.tag_to_entities.get_mut(tag) {
            entities.remove(&entity);
            if entities.is_empty() {
                self.tag_to_entities.remove(tag);
            }
        }
    }

    pub fn get_entities_with_tag(&self, tag: &str) -> Option<&HashSet<u32>> {
        self.tag_to_entities.get(tag)
    }

    pub fn tags_for_entity(&self, entity: u32) -> Vec<String> {
        self.tag_to_entities
            .iter()
            .filter_map(|(tag, entities)| {
                if entities.contains(&entity) {
                    Some(tag.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}
