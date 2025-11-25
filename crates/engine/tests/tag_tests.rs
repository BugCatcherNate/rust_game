use rust_game::ecs::tag_manager::TagManager;

#[test]
fn test_add_tag() {
    let mut tag_manager = TagManager::new();

    // Add a tag to an entity
    tag_manager.add_tag(1, "Player");

    // Check if the tag is associated with the entity
    let entities = tag_manager.get_entities_with_tag("Player").unwrap();
    assert!(entities.contains(&1));
}

#[test]
fn test_remove_tag() {
    let mut tag_manager = TagManager::new();

    // Add and then remove a tag
    tag_manager.add_tag(1, "Enemy");
    tag_manager.remove_tag(1, "Enemy");

    // Check if the tag is no longer associated
    assert!(tag_manager.get_entities_with_tag("Enemy").is_none());
}

#[test]
fn test_get_entities_with_tag() {
    let mut tag_manager = TagManager::new();

    // Add multiple entities to a tag
    tag_manager.add_tag(1, "NPC");
    tag_manager.add_tag(2, "NPC");
    tag_manager.add_tag(3, "Enemy");

    // Get entities with the tag
    let entities = tag_manager.get_entities_with_tag("NPC").unwrap();
    assert!(entities.contains(&1));
    assert!(entities.contains(&2));
    assert_eq!(entities.len(), 2);
}
#[test]
fn test_multiple_tags() {
    let mut tag_manager = TagManager::new();

    // Add multiple tags to a single entity
    tag_manager.add_tag(1, "Player");
    tag_manager.add_tag(1, "NPC");

    // Add another entity to one of the tags
    tag_manager.add_tag(2, "NPC");

    // Verify the entity is associated with both tags
    let player_entities = tag_manager.get_entities_with_tag("Player").unwrap();
    assert!(player_entities.contains(&1));
    assert_eq!(player_entities.len(), 1);

    let npc_entities = tag_manager.get_entities_with_tag("NPC").unwrap();
    assert!(npc_entities.contains(&1));
    assert!(npc_entities.contains(&2));
    assert_eq!(npc_entities.len(), 2);

    // Remove a tag from the first entity
    tag_manager.remove_tag(1, "Player");

    // Ensure the tag is removed from the entity
    assert!(tag_manager.get_entities_with_tag("Player").is_none());

    // Ensure the entity still has the other tag
    let npc_entities = tag_manager.get_entities_with_tag("NPC").unwrap();
    assert!(npc_entities.contains(&1));
    assert!(npc_entities.contains(&2));
}
#[test]
fn test_remove_entity_from_tag_group() {
    let mut tag_manager = TagManager::new();

    // Add multiple entities to the same tag
    tag_manager.add_tag(1, "Collectible");
    tag_manager.add_tag(2, "Collectible");

    // Remove one entity
    tag_manager.remove_tag(1, "Collectible");

    // Check the remaining entities
    let entities = tag_manager.get_entities_with_tag("Collectible").unwrap();
    assert!(entities.contains(&2));
    assert!(!entities.contains(&1));
}

#[test]
fn test_remove_last_entity_from_tag() {
    let mut tag_manager = TagManager::new();

    // Add an entity to a tag and then remove it
    tag_manager.add_tag(1, "QuestItem");
    tag_manager.remove_tag(1, "QuestItem");

    // Ensure the tag group is cleaned up
    assert!(tag_manager.get_entities_with_tag("QuestItem").is_none());
}

#[test]
fn test_handle_non_existent_tag() {
    let mut tag_manager = TagManager::new();

    // Try removing an entity from a non-existent tag
    tag_manager.remove_tag(1, "NonExistent");

    // Ensure no panics and behavior is as expected
    assert!(tag_manager.get_entities_with_tag("NonExistent").is_none());
}
