//! Tests for anchor and alias extraction using Scanner
//!
//! These tests validate the hybrid parsing approach that uses Scanner
//! to extract anchor/alias names and correlates them with the parsed tree.

use yamlquill::document::parser::parse_yaml_auto;
use yamlquill::document::node::{YamlValue, YamlString, YamlNumber};

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
            assert_eq!(defaults.anchor(), Some("config"), "Anchor should be 'config'");
            
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
