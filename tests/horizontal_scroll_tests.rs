use indexmap::IndexMap;
use yamlquill::document::node::{YamlNode, YamlNumber, YamlString, YamlValue};
use yamlquill::document::tree::YamlTree;
use yamlquill::editor::state::EditorState;

#[test]
fn test_horizontal_offset_defaults_to_zero() {
    let tree = YamlTree::new(YamlNode::new(YamlValue::Object(IndexMap::new())));
    let state = EditorState::new_with_default_theme(tree);
    assert_eq!(state.horizontal_offset(), 0);
}

#[test]
fn test_set_horizontal_offset() {
    let tree = YamlTree::new(YamlNode::new(YamlValue::Object(IndexMap::new())));
    let mut state = EditorState::new_with_default_theme(tree);
    state.set_horizontal_offset(10);
    assert_eq!(state.horizontal_offset(), 10);
}

#[test]
fn test_reset_horizontal_offset() {
    let tree = YamlTree::new(YamlNode::new(YamlValue::Object(IndexMap::new())));
    let mut state = EditorState::new_with_default_theme(tree);
    state.set_horizontal_offset(25);
    assert_eq!(state.horizontal_offset(), 25);
    state.reset_horizontal_offset();
    assert_eq!(state.horizontal_offset(), 0);
}

#[test]
fn test_scroll_right() {
    let tree = YamlTree::new(YamlNode::new(YamlValue::Object(IndexMap::new())));
    let mut state = EditorState::new_with_default_theme(tree);
    state.scroll_right(5);
    assert_eq!(state.horizontal_offset(), 5);
}

#[test]
fn test_scroll_left_clamps_to_zero() {
    let tree = YamlTree::new(YamlNode::new(YamlValue::Object(IndexMap::new())));
    let mut state = EditorState::new_with_default_theme(tree);
    state.scroll_right(3);
    state.scroll_left(10);
    assert_eq!(state.horizontal_offset(), 0);
}

#[test]
fn test_scroll_left() {
    let tree = YamlTree::new(YamlNode::new(YamlValue::Object(IndexMap::new())));
    let mut state = EditorState::new_with_default_theme(tree);
    state.scroll_right(10);
    state.scroll_left(3);
    assert_eq!(state.horizontal_offset(), 7);
}

#[test]
fn test_horizontal_offset_resets_on_move_down() {
    let mut obj = IndexMap::new();
    obj.insert(
        "a".to_string(),
        YamlNode::new(YamlValue::String(YamlString::Plain("hello".to_string()))),
    );
    obj.insert(
        "b".to_string(),
        YamlNode::new(YamlValue::String(YamlString::Plain("world".to_string()))),
    );
    let tree = YamlTree::new(YamlNode::new(YamlValue::Object(obj)));
    let mut state = EditorState::new_with_default_theme(tree);
    state.scroll_right(5);
    assert_eq!(state.horizontal_offset(), 5);
    state.move_cursor_down();
    assert_eq!(state.horizontal_offset(), 0);
}

#[test]
fn test_horizontal_offset_resets_on_move_up() {
    let mut obj = IndexMap::new();
    obj.insert(
        "a".to_string(),
        YamlNode::new(YamlValue::String(YamlString::Plain("hello".to_string()))),
    );
    obj.insert(
        "b".to_string(),
        YamlNode::new(YamlValue::String(YamlString::Plain("world".to_string()))),
    );
    let tree = YamlTree::new(YamlNode::new(YamlValue::Object(obj)));
    let mut state = EditorState::new_with_default_theme(tree);
    // Move down first so we can move up
    state.move_cursor_down();
    state.scroll_right(5);
    assert_eq!(state.horizontal_offset(), 5);
    state.move_cursor_up();
    assert_eq!(state.horizontal_offset(), 0);
}

#[test]
fn test_horizontal_offset_resets_on_jump_to_top() {
    let mut obj = IndexMap::new();
    obj.insert(
        "a".to_string(),
        YamlNode::new(YamlValue::String(YamlString::Plain("hello".to_string()))),
    );
    let tree = YamlTree::new(YamlNode::new(YamlValue::Object(obj)));
    let mut state = EditorState::new_with_default_theme(tree);
    state.scroll_right(5);
    state.jump_to_top();
    assert_eq!(state.horizontal_offset(), 0);
}

#[test]
fn test_horizontal_offset_resets_on_jump_to_bottom() {
    let mut obj = IndexMap::new();
    obj.insert(
        "a".to_string(),
        YamlNode::new(YamlValue::String(YamlString::Plain("hello".to_string()))),
    );
    let tree = YamlTree::new(YamlNode::new(YamlValue::Object(obj)));
    let mut state = EditorState::new_with_default_theme(tree);
    state.scroll_right(5);
    state.jump_to_bottom();
    assert_eq!(state.horizontal_offset(), 0);
}

#[test]
fn test_viewport_width_defaults_to_zero() {
    let tree = YamlTree::new(YamlNode::new(YamlValue::Object(IndexMap::new())));
    let state = EditorState::new_with_default_theme(tree);
    assert_eq!(state.viewport_width(), 0);
}

#[test]
fn test_set_viewport_width() {
    let tree = YamlTree::new(YamlNode::new(YamlValue::Object(IndexMap::new())));
    let mut state = EditorState::new_with_default_theme(tree);
    state.set_viewport_width(80);
    assert_eq!(state.viewport_width(), 80);
}

#[test]
fn test_cursor_line_display_width() {
    // Object with key "name" and value "Alice"
    let mut obj = IndexMap::new();
    obj.insert(
        "name".to_string(),
        YamlNode::new(YamlValue::String(YamlString::Plain("Alice".to_string()))),
    );
    let tree = YamlTree::new(YamlNode::new(YamlValue::Object(obj)));
    let mut state = EditorState::new_with_default_theme(tree);
    // Move to the "name" child
    state.move_cursor_down();
    let width = state.cursor_line_display_width();
    assert!(width > 0, "cursor line display width should be positive");
}

#[test]
fn test_scroll_cursor_to_left_edge() {
    // Create a nested object so children are at depth > 0
    let mut inner_obj = IndexMap::new();
    inner_obj.insert(
        "inner".to_string(),
        YamlNode::new(YamlValue::String(YamlString::Plain("value".to_string()))),
    );
    let mut obj = IndexMap::new();
    obj.insert(
        "outer".to_string(),
        YamlNode::new(YamlValue::Object(inner_obj)),
    );
    let tree = YamlTree::new(YamlNode::new(YamlValue::Object(obj)));
    let mut state = EditorState::new_with_default_theme(tree);
    // Cursor starts at [0] (outer), move down to [0,0] (inner) at depth 1
    state.move_cursor_down();
    state.scroll_cursor_to_left_edge();
    // The horizontal offset should be the indent of the cursor line
    assert_eq!(state.horizontal_offset(), 2); // depth 1 * 2 = 2
}

#[test]
fn test_scroll_cursor_to_right_edge() {
    let mut obj = IndexMap::new();
    obj.insert(
        "name".to_string(),
        YamlNode::new(YamlValue::String(YamlString::Plain("Alice".to_string()))),
    );
    let tree = YamlTree::new(YamlNode::new(YamlValue::Object(obj)));
    let mut state = EditorState::new_with_default_theme(tree);
    state.set_viewport_width(10);
    state.move_cursor_down();
    state.scroll_cursor_to_right_edge();
    // The offset should be set so the line ends at the right edge
    let width = state.cursor_line_display_width();
    if width > 10 {
        assert_eq!(state.horizontal_offset(), width - 10);
    } else {
        assert_eq!(state.horizontal_offset(), 0);
    }
}

#[test]
fn test_scroll_cursor_to_right_edge_narrow_content() {
    // When the content is narrower than the viewport, offset should be 0
    let mut obj = IndexMap::new();
    obj.insert(
        "a".to_string(),
        YamlNode::new(YamlValue::Number(YamlNumber::Integer(1))),
    );
    let tree = YamlTree::new(YamlNode::new(YamlValue::Object(obj)));
    let mut state = EditorState::new_with_default_theme(tree);
    state.set_viewport_width(200); // very wide viewport
    state.move_cursor_down();
    state.scroll_cursor_to_right_edge();
    assert_eq!(state.horizontal_offset(), 0);
}

#[test]
fn test_search_resets_horizontal_offset() {
    let mut obj = IndexMap::new();
    obj.insert(
        "a".to_string(),
        YamlNode::new(YamlValue::String(YamlString::Plain("hello".to_string()))),
    );
    let tree = YamlTree::new(YamlNode::new(YamlValue::Object(obj)));
    let mut state = EditorState::new_with_default_theme(tree);
    state.scroll_right(20);
    assert_eq!(state.horizontal_offset(), 20);
    // Navigating to top (simulating what happens after search) should reset
    state.jump_to_top();
    assert_eq!(state.horizontal_offset(), 0);
}
