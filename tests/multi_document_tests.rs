//! Tests for multi-document YAML support (Phase 3)
//!
//! Validates:
//! - Parsing multiple YAML documents from a single file
//! - Saving multi-document YAML with --- separators
//! - Round-trip preservation of multi-document structure

use std::fs;
use tempfile::TempDir;
use yamlquill::config::Config;
use yamlquill::document::node::{YamlNumber, YamlString, YamlValue};
use yamlquill::document::parser::parse_yaml_auto;
use yamlquill::document::tree::YamlTree;
use yamlquill::file::saver::save_yaml_file;

#[test]
fn test_parse_single_document() {
    let yaml = "key: value";
    let node = parse_yaml_auto(yaml).unwrap();

    // Single document should NOT be wrapped in MultiDoc
    assert!(!matches!(node.value(), YamlValue::MultiDoc(_)));
    assert!(matches!(node.value(), YamlValue::Object(_)));
}

#[test]
fn test_parse_multi_document() {
    let yaml = r#"---
name: first
value: 1
---
name: second
value: 2
---
name: third
value: 3
"#;

    let node = parse_yaml_auto(yaml).unwrap();

    // Multiple documents should be wrapped in MultiDoc
    match node.value() {
        YamlValue::MultiDoc(docs) => {
            assert_eq!(docs.len(), 3);

            // Check first document
            if let YamlValue::Object(obj) = docs[0].value() {
                if let Some(name_node) = obj.get("name") {
                    if let YamlValue::String(YamlString::Plain(s)) = name_node.value() {
                        assert_eq!(s, "first");
                    } else {
                        panic!("Expected Plain string for name");
                    }
                } else {
                    panic!("Expected 'name' key in first document");
                }
            } else {
                panic!("Expected Object for first document");
            }

            // Check second document
            if let YamlValue::Object(obj) = docs[1].value() {
                if let Some(name_node) = obj.get("name") {
                    if let YamlValue::String(YamlString::Plain(s)) = name_node.value() {
                        assert_eq!(s, "second");
                    } else {
                        panic!("Expected Plain string for name");
                    }
                } else {
                    panic!("Expected 'name' key in second document");
                }
            } else {
                panic!("Expected Object for second document");
            }
        }
        _ => panic!("Expected MultiDoc variant"),
    }
}

#[test]
fn test_parse_multi_document_with_scalars() {
    let yaml = r#"---
42
---
"hello"
---
true
"#;

    let node = parse_yaml_auto(yaml).unwrap();

    match node.value() {
        YamlValue::MultiDoc(docs) => {
            assert_eq!(docs.len(), 3);

            assert!(matches!(
                docs[0].value(),
                YamlValue::Number(YamlNumber::Integer(42))
            ));

            if let YamlValue::String(YamlString::Plain(s)) = docs[1].value() {
                assert_eq!(s, "hello");
            } else {
                panic!("Expected Plain string");
            }

            assert!(matches!(docs[2].value(), YamlValue::Boolean(true)));
        }
        _ => panic!("Expected MultiDoc variant"),
    }
}

#[test]
fn test_parse_multi_document_with_arrays() {
    let yaml = r#"---
- item1
- item2
---
- item3
- item4
"#;

    let node = parse_yaml_auto(yaml).unwrap();

    match node.value() {
        YamlValue::MultiDoc(docs) => {
            assert_eq!(docs.len(), 2);

            if let YamlValue::Array(arr) = docs[0].value() {
                assert_eq!(arr.len(), 2);
            } else {
                panic!("Expected Array for first document");
            }

            if let YamlValue::Array(arr) = docs[1].value() {
                assert_eq!(arr.len(), 2);
            } else {
                panic!("Expected Array for second document");
            }
        }
        _ => panic!("Expected MultiDoc variant"),
    }
}

#[test]
fn test_empty_documents_parsed_as_null() {
    // serde_yaml parses empty documents as Null
    let yaml = r#"---
key: value
---
---
key2: value2
"#;

    let node = parse_yaml_auto(yaml).unwrap();

    match node.value() {
        YamlValue::MultiDoc(docs) => {
            // Empty document is parsed as Null
            assert_eq!(docs.len(), 3);
            assert!(matches!(docs[1].value(), YamlValue::Null));
        }
        _ => panic!("Expected MultiDoc variant"),
    }
}

#[test]
fn test_roundtrip_multi_document() {
    let yaml = r#"---
name: first
value: 1
---
name: second
value: 2
---
name: third
value: 3
"#;

    // Parse the multi-document YAML
    let node = parse_yaml_auto(yaml).unwrap();
    let tree = YamlTree::new(node);

    // Save to a temporary file
    let temp_dir = TempDir::new().unwrap();
    let temp_file = temp_dir.path().join("test.yaml");
    let config = Config::default();

    save_yaml_file(&temp_file, &tree, &config).unwrap();

    // Read back the file
    let saved_content = fs::read_to_string(&temp_file).unwrap();

    // Parse the saved content
    let reparsed = parse_yaml_auto(&saved_content).unwrap();

    // Verify it's still a MultiDoc with 3 documents
    match reparsed.value() {
        YamlValue::MultiDoc(docs) => {
            assert_eq!(docs.len(), 3);

            // Verify first document
            if let YamlValue::Object(obj) = docs[0].value() {
                if let Some(name_node) = obj.get("name") {
                    if let YamlValue::String(YamlString::Plain(s)) = name_node.value() {
                        assert_eq!(s, "first");
                    }
                }
            }
        }
        _ => panic!("Expected MultiDoc after round-trip"),
    }
}

#[test]
fn test_save_multi_document_has_separators() {
    let yaml = r#"---
one: 1
---
two: 2
"#;

    let node = parse_yaml_auto(yaml).unwrap();
    let tree = YamlTree::new(node);

    let temp_dir = TempDir::new().unwrap();
    let temp_file = temp_dir.path().join("test.yaml");
    let config = Config::default();

    save_yaml_file(&temp_file, &tree, &config).unwrap();

    let saved_content = fs::read_to_string(&temp_file).unwrap();

    // Verify the file contains --- separators
    assert!(
        saved_content.contains("---"),
        "Saved file should contain --- separators"
    );

    // Count the number of --- separators
    let separator_count = saved_content.matches("---").count();
    assert_eq!(separator_count, 2, "Should have 2 document separators");
}

#[test]
fn test_roundtrip_multi_document_with_scalars() {
    let yaml = r#"---
42
---
"hello"
---
true
"#;

    let node = parse_yaml_auto(yaml).unwrap();
    let tree = YamlTree::new(node);

    let temp_dir = TempDir::new().unwrap();
    let temp_file = temp_dir.path().join("test.yaml");
    let config = Config::default();

    save_yaml_file(&temp_file, &tree, &config).unwrap();

    let saved_content = fs::read_to_string(&temp_file).unwrap();
    let reparsed = parse_yaml_auto(&saved_content).unwrap();

    match reparsed.value() {
        YamlValue::MultiDoc(docs) => {
            assert_eq!(docs.len(), 3);
            assert!(matches!(
                docs[0].value(),
                YamlValue::Number(YamlNumber::Integer(42))
            ));
            assert!(matches!(docs[2].value(), YamlValue::Boolean(true)));
        }
        _ => panic!("Expected MultiDoc after round-trip"),
    }
}
