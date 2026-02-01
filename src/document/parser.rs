//! YAML parsing with metadata preservation.
//!
//! This module provides functionality to parse YAML strings into `YamlNode` structures
//! using serde_yaml. The parser converts standard YAML into our internal representation
//! that tracks modification status for format-preserving edits.
//!
//! # Phase 1 Scope
//!
//! - Single document YAML only (multi-document support in Phase 3)
//! - All strings treated as Plain style (literal/folded detection in Phase 4)
//! - Basic value conversion without text span tracking (spans in Phase 4)
//!
//! # Example
//!
//! ```
//! use yamlquill::document::parser::parse_yaml;
//!
//! let yaml = "name: Alice\nage: 30";
//! let node = parse_yaml(yaml).unwrap();
//! ```

use crate::document::node::{YamlNode, YamlNumber, YamlString, YamlValue};
use anyhow::{Context, Result};
use indexmap::IndexMap;
use serde_yaml::{self, Value};

/// Parses a YAML string into a `YamlNode`.
///
/// This function uses `serde_yaml` to parse the YAML string, then converts
/// the result into our internal `YamlNode` structure with metadata tracking.
///
/// # Phase 1 Limitations
///
/// - Single document only (multi-document in Phase 3)
/// - All strings treated as Plain style (literal/folded detection in Phase 4)
/// - No text span tracking (added in Phase 4)
///
/// # Arguments
///
/// * `yaml_str` - A string slice containing valid YAML
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok(YamlNode)` if parsing succeeds
/// - `Err(anyhow::Error)` if the YAML is malformed
///
/// # Example
///
/// ```
/// use yamlquill::document::parser::parse_yaml;
/// use yamlquill::document::node::YamlValue;
///
/// let yaml = "name: Alice";
/// let node = parse_yaml(yaml).unwrap();
///
/// // Root should be an object
/// assert!(node.value().is_object());
/// ```
///
/// # Errors
///
/// This function will return an error if:
/// - The input string is not valid YAML
/// - The YAML contains syntax errors
/// - The YAML uses tagged values (not supported in v1)
pub fn parse_yaml(yaml_str: &str) -> Result<YamlNode> {
    let value: Value = serde_yaml::from_str(yaml_str).context("Failed to parse YAML")?;

    convert_value(value)
}

/// Converts a `serde_yaml::Value` into a `YamlNode`.
///
/// This is a recursive function that traverses the serde_yaml value tree
/// and converts each value into our internal representation with metadata.
///
/// # Arguments
///
/// * `value` - The `serde_yaml::Value` to convert
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok(YamlNode)` with the converted value
/// - `Err(anyhow::Error)` if the value type is not supported
///
/// # Type Conversions
///
/// - `Value::Null` → `YamlValue::Null`
/// - `Value::Bool` → `YamlValue::Boolean`
/// - `Value::Number` → `YamlValue::Number` (Integer or Float)
/// - `Value::String` → `YamlValue::String(YamlString::Plain)`
/// - `Value::Sequence` → `YamlValue::Array`
/// - `Value::Mapping` → `YamlValue::Object` (using IndexMap for order)
/// - `Value::Tagged` → Error (not supported in v1)
///
/// # Number Handling
///
/// Numbers are checked with `as_i64()` first to preserve integer types,
/// falling back to `as_f64()` for floating-point values.
fn convert_value(value: Value) -> Result<YamlNode> {
    let yaml_value = match value {
        Value::Null => YamlValue::Null,

        Value::Bool(b) => YamlValue::Boolean(b),

        Value::Number(n) => {
            // Try to preserve integer type
            if let Some(i) = n.as_i64() {
                YamlValue::Number(YamlNumber::Integer(i))
            } else if let Some(f) = n.as_f64() {
                YamlValue::Number(YamlNumber::Float(f))
            } else {
                // Fallback for u64 values that don't fit in i64
                YamlValue::Number(YamlNumber::Float(n.as_f64().unwrap_or(0.0)))
            }
        }

        Value::String(s) => {
            // Phase 1: Treat all strings as Plain
            // Phase 4 will add detection for Literal (|) and Folded (>)
            YamlValue::String(YamlString::Plain(s))
        }

        Value::Sequence(seq) => {
            let elements: Result<Vec<YamlNode>> = seq.into_iter().map(convert_value).collect();
            YamlValue::Array(elements?)
        }

        Value::Mapping(map) => {
            let mut entries = IndexMap::new();
            for (k, v) in map {
                // Convert key to string
                let key = match k {
                    Value::String(s) => s,
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => "null".to_string(),
                    _ => anyhow::bail!("Complex mapping keys are not supported"),
                };
                entries.insert(key, convert_value(v)?);
            }
            YamlValue::Object(entries)
        }

        Value::Tagged(tagged) => {
            anyhow::bail!("Tagged values are not supported in v1: !{}", tagged.tag)
        }
    };

    // Parsed nodes are marked as not modified
    Ok(YamlNode {
        value: yaml_value,
        metadata: crate::document::node::NodeMetadata {
            text_span: None,
            modified: false,
        },
        anchor: None,
        original_formatting: None,
    })
}

/// Parses YAML with automatic single/multi-document detection.
///
/// # Phase 1 Implementation
///
/// Currently just delegates to `parse_yaml()` for single-document support.
/// Phase 3 will add true multi-document support using `serde_yaml::Deserializer`.
///
/// # Arguments
///
/// * `yaml_str` - A string slice containing valid YAML
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok(YamlNode)` if parsing succeeds
/// - `Err(anyhow::Error)` if the YAML is malformed
pub fn parse_yaml_auto(yaml_str: &str) -> Result<YamlNode> {
    // V1: Single document only
    // Phase 3 will add multi-document support
    parse_yaml(yaml_str)
}

/// Converts a `serde_yaml::Value` reference into a `YamlNode`.
///
/// This is a compatibility function used by file loaders for multi-document YAML support.
/// It delegates to `convert_value` after cloning the value.
///
/// # Arguments
///
/// * `value` - A reference to the `serde_yaml::Value` to convert
///
/// # Returns
///
/// Returns a `YamlNode` with the converted value
pub fn parse_value(value: &Value) -> YamlNode {
    // Clone and convert - if conversion fails, create a null node
    convert_value(value.clone()).unwrap_or_else(|_| YamlNode {
        value: YamlValue::Null,
        metadata: crate::document::node::NodeMetadata {
            text_span: None,
            modified: false,
        },
        anchor: None,
        original_formatting: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_null() {
        let yaml = "null";
        let node = parse_yaml(yaml).unwrap();
        assert!(matches!(node.value(), YamlValue::Null));
        assert!(!node.is_modified());
    }

    #[test]
    fn test_parse_boolean_true() {
        let yaml = "true";
        let node = parse_yaml(yaml).unwrap();
        assert!(matches!(node.value(), YamlValue::Boolean(true)));
    }

    #[test]
    fn test_parse_boolean_false() {
        let yaml = "false";
        let node = parse_yaml(yaml).unwrap();
        assert!(matches!(node.value(), YamlValue::Boolean(false)));
    }

    #[test]
    fn test_parse_integer() {
        let yaml = "42";
        let node = parse_yaml(yaml).unwrap();
        match node.value() {
            YamlValue::Number(YamlNumber::Integer(i)) => assert_eq!(*i, 42),
            _ => panic!("Expected integer"),
        }
    }

    #[test]
    fn test_parse_negative_integer() {
        let yaml = "-100";
        let node = parse_yaml(yaml).unwrap();
        match node.value() {
            YamlValue::Number(YamlNumber::Integer(i)) => assert_eq!(*i, -100),
            _ => panic!("Expected integer"),
        }
    }

    #[test]
    fn test_parse_float() {
        let yaml = "3.14";
        let node = parse_yaml(yaml).unwrap();
        match node.value() {
            YamlValue::Number(YamlNumber::Float(f)) => assert_eq!(*f, 3.14),
            _ => panic!("Expected float"),
        }
    }

    #[test]
    fn test_parse_string() {
        let yaml = r#""hello world""#;
        let node = parse_yaml(yaml).unwrap();
        match node.value() {
            YamlValue::String(YamlString::Plain(s)) => assert_eq!(s, "hello world"),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_parse_plain_string() {
        let yaml = "unquoted string";
        let node = parse_yaml(yaml).unwrap();
        match node.value() {
            YamlValue::String(YamlString::Plain(s)) => assert_eq!(s, "unquoted string"),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_parse_empty_array() {
        let yaml = "[]";
        let node = parse_yaml(yaml).unwrap();
        match node.value() {
            YamlValue::Array(elements) => assert_eq!(elements.len(), 0),
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_parse_array_with_elements() {
        let yaml = "[1, 2, 3]";
        let node = parse_yaml(yaml).unwrap();
        match node.value() {
            YamlValue::Array(elements) => {
                assert_eq!(elements.len(), 3);
                match elements[0].value() {
                    YamlValue::Number(YamlNumber::Integer(i)) => assert_eq!(*i, 1),
                    _ => panic!("Expected integer"),
                }
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_parse_empty_object() {
        let yaml = "{}";
        let node = parse_yaml(yaml).unwrap();
        match node.value() {
            YamlValue::Object(map) => assert_eq!(map.len(), 0),
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_parse_object_with_fields() {
        let yaml = "name: Alice\nage: 30";
        let node = parse_yaml(yaml).unwrap();
        match node.value() {
            YamlValue::Object(map) => {
                assert_eq!(map.len(), 2);

                let name = map.get("name").unwrap();
                match name.value() {
                    YamlValue::String(YamlString::Plain(s)) => assert_eq!(s, "Alice"),
                    _ => panic!("Expected string"),
                }

                let age = map.get("age").unwrap();
                match age.value() {
                    YamlValue::Number(YamlNumber::Integer(i)) => assert_eq!(*i, 30),
                    _ => panic!("Expected integer"),
                }
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_parse_nested_object() {
        let yaml = r#"
user:
  name: Bob
  email: bob@example.com
"#;
        let node = parse_yaml(yaml).unwrap();
        match node.value() {
            YamlValue::Object(map) => {
                let user = map.get("user").unwrap();
                match user.value() {
                    YamlValue::Object(user_map) => {
                        assert_eq!(user_map.len(), 2);
                        assert!(user_map.contains_key("name"));
                        assert!(user_map.contains_key("email"));
                    }
                    _ => panic!("Expected nested object"),
                }
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_parse_array_of_objects() {
        let yaml = r#"
- name: Alice
  age: 30
- name: Bob
  age: 25
"#;
        let node = parse_yaml(yaml).unwrap();
        match node.value() {
            YamlValue::Array(elements) => {
                assert_eq!(elements.len(), 2);

                match elements[0].value() {
                    YamlValue::Object(map) => {
                        assert_eq!(map.len(), 2);
                    }
                    _ => panic!("Expected object in array"),
                }
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_parse_preserves_key_order() {
        let yaml = "z: 1\na: 2\nm: 3";
        let node = parse_yaml(yaml).unwrap();
        match node.value() {
            YamlValue::Object(map) => {
                let keys: Vec<&String> = map.keys().collect();
                assert_eq!(keys, vec!["z", "a", "m"]);
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_parse_nodes_not_modified() {
        let yaml = "name: Alice";
        let node = parse_yaml(yaml).unwrap();
        assert!(!node.is_modified());
    }

    #[test]
    fn test_parse_invalid_yaml() {
        let invalid = "{ invalid yaml: [";
        let result = parse_yaml(invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_yaml_auto_single_doc() {
        let yaml = "name: Alice";
        let node = parse_yaml_auto(yaml).unwrap();
        match node.value() {
            YamlValue::Object(map) => {
                assert_eq!(map.len(), 1);
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_convert_value_null() {
        let value = Value::Null;
        let node = convert_value(value).unwrap();
        assert!(matches!(node.value(), YamlValue::Null));
    }

    #[test]
    fn test_convert_value_bool() {
        let value = Value::Bool(true);
        let node = convert_value(value).unwrap();
        assert!(matches!(node.value(), YamlValue::Boolean(true)));
    }

    #[test]
    fn test_convert_value_integer() {
        let value = Value::Number(serde_yaml::Number::from(42));
        let node = convert_value(value).unwrap();
        match node.value() {
            YamlValue::Number(YamlNumber::Integer(i)) => assert_eq!(*i, 42),
            _ => panic!("Expected integer"),
        }
    }

    #[test]
    fn test_convert_value_float() {
        let value = Value::Number(serde_yaml::Number::from(3.14));
        let node = convert_value(value).unwrap();
        match node.value() {
            YamlValue::Number(YamlNumber::Float(f)) => assert!((f - 3.14).abs() < 0.001),
            _ => panic!("Expected float"),
        }
    }

    #[test]
    fn test_convert_value_string() {
        let value = Value::String("hello".to_string());
        let node = convert_value(value).unwrap();
        match node.value() {
            YamlValue::String(YamlString::Plain(s)) => assert_eq!(s, "hello"),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_parse_multiline_string_as_plain() {
        // Phase 1: All strings are Plain, even if they were literal/folded
        let yaml = r#"
description: |
  This is a
  literal block
"#;
        let node = parse_yaml(yaml).unwrap();
        match node.value() {
            YamlValue::Object(map) => {
                let desc = map.get("description").unwrap();
                match desc.value() {
                    YamlValue::String(YamlString::Plain(s)) => {
                        assert!(s.contains("This is a"));
                    }
                    _ => panic!("Expected plain string (Phase 1)"),
                }
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_parse_numeric_key() {
        let yaml = "123: value";
        let node = parse_yaml(yaml).unwrap();
        match node.value() {
            YamlValue::Object(map) => {
                assert!(map.contains_key("123"));
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_parse_boolean_key() {
        let yaml = "true: yes\nfalse: no";
        let node = parse_yaml(yaml).unwrap();
        match node.value() {
            YamlValue::Object(map) => {
                assert!(map.contains_key("true"));
                assert!(map.contains_key("false"));
            }
            _ => panic!("Expected object"),
        }
    }
}
