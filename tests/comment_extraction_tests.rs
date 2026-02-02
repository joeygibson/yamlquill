//! Tests for comment extraction during YAML parsing.
//!
//! These tests verify that comments are properly extracted from YAML source
//! and inserted into the tree as Comment nodes.

use yamlquill::document::node::{CommentPosition, YamlValue};
use yamlquill::document::parser::parse_yaml_auto;

/// Test 1: Comment above a scalar value
#[test]
fn test_comment_above_scalar() {
    let yaml = r#"
# This is a comment above name
name: Alice
"#;

    let node = parse_yaml_auto(yaml).unwrap();

    // Should have an object with comment and name
    match node.value() {
        YamlValue::Object(map) => {
            // Look for comment node with special key
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
            assert!(has_comment, "Expected to find comment node in map");
            assert!(map.contains_key("name"));
        }
        _ => panic!("Expected object"),
    }
}

/// Test 2: Inline comment on same line as value
#[test]
fn test_comment_inline() {
    let yaml = r#"
name: Alice  # inline comment
age: 30
"#;

    let node = parse_yaml_auto(yaml).unwrap();

    match node.value() {
        YamlValue::Object(map) => {
            // Look for comment node with Line position
            let mut has_comment = false;
            for (key, val) in map.iter() {
                if key.starts_with("__comment_") {
                    has_comment = true;
                    match val.value() {
                        YamlValue::Comment(comment) => {
                            assert_eq!(comment.content(), "inline comment");
                            assert_eq!(comment.position(), &CommentPosition::Line);
                        }
                        _ => panic!("Expected Comment value"),
                    }
                }
            }
            assert!(has_comment, "Expected to find inline comment");
        }
        _ => panic!("Expected object"),
    }
}

/// Test 3: Standalone comment (between blank lines)
#[test]
fn test_comment_standalone() {
    let yaml = r#"
name: Alice

# This is a standalone comment

age: 30
"#;

    let node = parse_yaml_auto(yaml).unwrap();

    match node.value() {
        YamlValue::Object(map) => {
            // Look for standalone comment
            let mut has_standalone = false;
            for (key, val) in map.iter() {
                if key.starts_with("__comment_") {
                    match val.value() {
                        YamlValue::Comment(comment) => {
                            if comment.position() == &CommentPosition::Standalone {
                                has_standalone = true;
                                assert_eq!(comment.content(), "This is a standalone comment");
                            }
                        }
                        _ => {}
                    }
                }
            }
            assert!(has_standalone, "Expected to find standalone comment");
        }
        _ => panic!("Expected object"),
    }
}

/// Test 4: Comments in arrays
#[test]
fn test_comment_in_array() {
    let yaml = r#"
items:
  # Comment above first item
  - apple
  - banana  # inline comment on banana
  - cherry
"#;

    let node = parse_yaml_auto(yaml).unwrap();

    match node.value() {
        YamlValue::Object(map) => {
            let items = map.get("items").expect("Expected items key");
            match items.value() {
                YamlValue::Array(elements) => {
                    // Check for comment nodes in array
                    let mut has_above_comment = false;
                    let mut has_inline_comment = false;

                    for elem in elements {
                        match elem.value() {
                            YamlValue::Comment(comment) => {
                                if comment.position() == &CommentPosition::Above {
                                    has_above_comment = true;
                                    assert_eq!(comment.content(), "Comment above first item");
                                } else if comment.position() == &CommentPosition::Line {
                                    has_inline_comment = true;
                                    assert_eq!(comment.content(), "inline comment on banana");
                                }
                            }
                            _ => {}
                        }
                    }

                    assert!(has_above_comment, "Expected Above comment in array");
                    assert!(has_inline_comment, "Expected Line comment in array");
                }
                _ => panic!("Expected array for items"),
            }
        }
        _ => panic!("Expected object"),
    }
}

/// Test 5: Hashes in strings should not be treated as comments
#[test]
fn test_hash_in_quoted_strings() {
    let yaml = r#"
# Real comment
url: "http://example.com#fragment"
tag: 'value with # hash'
normal: value  # real comment
"#;

    let node = parse_yaml_auto(yaml).unwrap();

    match node.value() {
        YamlValue::Object(map) => {
            // Check that URL and tag values contain the hash character
            let url = map.get("url").expect("Expected url key");
            match url.value() {
                YamlValue::String(s) => {
                    let url_str = s.to_string();
                    assert!(
                        url_str.contains("#fragment"),
                        "URL should contain #fragment, got: {}",
                        url_str
                    );
                }
                _ => panic!("Expected string value for url"),
            }

            let tag = map.get("tag").expect("Expected tag key");
            match tag.value() {
                YamlValue::String(s) => {
                    let tag_str = s.to_string();
                    assert!(
                        tag_str.contains("# hash"),
                        "Tag should contain # hash, got: {}",
                        tag_str
                    );
                }
                _ => panic!("Expected string value for tag"),
            }

            // Check that we only have 2 real comments (not 4)
            let comment_count = map
                .iter()
                .filter(|(key, _)| key.starts_with("__comment_"))
                .count();
            assert_eq!(comment_count, 2, "Should have exactly 2 comments, not 4");

            // Verify the actual comment contents
            let mut found_real_comment = false;
            let mut found_inline_comment = false;

            for (key, val) in map.iter() {
                if key.starts_with("__comment_") {
                    match val.value() {
                        YamlValue::Comment(comment) => {
                            let content = comment.content();
                            if content.contains("Real comment") {
                                found_real_comment = true;
                            } else if content.contains("real comment") {
                                found_inline_comment = true;
                            }
                        }
                        _ => {}
                    }
                }
            }

            assert!(found_real_comment, "Should find 'Real comment'");
            assert!(found_inline_comment, "Should find 'real comment'");
        }
        _ => panic!("Expected object"),
    }
}
