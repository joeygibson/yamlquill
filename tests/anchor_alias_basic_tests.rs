//! Basic tests for anchor/alias data model

use yamlquill::document::node::{YamlNode, YamlValue};

#[test]
fn test_yaml_node_has_alias_target_field() {
    let mut node = YamlNode::new(YamlValue::Null);

    // Should be None initially
    assert!(node.alias_target().is_none());

    // Should be settable
    node.set_alias_target(Some("test_anchor".to_string()));
    assert_eq!(node.alias_target(), Some("test_anchor"));
}
