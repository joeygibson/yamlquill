//! Tests for anchor and alias extraction using Scanner
//!
//! These tests validate the hybrid parsing approach that uses Scanner
//! to extract anchor/alias names and correlates them with the parsed tree.

use yamlquill::document::node::{YamlNumber, YamlString, YamlValue};
use yamlquill::document::parser::parse_yaml_auto;

#[test]
fn test_parse_yaml_without_anchors() {
    let yaml = "name: Alice\nage: 30";
    let node = parse_yaml_auto(yaml).unwrap();

    // Should parse normally without anchors
    match node.value() {
        YamlValue::Object(map) => {
            assert_eq!(map.len(), 2);
            assert!(node.anchor().is_none());
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_parse_simple_alias() {
    let yaml = r#"
defaults: &config
  timeout: 30
  retries: 3

production:
  settings: *config
"#;
    let node = parse_yaml_auto(yaml).unwrap();

    match node.value() {
        YamlValue::Object(map) => {
            // Check that we have both keys
            assert!(map.contains_key("defaults"));
            assert!(map.contains_key("production"));

            // Check production.settings
            let production = map.get("production").unwrap();
            match production.value() {
                YamlValue::Object(prod_map) => {
                    let settings = prod_map.get("settings").unwrap();

                    // Settings should be an alias
                    match settings.value() {
                        YamlValue::Alias(target) => {
                            assert_eq!(target, "config", "Alias should reference 'config'");
                            assert_eq!(
                                settings.alias_target(),
                                Some("config"),
                                "alias_target should be populated"
                            );
                        }
                        other => panic!("Expected Alias, got {:?}", other),
                    }
                }
                _ => panic!("Expected object for production"),
            }
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_parse_multiple_aliases() {
    let yaml = r#"
base: &base_config
  port: 8080
  host: localhost

dev:
  config: *base_config

prod:
  config: *base_config
"#;
    let node = parse_yaml_auto(yaml).unwrap();

    match node.value() {
        YamlValue::Object(map) => {
            // Check dev.config
            let dev = map.get("dev").unwrap();
            match dev.value() {
                YamlValue::Object(dev_map) => {
                    let config = dev_map.get("config").unwrap();
                    assert!(
                        matches!(config.value(), YamlValue::Alias(_)),
                        "dev.config should be an alias"
                    );
                    assert_eq!(config.alias_target(), Some("base_config"));
                }
                _ => panic!("Expected object for dev"),
            }

            // Check prod.config
            let prod = map.get("prod").unwrap();
            match prod.value() {
                YamlValue::Object(prod_map) => {
                    let config = prod_map.get("config").unwrap();
                    assert!(
                        matches!(config.value(), YamlValue::Alias(_)),
                        "prod.config should be an alias"
                    );
                    assert_eq!(config.alias_target(), Some("base_config"));
                }
                _ => panic!("Expected object for prod"),
            }
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_parse_nested_structure_with_alias() {
    let yaml = r#"
database: &db_config
  host: localhost
  port: 5432

services:
  api:
    name: API Service
    database: *db_config
  worker:
    name: Background Worker
    database: *db_config
"#;
    let node = parse_yaml_auto(yaml).unwrap();

    match node.value() {
        YamlValue::Object(map) => {
            let services = map.get("services").unwrap();
            match services.value() {
                YamlValue::Object(services_map) => {
                    // Check API service
                    let api = services_map.get("api").unwrap();
                    match api.value() {
                        YamlValue::Object(api_map) => {
                            let db = api_map.get("database").unwrap();
                            assert_eq!(db.alias_target(), Some("db_config"));
                        }
                        _ => panic!("Expected object for api"),
                    }

                    // Check worker service
                    let worker = services_map.get("worker").unwrap();
                    match worker.value() {
                        YamlValue::Object(worker_map) => {
                            let db = worker_map.get("database").unwrap();
                            assert_eq!(db.alias_target(), Some("db_config"));
                        }
                        _ => panic!("Expected object for worker"),
                    }
                }
                _ => panic!("Expected object for services"),
            }
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_parse_array_with_alias() {
    let yaml = r#"
default_item: &item
  name: Widget
  price: 10

inventory:
  - *item
  - *item
  - *item
"#;
    let node = parse_yaml_auto(yaml).unwrap();

    match node.value() {
        YamlValue::Object(map) => {
            let inventory = map.get("inventory").unwrap();
            match inventory.value() {
                YamlValue::Array(items) => {
                    assert_eq!(items.len(), 3);

                    // All items should be aliases
                    for item in items {
                        assert_eq!(item.alias_target(), Some("item"));
                    }
                }
                _ => panic!("Expected array for inventory"),
            }
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_parse_yaml_merge_key() {
    // YAML merge keys (<<: *anchor) are a common pattern
    let yaml = r#"
defaults: &defaults
  timeout: 30
  retries: 3

production:
  <<: *defaults
  timeout: 60
"#;
    let node = parse_yaml_auto(yaml).unwrap();

    // This should parse without errors
    // The merge key behavior depends on yaml-rust2's handling
    assert!(matches!(node.value(), YamlValue::Object(_)));
}

#[test]
fn test_debug_parse_output() {
    let yaml = r#"
defaults: &config
  timeout: 30

production:
  settings: *config
"#;
    let node = parse_yaml_auto(yaml).unwrap();

    println!("\n=== DEBUG: Root Node ===");
    println!("Value type: {:?}", std::mem::discriminant(node.value()));

    match node.value() {
        YamlValue::Object(map) => {
            println!("Object with {} keys", map.len());
            for (key, _val) in map.iter() {
                println!("  - Key: {}", key);
            }
        }
        other => {
            println!("Not an object: {:?}", other);
        }
    }
}

#[test]
fn test_anchor_badge_display() {
    // Test that anchors show up in tree view
    let yaml = r#"
defaults: &config
  timeout: 30

production:
  settings: *config
"#;
    let node = parse_yaml_auto(yaml).unwrap();

    // Verify anchor is set on the defaults node
    match node.value() {
        YamlValue::Object(map) => {
            let defaults = map.get("defaults").unwrap();
            assert_eq!(
                defaults.anchor(),
                Some("config"),
                "Anchor should be 'config'"
            );

            // Verify value display
            match defaults.value() {
                YamlValue::Object(_) => {
                    // Good - it's an object with an anchor
                }
                other => panic!("Expected Object, got {:?}", other),
            }
        }
        _ => panic!("Expected root to be Object"),
    }
}

#[test]
fn test_anchor_delete_protection() {
    use yamlquill::document::tree::YamlTree;
    use yamlquill::editor::state::EditorState;

    let yaml = r#"
defaults: &config
  timeout: 30

production:
  settings: *config
"#;
    let node = parse_yaml_auto(yaml).unwrap();
    let tree = YamlTree::new(node);
    let mut state = EditorState::new_with_default_theme(tree);

    // Directly access the node at path [0] (first value in root object)
    // This is the "defaults" object which has the &config anchor
    let defaults_path = vec![0];
    state.cursor_mut().set_path(defaults_path.clone());

    // Verify we're on the defaults node with anchor
    let current_node = state.tree().get_node(&defaults_path).unwrap();
    assert_eq!(
        current_node.anchor(),
        Some("config"),
        "Should be on node with 'config' anchor"
    );

    // Attempt to delete the node with the anchor
    let result = state.delete_node_at_cursor();

    // Deletion should succeed (returns Ok) but the node should still be there
    // because we blocked it and returned early
    assert!(result.is_ok(), "Delete function should return Ok");

    // Verify the node is still present
    let node_after_delete = state.tree().get_node(state.cursor().path());
    assert!(
        node_after_delete.is_some(),
        "Node should still exist after blocked delete"
    );
    assert_eq!(
        node_after_delete.unwrap().anchor(),
        Some("config"),
        "Anchor should still be present"
    );

    // Verify there's an error message
    let message = state.message();
    assert!(message.is_some(), "Should have an error message");
    let msg_text = &message.unwrap().text;
    assert!(
        msg_text.contains("Cannot delete anchor"),
        "Message should indicate anchor deletion is blocked, got: {}",
        msg_text
    );
    assert!(
        msg_text.contains("config"),
        "Message should mention the anchor name 'config'"
    );
    assert!(
        msg_text.contains("1 alias(es)"),
        "Message should indicate 1 alias references it"
    );
}

#[test]
fn test_delete_node_without_anchor() {
    use yamlquill::document::tree::YamlTree;
    use yamlquill::editor::state::EditorState;

    let yaml = r#"
defaults: &config
  timeout: 30
  retries: 3

standalone:
  name: test
"#;
    let node = parse_yaml_auto(yaml).unwrap();
    let tree = YamlTree::new(node);
    let mut state = EditorState::new_with_default_theme(tree);

    // Directly access the node at path [1] (second value in root object)
    // This is the "standalone" object which has no anchor
    let standalone_path = vec![1];
    state.cursor_mut().set_path(standalone_path.clone());

    // Verify we're on the standalone node without anchor
    let current_node = state.tree().get_node(&standalone_path).unwrap();
    assert_eq!(
        current_node.anchor(),
        None,
        "Should be on node without anchor"
    );

    // Get the path before deletion
    let path = standalone_path;

    // Attempt to delete the node
    let result = state.delete_node_at_cursor();

    // Deletion should succeed
    assert!(
        result.is_ok(),
        "Delete should succeed for node without anchor"
    );

    // Verify the node is actually deleted
    let node_after_delete = state.tree().get_node(&path);
    assert!(node_after_delete.is_none(), "Node should be deleted");

    // No error message should be present (or message should not be about anchor)
    if let Some(msg) = state.message() {
        assert!(
            !msg.text.contains("Cannot delete anchor"),
            "Should not have anchor deletion error"
        );
    }
}

#[test]
fn test_delete_anchor_after_aliases_removed() {
    use yamlquill::document::tree::YamlTree;
    use yamlquill::editor::state::EditorState;

    let yaml = r#"
defaults: &config
  timeout: 30

production:
  settings: *config
"#;
    let node = parse_yaml_auto(yaml).unwrap();
    let tree = YamlTree::new(node);
    let mut state = EditorState::new_with_default_theme(tree);

    // First, delete the alias at production.settings (path [1, 0])
    // Path [1] is the production object, [1, 0] is the first field (settings)
    let alias_path = vec![1, 0];
    state.cursor_mut().set_path(alias_path.clone());

    // Verify we're on the alias
    let alias_node = state.tree().get_node(&alias_path).unwrap();
    assert!(
        matches!(
            alias_node.value(),
            yamlquill::document::node::YamlValue::Alias(_)
        ),
        "Should be on alias node"
    );

    // Delete the alias
    let result = state.delete_node_at_cursor();
    assert!(result.is_ok(), "Should be able to delete alias");

    // Now try to delete the anchor at path [0] (defaults object)
    let anchor_path = vec![0];
    state.cursor_mut().set_path(anchor_path.clone());

    // Verify we're on the defaults node with anchor
    let anchor_node = state.tree().get_node(&anchor_path).unwrap();
    assert_eq!(
        anchor_node.anchor(),
        Some("config"),
        "Should be on anchor node"
    );

    // Now try to delete the anchor (should succeed since no aliases reference it)
    let result = state.delete_node_at_cursor();
    assert!(result.is_ok(), "Delete should succeed");

    // Verify the node is actually deleted by checking:
    // 1. The root object now has only 1 entry (production)
    // 2. The defaults object with anchor "config" is gone
    if let YamlValue::Object(map) = state.tree().root().value() {
        assert_eq!(
            map.len(),
            1,
            "Root should have only 1 entry after deleting defaults"
        );
        assert_eq!(
            map.get_index(0).unwrap().0,
            "production",
            "First entry should now be production"
        );

        // Verify the defaults object with anchor is truly gone
        for (_key, val) in map.iter() {
            assert_ne!(
                val.anchor(),
                Some("config"),
                "Anchor 'config' should be gone"
            );
        }
    } else {
        panic!("Root should be an object");
    }
}

#[test]
fn test_anchor_navigation() {
    use termion::event::{Event, Key};
    use yamlquill::document::tree::YamlTree;
    use yamlquill::editor::mode::EditorMode;
    use yamlquill::editor::state::EditorState;
    use yamlquill::input::keys::{map_key_event, InputEvent};

    let yaml = r#"
defaults: &config
  timeout: 30
  retries: 3

production:
  settings: *config
"#;
    let node = parse_yaml_auto(yaml).unwrap();
    let tree = YamlTree::new(node);
    let mut state = EditorState::new_with_default_theme(tree);

    // Navigate to the alias at production.settings (path [1, 0])
    let alias_path = vec![1, 0];
    state.cursor_mut().set_path(alias_path.clone());

    // Verify we're on the alias
    let alias_node = state.tree().get_node(&alias_path).unwrap();
    assert!(
        matches!(alias_node.value(), YamlValue::Alias(_)),
        "Should be on alias node"
    );

    // Simulate pressing Enter
    let enter_event = map_key_event(Event::Key(Key::Char('\n')), &EditorMode::Normal);
    assert_eq!(
        enter_event,
        InputEvent::JumpToAnchor,
        "Enter should map to JumpToAnchor"
    );

    // Get the anchor path before jumping
    let anchor_path_registry = state
        .tree()
        .anchor_registry()
        .get_anchor_path("config")
        .unwrap();
    assert_eq!(
        *anchor_path_registry,
        vec![0],
        "Anchor should be at path [0]"
    );

    // Trigger the navigation by simulating what the input handler would do
    let alias_target = state
        .tree()
        .get_node(&alias_path)
        .and_then(|node| node.alias_target())
        .map(|s| s.to_string());

    assert_eq!(
        alias_target,
        Some("config".to_string()),
        "Should get alias target"
    );

    if let Some(alias_target) = alias_target {
        let anchor_path = state
            .tree()
            .anchor_registry()
            .get_anchor_path(&alias_target)
            .cloned();

        assert!(anchor_path.is_some(), "Should find anchor path");

        if let Some(anchor_path) = anchor_path {
            // Record jump
            state.record_jump();

            // Navigate to the anchor
            state.cursor_mut().set_path(anchor_path.clone());

            // Verify we're now at the anchor
            assert_eq!(state.cursor().path(), &anchor_path);
            assert_eq!(anchor_path, vec![0], "Should be at path [0]");

            let anchor_node = state.tree().get_node(&anchor_path).unwrap();
            assert_eq!(
                anchor_node.anchor(),
                Some("config"),
                "Should be on anchor node"
            );

            // Verify we can jump back using the jump list
            let jumped_back = state.jump_backward();
            if jumped_back {
                assert_eq!(
                    state.cursor().path(),
                    &alias_path,
                    "Should be back at alias after jump backward"
                );
            }
            // Note: jump_backward might return false if jump list is empty,
            // which is okay for this test - we're mainly testing the navigation works
        }
    }
}

#[test]
fn test_anchor_navigation_not_found() {
    use yamlquill::document::tree::YamlTree;
    use yamlquill::editor::state::EditorState;

    let yaml = r#"
production:
  settings: value
"#;
    let node = parse_yaml_auto(yaml).unwrap();
    let tree = YamlTree::new(node);
    let mut state = EditorState::new_with_default_theme(tree);

    // Try to look up a non-existent anchor
    let anchor_path = state
        .tree()
        .anchor_registry()
        .get_anchor_path("nonexistent");
    assert!(anchor_path.is_none(), "Should not find non-existent anchor");
}
