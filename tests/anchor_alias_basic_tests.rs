//! Basic tests for anchor/alias data model

use yamlquill::document::node::{YamlNode, YamlValue};
use yamlquill::document::tree::{AnchorRegistry, YamlTree};

#[test]
fn test_yaml_node_has_alias_target_field() {
    let mut node = YamlNode::new(YamlValue::Null);

    // Should be None initially
    assert!(node.alias_target().is_none());

    // Should be settable
    node.set_alias_target(Some("test_anchor".to_string()));
    assert_eq!(node.alias_target(), Some("test_anchor"));
}

#[test]
fn test_anchor_registry_register_and_lookup() {
    let mut registry = AnchorRegistry::new();

    // Register an anchor
    registry.register_anchor("default".to_string(), vec![0, 1]);

    // Should be able to look it up
    assert_eq!(registry.get_anchor_path("default"), Some(&vec![0, 1]));
    assert_eq!(registry.get_anchor_path("nonexistent"), None);
}

#[test]
fn test_anchor_registry_aliases() {
    let mut registry = AnchorRegistry::new();

    registry.register_anchor("config".to_string(), vec![0]);
    registry.register_alias(vec![1, 0], "config".to_string());
    registry.register_alias(vec![2, 0], "config".to_string());

    let aliases = registry.get_aliases_for("config");
    assert_eq!(aliases.len(), 2);
    assert!(aliases.contains(&&vec![1, 0]));
    assert!(aliases.contains(&&vec![2, 0]));
}

#[test]
fn test_anchor_registry_can_delete() {
    let mut registry = AnchorRegistry::new();

    registry.register_anchor("test".to_string(), vec![0]);

    // Can delete when no aliases
    assert!(registry.can_delete_anchor("test"));

    // Cannot delete when aliases exist
    registry.register_alias(vec![1], "test".to_string());
    assert!(!registry.can_delete_anchor("test"));
}

#[test]
fn test_anchor_registry_remove_node() {
    let mut registry = AnchorRegistry::new();

    registry.register_anchor("test".to_string(), vec![0]);
    registry.register_alias(vec![1], "test".to_string());

    // Remove anchor node
    registry.remove_node(&[0]);
    assert!(registry.get_anchor_path("test").is_none());

    // Remove alias node
    registry.remove_node(&[1]);
    assert_eq!(registry.get_aliases_for("test").len(), 0);
}

#[test]
fn test_yaml_tree_has_anchor_registry() {
    let tree = YamlTree::new(YamlNode::new(YamlValue::Null));

    // Should have an anchor registry
    assert!(tree.anchor_registry().get_anchor_path("test").is_none());
}
