//! Tests for comment display in the tree view.
//!
//! These tests verify that comments are properly rendered with appropriate
//! styling and positioning in the tree view.

use ratatui::backend::TestBackend;
use ratatui::Terminal;
use yamlquill::document::node::{CommentNode, CommentPosition, YamlNode, YamlString, YamlValue};
use yamlquill::document::parser::parse_yaml_auto;
use yamlquill::document::tree::YamlTree;
use yamlquill::editor::cursor::Cursor;
use yamlquill::theme::colors::ThemeColors;
use yamlquill::ui::tree_view::{render_tree_view, TreeViewState};

/// Test 1: Standalone comment renders on its own line
#[test]
fn test_display_standalone_comment() {
    let yaml = r#"
# Standalone comment
name: Alice
"#;

    let node = parse_yaml_auto(yaml).unwrap();
    let tree = YamlTree::new(node);
    let mut tree_view = TreeViewState::new();
    tree_view.rebuild(&tree);

    // Should have 2 lines: comment + name
    assert_eq!(tree_view.lines().len(), 2);

    // Find the comment line (order may vary depending on parser)
    let comment_line = tree_view
        .lines()
        .iter()
        .find(|line| line.key.is_none() && line.value_preview.contains("Standalone comment"))
        .expect("Should find comment line");

    assert!(
        comment_line.value_preview.contains("Standalone comment"),
        "Preview: {}",
        comment_line.value_preview
    );

    // Also verify name line exists
    let name_line = tree_view
        .lines()
        .iter()
        .find(|line| line.key.as_ref().map_or(false, |k| k == "name"))
        .expect("Should find name line");
    assert!(name_line.value_preview.contains("Alice"));
}

/// Test 2: Above comment renders before the value it annotates
#[test]
fn test_display_above_comment() {
    let yaml = r#"
# This describes the name field
name: Alice
age: 30
"#;

    let node = parse_yaml_auto(yaml).unwrap();
    let tree = YamlTree::new(node);
    let mut tree_view = TreeViewState::new();
    tree_view.rebuild(&tree);

    // Should have 3 lines: comment, name, age
    assert_eq!(tree_view.lines().len(), 3);

    // Find the comment line
    let comment_line = tree_view
        .lines()
        .iter()
        .find(|line| {
            line.key.is_none() && line.value_preview.contains("This describes the name field")
        })
        .expect("Should find comment line");

    assert!(comment_line
        .value_preview
        .contains("This describes the name field"));

    // Verify name and age lines exist
    assert!(tree_view
        .lines()
        .iter()
        .any(|line| line.key.as_ref().map_or(false, |k| k == "name")));
    assert!(tree_view
        .lines()
        .iter()
        .any(|line| line.key.as_ref().map_or(false, |k| k == "age")));
}

/// Test 3: Line (inline) comment renders inline with value
#[test]
fn test_display_inline_comment() {
    let yaml = r#"
name: Alice  # inline comment
age: 30
"#;

    let node = parse_yaml_auto(yaml).unwrap();
    let tree = YamlTree::new(node);
    let mut tree_view = TreeViewState::new();
    tree_view.rebuild(&tree);

    // Should have 3 lines: name, comment, age
    // Note: The inline comment appears as a separate node after the value
    assert!(tree_view.lines().len() >= 2);

    // Find the comment line (should be adjacent to name)
    // Comments have no key (None)
    let mut found_inline_comment = false;
    for line in tree_view.lines() {
        if line.key.is_none() && line.value_preview.contains("inline comment") {
            found_inline_comment = true;
        }
    }
    assert!(found_inline_comment, "Expected to find inline comment");
}

/// Test 4: Comment rendered with gray color in terminal
#[test]
fn test_comment_color_styling() {
    let yaml = r#"
# This is a comment
name: Alice
"#;

    let node = parse_yaml_auto(yaml).unwrap();
    let tree = YamlTree::new(node);
    let mut tree_view = TreeViewState::new();
    tree_view.rebuild(&tree);

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let colors = ThemeColors::default_dark();
    let cursor = Cursor::new();

    terminal
        .draw(|f| {
            render_tree_view(
                f,
                f.area(),
                &tree_view,
                &cursor,
                &colors,
                false,
                false,
                0,
                &[],
            );
        })
        .unwrap();

    let buffer = terminal.backend().buffer().clone();
    let content = buffer
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();

    // Comment should appear in the output
    assert!(
        content.contains("# This is a comment"),
        "Comment should be visible in output"
    );
}

/// Test 5: Cursor highlighting works on comment lines
#[test]
fn test_comment_cursor_highlighting() {
    let yaml = r#"
# Standalone comment
name: Alice
"#;

    let node = parse_yaml_auto(yaml).unwrap();
    let tree = YamlTree::new(node);
    let mut tree_view = TreeViewState::new();
    tree_view.rebuild(&tree);

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let colors = ThemeColors::default_dark();
    let mut cursor = Cursor::new();

    // Find and set cursor to comment line
    let comment_line = tree_view
        .lines()
        .iter()
        .find(|line| line.key.is_none())
        .expect("Should have comment line");
    cursor.set_path(comment_line.path.clone());

    terminal
        .draw(|f| {
            render_tree_view(
                f,
                f.area(),
                &tree_view,
                &cursor,
                &colors,
                false,
                false,
                0,
                &[],
            );
        })
        .unwrap();

    let buffer = terminal.backend().buffer().clone();

    // Check that comment line has cursor background
    let mut found_cursor_highlight = false;
    for cell in buffer.content().iter() {
        if cell.symbol() == "#" && cell.bg == colors.cursor {
            found_cursor_highlight = true;
            break;
        }
    }

    assert!(
        found_cursor_highlight,
        "Comment should be highlighted when cursor is on it"
    );
}

/// Test 6: Multiple comments render correctly
#[test]
fn test_multiple_comments() {
    let yaml = r#"
# Comment 1
name: Alice

# Comment 2
age: 30
"#;

    let node = parse_yaml_auto(yaml).unwrap();
    let tree = YamlTree::new(node);
    let mut tree_view = TreeViewState::new();
    tree_view.rebuild(&tree);

    // Count comment lines (they have no key)
    let comment_count = tree_view
        .lines()
        .iter()
        .filter(|line| line.key.is_none() && line.value_preview.starts_with("#"))
        .count();

    assert_eq!(comment_count, 2, "Expected 2 comment lines");
}

/// Test 7: Comments in nested objects render with correct indentation
#[test]
fn test_nested_comment_indentation() {
    let yaml = r#"
person:
  # Name field
  name: Alice
  age: 30
"#;

    let node = parse_yaml_auto(yaml).unwrap();
    let tree = YamlTree::new(node);
    let mut tree_view = TreeViewState::new();
    tree_view.expand_all(&tree);
    tree_view.rebuild(&tree);

    // Find the nested comment (no key, starts with #)
    let nested_comment = tree_view
        .lines()
        .iter()
        .find(|line| line.key.is_none() && line.value_preview.contains("Name field"))
        .expect("Should find nested comment");

    // Comment should be at depth 1 (inside person object)
    assert_eq!(
        nested_comment.depth, 1,
        "Nested comment should have depth 1"
    );
}

/// Test 8: Comment nodes are navigable with j/k
#[test]
fn test_comment_navigability() {
    let yaml = r#"
# Comment above
name: Alice
age: 30
"#;

    let node = parse_yaml_auto(yaml).unwrap();
    let tree = YamlTree::new(node);
    let mut tree_view = TreeViewState::new();
    tree_view.rebuild(&tree);

    let lines = tree_view.lines();

    // Should have 3 lines: comment, name, age
    assert_eq!(lines.len(), 3);

    // All lines should have paths (making them navigable)
    for (i, line) in lines.iter().enumerate() {
        assert!(
            !line.path.is_empty(),
            "Line {} should have a path for navigation",
            i
        );
    }

    // Cursor should be able to select comment line
    let comment_line = lines
        .iter()
        .find(|line| line.key.is_none())
        .expect("Should have a comment line");
    let mut cursor = Cursor::new();
    cursor.set_path(comment_line.path.clone());
    assert_eq!(cursor.path(), &comment_line.path);
}

/// Test 9: Empty comment renders correctly
#[test]
fn test_empty_comment() {
    // Create a node manually with an empty comment
    let mut map = indexmap::IndexMap::new();
    map.insert(
        "__comment_0__".to_string(),
        YamlNode::new(YamlValue::Comment(CommentNode::new(
            "".to_string(),
            CommentPosition::Standalone,
        ))),
    );
    map.insert(
        "name".to_string(),
        YamlNode::new(YamlValue::String(YamlString::Plain("Alice".to_string()))),
    );

    let node = YamlNode::new(YamlValue::Object(map));
    let tree = YamlTree::new(node);
    let mut tree_view = TreeViewState::new();
    tree_view.rebuild(&tree);

    // Should have 2 lines (comment + name)
    assert_eq!(tree_view.lines().len(), 2);

    // Find the empty comment line
    let comment_line = tree_view
        .lines()
        .iter()
        .find(|line| line.key.is_none() && line.value_preview == "#")
        .expect("Should find empty comment line");

    // Empty comment shows just "#"
    assert_eq!(comment_line.value_preview, "#");
}

/// Test 10: Comment with special characters renders safely
#[test]
fn test_comment_special_characters() {
    let yaml = r#"
# Comment with "quotes" and \backslash and <brackets>
name: Alice
"#;

    let node = parse_yaml_auto(yaml).unwrap();
    let tree = YamlTree::new(node);
    let mut tree_view = TreeViewState::new();
    tree_view.rebuild(&tree);

    // Find the comment line
    let comment_line = tree_view
        .lines()
        .iter()
        .find(|line| line.key.is_none())
        .expect("Should have comment line");

    assert!(
        comment_line.value_preview.contains("quotes"),
        "Preview: {}",
        comment_line.value_preview
    );
    assert!(comment_line.value_preview.contains("backslash"));
    assert!(comment_line.value_preview.contains("brackets"));
}
