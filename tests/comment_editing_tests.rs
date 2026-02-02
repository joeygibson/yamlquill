//! Tests for comment editing keybindings and operations.
//!
//! These tests verify that comments can be added, edited, and deleted using
//! vim-style keybindings with proper integration into undo/redo system.

use termion::event::{Event, Key};
use yamlquill::document::node::{CommentNode, CommentPosition, YamlNode, YamlValue};
use yamlquill::document::parser::parse_yaml_auto;
use yamlquill::document::tree::YamlTree;
use yamlquill::editor::mode::EditorMode;
use yamlquill::editor::state::EditorState;
use yamlquill::input::keys::{map_key_event, InputEvent};

/// Test 1: 'c' key maps to AddComment event in Normal mode
#[test]
fn test_c_key_maps_to_add_comment() {
    let event = Event::Key(Key::Char('c'));
    let input_event = map_key_event(event, &EditorMode::Normal);
    assert_eq!(input_event, InputEvent::AddComment);
}

/// Test 2: 'c' key on value node prompts for comment position
#[test]
fn test_add_comment_on_value_node() {
    let yaml = r#"
name: Alice
age: 30
"#;

    let node = parse_yaml_auto(yaml).unwrap();
    let tree = YamlTree::new(node);
    let mut state = EditorState::new(tree, "default".to_string());

    // Navigate down to a value
    state.move_cursor_down();

    // Simulate pressing 'c' key - should trigger position prompt
    // Note: This will be tested through integration once implemented
    // For now, just verify the key maps correctly
    let event = Event::Key(Key::Char('c'));
    let input_event = map_key_event(event, state.mode());
    assert_eq!(input_event, InputEvent::AddComment);
}

/// Test 3: 'e' key on comment node enters edit mode
#[test]
fn test_edit_comment_node() {
    let yaml = r#"
# This is a comment
name: Alice
"#;

    let node = parse_yaml_auto(yaml).unwrap();
    let tree = YamlTree::new(node);
    let mut state = EditorState::new(tree, "default".to_string());

    // Find and navigate to the comment node
    // The comment should be the first node in the tree
    state.move_cursor_down();

    // Pressing 'e' should enter insert mode for editing the comment
    let event = Event::Key(Key::Char('e'));
    let input_event = map_key_event(event, state.mode());
    assert_eq!(input_event, InputEvent::EnterInsertMode);
}

/// Test 4: 'dd' on comment node deletes the comment
#[test]
fn test_delete_comment_node() {
    let yaml = r#"
# This is a comment
name: Alice
"#;

    let node = parse_yaml_auto(yaml).unwrap();
    let tree = YamlTree::new(node);
    let mut state = EditorState::new(tree, "default".to_string());

    // Navigate to the comment node
    state.move_cursor_down();

    // Simulate 'dd' - first 'd' maps to Delete
    let event = Event::Key(Key::Char('d'));
    let input_event = map_key_event(event, state.mode());
    assert_eq!(input_event, InputEvent::Delete);

    // Note: Full 'dd' handling is done in the input handler
    // This test just verifies the key mapping is correct
}

/// Test 5: Add comment Above position
#[test]
fn test_add_comment_above() {
    let yaml = r#"
name: Alice
age: 30
"#;

    let node = parse_yaml_auto(yaml).unwrap();
    let _tree = YamlTree::new(node);

    // Simulate adding a comment "Above" a value
    let comment = CommentNode::new(
        "This is a comment above".to_string(),
        CommentPosition::Above,
    );

    // Verify comment node can be created
    let comment_value = YamlValue::Comment(comment);
    let comment_node = YamlNode::new(comment_value);

    match comment_node.value() {
        YamlValue::Comment(c) => {
            assert_eq!(c.content(), "This is a comment above");
            assert_eq!(c.position(), &CommentPosition::Above);
        }
        _ => panic!("Expected comment value"),
    }
}

/// Test 6: Add comment Line (inline) position
#[test]
fn test_add_comment_line() {
    let yaml = r#"
name: Alice
"#;

    let node = parse_yaml_auto(yaml).unwrap();
    let _tree = YamlTree::new(node);

    // Simulate adding an inline comment
    let comment = CommentNode::new("inline comment".to_string(), CommentPosition::Line);

    let comment_value = YamlValue::Comment(comment);
    let comment_node = YamlNode::new(comment_value);

    match comment_node.value() {
        YamlValue::Comment(c) => {
            assert_eq!(c.content(), "inline comment");
            assert_eq!(c.position(), &CommentPosition::Line);
        }
        _ => panic!("Expected comment value"),
    }
}

/// Test 7: Add comment Below position
#[test]
fn test_add_comment_below() {
    let yaml = r#"
people:
  - name: Alice
  - name: Bob
"#;

    let node = parse_yaml_auto(yaml).unwrap();
    let _tree = YamlTree::new(node);

    // Simulate adding a comment below a block
    let comment = CommentNode::new(
        "This is after the block".to_string(),
        CommentPosition::Below,
    );

    let comment_value = YamlValue::Comment(comment);
    let comment_node = YamlNode::new(comment_value);

    match comment_node.value() {
        YamlValue::Comment(c) => {
            assert_eq!(c.content(), "This is after the block");
            assert_eq!(c.position(), &CommentPosition::Below);
        }
        _ => panic!("Expected comment value"),
    }
}

/// Test 8: Undo after adding comment
#[test]
fn test_undo_add_comment() {
    let yaml = r#"
name: Alice
"#;

    let node = parse_yaml_auto(yaml).unwrap();
    let tree = YamlTree::new(node);
    let state = EditorState::new(tree, "default".to_string());

    // TODO: Once add_comment is implemented, test:
    // 1. Add a comment
    // 2. Verify comment was added (count increases)
    // 3. Call undo
    // 4. Verify comment is removed (count returns to initial)

    // For now, just verify undo key mapping works
    let event = Event::Key(Key::Char('u'));
    let input_event = map_key_event(event, state.mode());
    assert_eq!(input_event, InputEvent::Undo);
}

/// Test 9: Redo after undoing comment addition
#[test]
fn test_redo_add_comment() {
    let yaml = r#"
name: Alice
"#;

    let node = parse_yaml_auto(yaml).unwrap();
    let tree = YamlTree::new(node);
    let state = EditorState::new(tree, "default".to_string());

    // TODO: Once add_comment is implemented, test:
    // 1. Add a comment
    // 2. Undo
    // 3. Redo
    // 4. Verify comment is back

    // For now, verify redo key mapping
    let event = Event::Key(Key::Ctrl('r'));
    let input_event = map_key_event(event, state.mode());
    assert_eq!(input_event, InputEvent::Redo);
}

/// Test 10: Edit existing comment content
#[test]
fn test_edit_existing_comment_content() {
    let yaml = r#"
# Original comment
name: Alice
"#;

    let node = parse_yaml_auto(yaml).unwrap();
    let tree = YamlTree::new(node);
    let state = EditorState::new(tree, "default".to_string());

    // TODO: Once edit_comment is implemented, test:
    // 1. Navigate to comment
    // 2. Press 'e' to edit
    // 3. Modify text
    // 4. Press Enter to save
    // 5. Verify comment content changed

    // For now, verify comment exists in tree
    let has_comment = tree_contains_comment(&state);
    assert!(has_comment, "Tree should contain comment");
}

/// Test 11: Integration test - Add comment to array element
#[test]
fn test_integration_add_comment_to_array() {
    let yaml = r#"
items:
  - apple
  - banana
  - cherry
"#;

    let node = parse_yaml_auto(yaml).unwrap();
    let tree = YamlTree::new(node);
    let mut state = EditorState::new(tree, "default".to_string());

    // Navigate to first array element (apple)
    // Tree structure: root (Object) -> items (Array) -> [0] (String)
    state.move_cursor_down(); // Move to "items" key
    state.move_cursor_down(); // Move into array
    state.move_cursor_down(); // Move to first element "apple"

    // Get initial node count
    let initial_count = count_tree_nodes(&state);

    // Start add comment operation via the method (simulating 'c' key press)
    state.start_add_comment_operation();

    // Verify we're in insert mode
    assert_eq!(state.mode(), &EditorMode::Insert);

    // Simulate typing a comment
    state.push_to_edit_buffer('T');
    state.push_to_edit_buffer('e');
    state.push_to_edit_buffer('s');
    state.push_to_edit_buffer('t');

    // Commit the comment
    let result = state.commit_add_comment();
    assert!(result.is_ok(), "Failed to add comment: {:?}", result.err());

    // Verify comment was added (node count increased)
    let new_count = count_tree_nodes(&state);
    assert_eq!(
        new_count,
        initial_count + 1,
        "Comment should add 1 node to tree"
    );

    // Verify the comment exists in the tree
    assert!(
        tree_contains_comment(&state),
        "Tree should contain comment after adding"
    );
}

/// Test 12: Integration test - Edit existing comment
#[test]
fn test_integration_edit_existing_comment() {
    let yaml = r#"
items:
  # Original comment
  - apple
  - banana
"#;

    let node = parse_yaml_auto(yaml).unwrap();
    let tree = YamlTree::new(node);
    let mut state = EditorState::new(tree, "default".to_string());

    // Find the comment node
    // Navigate down to find it
    state.move_cursor_down(); // items key
    state.move_cursor_down(); // items array
    state.move_cursor_down(); // Should be comment or first array element

    // Keep navigating until we find the comment
    let mut found_comment = false;
    for _ in 0..5 {
        // Try up to 5 positions
        if let Some(current_node) = state.tree().get_node(state.cursor().path()) {
            if matches!(current_node.value(), YamlValue::Comment(_)) {
                found_comment = true;
                break;
            }
        }
        state.move_cursor_down();
    }

    assert!(found_comment, "Should find comment in tree");

    // Start editing the comment (this populates the buffer but doesn't change mode)
    state.start_editing();

    // Verify buffer contains original text (start_editing sets up buffer)
    assert_eq!(
        state.edit_buffer(),
        Some("Original comment"),
        "Edit buffer should contain comment text"
    );

    // Now enter Insert mode (simulating what the key handler does)
    state.set_mode(EditorMode::Insert);
    assert_eq!(state.mode(), &EditorMode::Insert);

    // Clear buffer and type new text
    state.clear_edit_buffer();
    state.push_to_edit_buffer('E');
    state.push_to_edit_buffer('d');
    state.push_to_edit_buffer('i');
    state.push_to_edit_buffer('t');
    state.push_to_edit_buffer('e');
    state.push_to_edit_buffer('d');

    // Commit the edit
    let result = state.commit_editing();
    assert!(result.is_ok(), "Failed to edit comment: {:?}", result.err());

    // Verify comment content changed
    if let Some(current_node) = state.tree().get_node(state.cursor().path()) {
        match current_node.value() {
            YamlValue::Comment(c) => {
                assert_eq!(c.content(), "Edited", "Comment content should be updated");
            }
            _ => panic!("Expected comment node after edit"),
        }
    } else {
        panic!("Current node not found after edit");
    }
}

// Helper functions

/// Count total nodes in the editor state tree
fn count_tree_nodes(state: &EditorState) -> usize {
    count_yaml_nodes(state.tree().root())
}

/// Recursively count nodes in a YamlNode tree
fn count_yaml_nodes(node: &YamlNode) -> usize {
    match node.value() {
        YamlValue::Object(map) => 1 + map.values().map(|v| count_yaml_nodes(v)).sum::<usize>(),
        YamlValue::Array(arr) => 1 + arr.iter().map(|v| count_yaml_nodes(v)).sum::<usize>(),
        YamlValue::MultiDoc(docs) => 1 + docs.iter().map(|v| count_yaml_nodes(v)).sum::<usize>(),
        _ => 1,
    }
}

/// Check if tree contains any comment nodes
fn tree_contains_comment(state: &EditorState) -> bool {
    contains_comment_recursive(state.tree().root())
}

/// Recursively check for comment nodes
fn contains_comment_recursive(node: &YamlNode) -> bool {
    match node.value() {
        YamlValue::Comment(_) => true,
        YamlValue::Object(map) => map.values().any(|v| contains_comment_recursive(v)),
        YamlValue::Array(arr) => arr.iter().any(|v| contains_comment_recursive(v)),
        YamlValue::MultiDoc(docs) => docs.iter().any(|v| contains_comment_recursive(v)),
        _ => false,
    }
}
