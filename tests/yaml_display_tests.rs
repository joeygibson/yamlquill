//! Tests for YAML-aware display features (Phase 2e)
//!
//! Validates that the tree view correctly displays YAML type indicators:
//! - Plain strings: "text"
//! - Literal strings: | multiline
//! - Folded strings: > folded
//! - Integers vs Floats
//! - Booleans and null

use indexmap::IndexMap;
use yamlquill::document::node::{YamlNode, YamlNumber, YamlString, YamlValue};
use yamlquill::document::tree::YamlTree;
use yamlquill::ui::tree_view::{format_collapsed_preview, TreeViewState};

#[test]
fn test_plain_string_display() {
    let node = YamlNode::new(YamlValue::String(YamlString::Plain("hello".to_string())));
    let preview = format_collapsed_preview(&node, 100);
    assert_eq!(
        preview, "\"hello\"",
        "Plain strings should show with quotes"
    );
}

#[test]
fn test_literal_string_single_line_display() {
    let node = YamlNode::new(YamlValue::String(YamlString::Literal(
        "single line".to_string(),
    )));
    let preview = format_collapsed_preview(&node, 100);
    assert_eq!(
        preview, "| single line",
        "Single-line literal should show | prefix"
    );
}

#[test]
fn test_literal_string_multiline_display() {
    let content = "line1\nline2\nline3".to_string();
    let node = YamlNode::new(YamlValue::String(YamlString::Literal(content)));
    let preview = format_collapsed_preview(&node, 100);
    assert_eq!(
        preview, "| line1...",
        "Multi-line literal should show | prefix with first line and ..."
    );
}

#[test]
fn test_folded_string_single_line_display() {
    let node = YamlNode::new(YamlValue::String(YamlString::Folded(
        "single line".to_string(),
    )));
    let preview = format_collapsed_preview(&node, 100);
    assert_eq!(
        preview, "> single line",
        "Single-line folded should show > prefix"
    );
}

#[test]
fn test_folded_string_multiline_display() {
    let content = "line1\nline2\nline3".to_string();
    let node = YamlNode::new(YamlValue::String(YamlString::Folded(content)));
    let preview = format_collapsed_preview(&node, 100);
    assert_eq!(
        preview, "> line1...",
        "Multi-line folded should show > prefix with first line and ..."
    );
}

#[test]
fn test_integer_display() {
    let node = YamlNode::new(YamlValue::Number(YamlNumber::Integer(42)));
    let preview = format_collapsed_preview(&node, 100);
    assert_eq!(
        preview, "42",
        "Integer should display without decimal point"
    );
}

#[test]
fn test_float_display() {
    let node = YamlNode::new(YamlValue::Number(YamlNumber::Float(3.14)));
    let preview = format_collapsed_preview(&node, 100);
    assert_eq!(preview, "3.14", "Float should display with decimal point");
}

#[test]
fn test_float_whole_number_display() {
    let node = YamlNode::new(YamlValue::Number(YamlNumber::Float(42.0)));
    let preview = format_collapsed_preview(&node, 100);
    assert_eq!(
        preview, "42",
        "Float with .0 should display without decimal"
    );
}

#[test]
fn test_boolean_true_display() {
    let node = YamlNode::new(YamlValue::Boolean(true));
    let preview = format_collapsed_preview(&node, 100);
    assert_eq!(preview, "true", "Boolean true should display as lowercase");
}

#[test]
fn test_boolean_false_display() {
    let node = YamlNode::new(YamlValue::Boolean(false));
    let preview = format_collapsed_preview(&node, 100);
    assert_eq!(
        preview, "false",
        "Boolean false should display as lowercase"
    );
}

#[test]
fn test_null_display() {
    let node = YamlNode::new(YamlValue::Null);
    let preview = format_collapsed_preview(&node, 100);
    assert_eq!(preview, "null", "Null should display as lowercase 'null'");
}

#[test]
fn test_alias_display() {
    let node = YamlNode::new(YamlValue::Alias("anchor_name".to_string()));
    let preview = format_collapsed_preview(&node, 100);
    assert_eq!(
        preview, "*anchor_name",
        "Alias should display with * prefix"
    );
}

#[test]
fn test_tree_view_shows_string_styles() {
    // Create tree with different string styles
    let mut obj = IndexMap::new();
    obj.insert(
        "plain".to_string(),
        YamlNode::new(YamlValue::String(YamlString::Plain(
            "plain text".to_string(),
        ))),
    );
    obj.insert(
        "literal".to_string(),
        YamlNode::new(YamlValue::String(YamlString::Literal(
            "line1\nline2".to_string(),
        ))),
    );
    obj.insert(
        "folded".to_string(),
        YamlNode::new(YamlValue::String(YamlString::Folded(
            "folded\ntext".to_string(),
        ))),
    );

    let tree = YamlTree::new(YamlNode::new(YamlValue::Object(obj)));
    let mut view_state = TreeViewState::new();
    view_state.rebuild(&tree);

    let lines = view_state.lines();
    assert_eq!(lines.len(), 3, "Should have 3 lines for 3 keys");

    // Check plain string
    assert_eq!(lines[0].key, Some("plain".to_string()));
    assert_eq!(lines[0].value_preview, "\"plain text\"");

    // Check literal string
    assert_eq!(lines[1].key, Some("literal".to_string()));
    assert_eq!(lines[1].value_preview, "| line1...");

    // Check folded string
    assert_eq!(lines[2].key, Some("folded".to_string()));
    assert_eq!(lines[2].value_preview, "> folded...");
}

#[test]
fn test_tree_view_shows_number_types() {
    // Create tree with different number types
    let mut obj = IndexMap::new();
    obj.insert(
        "integer".to_string(),
        YamlNode::new(YamlValue::Number(YamlNumber::Integer(42))),
    );
    obj.insert(
        "float".to_string(),
        YamlNode::new(YamlValue::Number(YamlNumber::Float(3.14))),
    );

    let tree = YamlTree::new(YamlNode::new(YamlValue::Object(obj)));
    let mut view_state = TreeViewState::new();
    view_state.rebuild(&tree);

    let lines = view_state.lines();
    assert_eq!(lines.len(), 2);

    // Check integer
    assert_eq!(lines[0].key, Some("integer".to_string()));
    assert_eq!(lines[0].value_preview, "42");

    // Check float
    assert_eq!(lines[1].key, Some("float".to_string()));
    assert_eq!(lines[1].value_preview, "3.14");
}

#[test]
fn test_empty_literal_string() {
    let node = YamlNode::new(YamlValue::String(YamlString::Literal("".to_string())));
    let preview = format_collapsed_preview(&node, 100);
    assert_eq!(
        preview, "| ",
        "Empty literal should show | prefix with space"
    );
}

#[test]
fn test_empty_folded_string() {
    let node = YamlNode::new(YamlValue::String(YamlString::Folded("".to_string())));
    let preview = format_collapsed_preview(&node, 100);
    assert_eq!(
        preview, "> ",
        "Empty folded should show > prefix with space"
    );
}
