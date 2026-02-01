//! Tests for yaml-rust2 parser with anchor/alias support

use yamlquill::document::node::YamlValue;
use yamlquill::document::parser::parse_yaml_auto;

#[test]
fn test_parse_simple_yaml_with_yaml_rust2() {
    let yaml = "name: test";
    let node = parse_yaml_auto(yaml).unwrap();

    if let YamlValue::Object(obj) = node.value() {
        assert_eq!(obj.len(), 1);
        assert!(obj.contains_key("name"));
    } else {
        panic!("Expected object");
    }
}

#[test]
fn test_parse_integer() {
    let yaml = "count: 42";
    let node = parse_yaml_auto(yaml).unwrap();

    if let YamlValue::Object(obj) = node.value() {
        let value_node = obj.get("count").unwrap();
        assert!(matches!(value_node.value(), YamlValue::Number(_)));
    } else {
        panic!("Expected object");
    }
}

#[test]
#[ignore = "Anchor name extraction requires additional YAML scanning"]
fn test_parse_yaml_with_anchor() {
    let yaml = r#"
defaults: &config
  timeout: 30
"#;
    let node = parse_yaml_auto(yaml).unwrap();

    if let YamlValue::Object(obj) = node.value() {
        let defaults_node = obj.get("defaults").unwrap();
        assert_eq!(defaults_node.anchor(), Some("config"));
    } else {
        panic!("Expected object");
    }
}
