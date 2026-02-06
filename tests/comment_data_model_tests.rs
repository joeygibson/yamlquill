// tests/comment_data_model_tests.rs
use yamlquill::document::node::{CommentNode, CommentPosition, YamlNode, YamlValue};

#[test]
fn test_comment_node_creation() {
    let comment = CommentNode {
        content: "Test comment".to_string(),
        position: CommentPosition::Above,
        source_line: None,
    };
    assert_eq!(comment.content, "Test comment");
    assert!(matches!(comment.position, CommentPosition::Above));
}

#[test]
fn test_comment_value_variant() {
    let comment_node = CommentNode {
        content: "Line comment".to_string(),
        position: CommentPosition::Line,
        source_line: None,
    };
    let value = YamlValue::Comment(comment_node);
    assert!(matches!(value, YamlValue::Comment(_)));
}

#[test]
fn test_yaml_node_is_comment() {
    let comment = YamlNode::new(YamlValue::Comment(CommentNode {
        content: "Test".to_string(),
        position: CommentPosition::Standalone,
        source_line: None,
    }));
    assert!(comment.is_comment());

    let string = YamlNode::new(YamlValue::String(
        yamlquill::document::node::YamlString::Plain("text".to_string()),
    ));
    assert!(!string.is_comment());
}

#[test]
fn test_comment_position_variants() {
    let above = CommentPosition::Above;
    let line = CommentPosition::Line;
    let below = CommentPosition::Below;
    let standalone = CommentPosition::Standalone;

    assert!(matches!(above, CommentPosition::Above));
    assert!(matches!(line, CommentPosition::Line));
    assert!(matches!(below, CommentPosition::Below));
    assert!(matches!(standalone, CommentPosition::Standalone));
}
