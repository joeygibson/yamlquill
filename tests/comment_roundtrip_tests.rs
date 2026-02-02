//! Roundtrip tests for comment preservation when saving YAML files.
//!
//! These tests verify that comments are properly preserved through the
//! load -> edit -> save -> load cycle.

use std::fs;
use tempfile::NamedTempFile;
use yamlquill::config::Config;
use yamlquill::document::node::{CommentPosition, YamlValue};
use yamlquill::document::parser::parse_yaml_auto;
use yamlquill::document::tree::YamlTree;
use yamlquill::file::saver::save_yaml_file;

/// Test 1: Roundtrip comment above a value
#[test]
fn test_roundtrip_comment_above() {
    let yaml = r#"# This is a comment above name
name: Alice
age: 30
"#;

    // Parse
    let root = parse_yaml_auto(yaml).unwrap();
    let tree = YamlTree::new(root);

    // Save
    let temp_file = NamedTempFile::new().unwrap();
    let config = Config::default();
    save_yaml_file(temp_file.path(), &tree, &config).unwrap();

    // Read back
    let saved = fs::read_to_string(temp_file.path()).unwrap();

    // Re-parse
    let reloaded = parse_yaml_auto(&saved).unwrap();

    // Verify comment is preserved
    match reloaded.value() {
        YamlValue::Object(map) => {
            let mut has_comment = false;
            for (key, val) in map.iter() {
                if key.starts_with("__comment_") {
                    has_comment = true;
                    match val.value() {
                        YamlValue::Comment(comment) => {
                            assert_eq!(comment.content(), "This is a comment above name");
                            assert_eq!(comment.position(), &CommentPosition::Above);
                        }
                        _ => panic!("Expected Comment value"),
                    }
                }
            }
            assert!(has_comment, "Comment was not preserved through roundtrip");
            assert_eq!(map.get("name").unwrap().value().to_string(), "Alice");
            assert_eq!(map.get("age").unwrap().value().to_string(), "30");
        }
        _ => panic!("Expected object"),
    }
}

/// Test 2: Roundtrip inline comment
#[test]
fn test_roundtrip_comment_inline() {
    let yaml = r#"name: Alice  # inline comment
age: 30
"#;

    // Parse
    let root = parse_yaml_auto(yaml).unwrap();
    let tree = YamlTree::new(root);

    // Save
    let temp_file = NamedTempFile::new().unwrap();
    let config = Config::default();
    save_yaml_file(temp_file.path(), &tree, &config).unwrap();

    // Read back
    let saved = fs::read_to_string(temp_file.path()).unwrap();

    // Re-parse
    let reloaded = parse_yaml_auto(&saved).unwrap();

    // Verify inline comment is preserved
    match reloaded.value() {
        YamlValue::Object(map) => {
            let mut has_inline_comment = false;
            for (key, val) in map.iter() {
                if key.starts_with("__comment_") {
                    match val.value() {
                        YamlValue::Comment(comment) => {
                            if comment.position() == &CommentPosition::Line {
                                has_inline_comment = true;
                                assert_eq!(comment.content(), "inline comment");
                            }
                        }
                        _ => {}
                    }
                }
            }
            assert!(
                has_inline_comment,
                "Inline comment was not preserved through roundtrip"
            );
        }
        _ => panic!("Expected object"),
    }
}

/// Test 3: Roundtrip multiple comments
#[test]
fn test_roundtrip_multiple_comments() {
    let yaml = r#"# Comment 1 above name
name: Alice  # inline comment
# Comment 2 above age
age: 30
"#;

    // Parse
    let root = parse_yaml_auto(yaml).unwrap();
    let tree = YamlTree::new(root);

    // Save
    let temp_file = NamedTempFile::new().unwrap();
    let config = Config::default();
    save_yaml_file(temp_file.path(), &tree, &config).unwrap();

    // Read back
    let saved = fs::read_to_string(temp_file.path()).unwrap();

    // Re-parse
    let reloaded = parse_yaml_auto(&saved).unwrap();

    // Verify all comments are preserved
    match reloaded.value() {
        YamlValue::Object(map) => {
            let mut comment_count = 0;
            let mut has_above = false;
            let mut has_inline = false;

            for (key, val) in map.iter() {
                if key.starts_with("__comment_") {
                    comment_count += 1;
                    match val.value() {
                        YamlValue::Comment(comment) => match comment.position() {
                            CommentPosition::Above => {
                                has_above = true;
                            }
                            CommentPosition::Line => {
                                has_inline = true;
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }

            assert_eq!(
                comment_count, 3,
                "Expected 3 comments to be preserved, found {}",
                comment_count
            );
            assert!(has_above, "Expected at least one Above comment");
            assert!(has_inline, "Expected at least one Line comment");
        }
        _ => panic!("Expected object"),
    }
}

/// Test 4: Roundtrip comments in arrays
#[test]
fn test_roundtrip_comments_in_array() {
    let yaml = r#"items:
  # Comment above first item
  - name: Item1
  - name: Item2  # inline comment
"#;

    // Parse
    let root = parse_yaml_auto(yaml).unwrap();
    let tree = YamlTree::new(root);

    // Save
    let temp_file = NamedTempFile::new().unwrap();
    let config = Config::default();
    save_yaml_file(temp_file.path(), &tree, &config).unwrap();

    // Read back
    let saved = fs::read_to_string(temp_file.path()).unwrap();

    // Re-parse
    let reloaded = parse_yaml_auto(&saved).unwrap();

    // Verify structure and comments are preserved
    match reloaded.value() {
        YamlValue::Object(map) => {
            assert!(map.contains_key("items"));
            // Comments in arrays should be preserved somehow
            // This test will evolve as we implement array comment support
        }
        _ => panic!("Expected object"),
    }
}
