# Comment Support Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add full comment editing support to YAMLQuill with comments as first-class navigable tree nodes.

**Architecture:** Comments become YamlValue::Comment variants with position metadata. TreeBuilder extracts comments during parsing via Scanner tokens. Display renders comments inline (Line position) or as separate lines (Above/Below/Standalone). Editing uses vim keybindings (c=add, e=edit, dd=delete). Save injects comments into serde_yaml output.

**Tech Stack:** yaml-rust2 Scanner for tokenization, indexmap for ordered storage, ratatui for TUI rendering

---

## Task 1: Add Comment Data Types

**Files:**
- Modify: `src/document/node.rs:116-133` (YamlValue enum)
- Test: `tests/comment_data_model_tests.rs` (create new)

**Step 1: Write failing tests for CommentNode types**

```rust
// tests/comment_data_model_tests.rs
use yamlquill::document::node::{CommentNode, CommentPosition, YamlValue, YamlNode};

#[test]
fn test_comment_node_creation() {
    let comment = CommentNode {
        content: "Test comment".to_string(),
        position: CommentPosition::Above,
    };
    assert_eq!(comment.content, "Test comment");
    assert!(matches!(comment.position, CommentPosition::Above));
}

#[test]
fn test_comment_value_variant() {
    let comment_node = CommentNode {
        content: "Line comment".to_string(),
        position: CommentPosition::Line,
    };
    let value = YamlValue::Comment(comment_node);
    assert!(matches!(value, YamlValue::Comment(_)));
}

#[test]
fn test_yaml_node_is_comment() {
    let comment = YamlNode::new(YamlValue::Comment(CommentNode {
        content: "Test".to_string(),
        position: CommentPosition::Standalone,
    }));
    assert!(comment.is_comment());

    let string = YamlNode::new(YamlValue::String(yamlquill::document::node::YamlString::Plain("text".to_string())));
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
```

**Step 2: Run tests to verify they fail**

Run: `cd ~/.config/superpowers/worktrees/yamlquill/comment-support && cargo test comment_data_model`

Expected: FAIL with "unresolved import" errors for CommentNode and CommentPosition

**Step 3: Add CommentPosition enum to node.rs**

Add after YamlNumber implementation (around line 108):

```rust
/// Position of a comment relative to YAML nodes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommentPosition {
    /// Comment line(s) before a value
    Above,
    /// Inline comment after a value on the same line
    Line,
    /// Comment after children/end of block
    Below,
    /// Comment between blank lines (not associated with specific value)
    Standalone,
}
```

**Step 4: Add CommentNode struct to node.rs**

Add after CommentPosition:

```rust
/// A YAML comment node
#[derive(Debug, Clone, PartialEq)]
pub struct CommentNode {
    /// Comment text without the '#' prefix
    pub content: String,
    /// Position of this comment relative to other nodes
    pub position: CommentPosition,
}

impl CommentNode {
    /// Create a new comment node
    pub fn new(content: String, position: CommentPosition) -> Self {
        Self { content, position }
    }

    /// Get the comment content
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Get the comment position
    pub fn position(&self) -> &CommentPosition {
        &self.position
    }
}
```

**Step 5: Add Comment variant to YamlValue enum**

Modify YamlValue enum (line 116):

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum YamlValue {
    /// A YAML object containing key-value pairs
    Object(IndexMap<String, YamlNode>),
    /// A YAML array containing ordered values
    Array(Vec<YamlNode>),
    /// A YAML string with style information
    String(YamlString),
    /// A YAML number (integer or float)
    Number(YamlNumber),
    /// A YAML boolean
    Boolean(bool),
    /// A YAML null value
    Null,
    /// A YAML alias reference
    Alias(String),
    /// A multi-document YAML file (each document is a YamlNode)
    MultiDoc(Vec<YamlNode>),
    /// A YAML comment
    Comment(CommentNode),  // NEW
}
```

**Step 6: Add is_comment() helper to YamlValue**

Add after is_multidoc() method (find similar pattern around line 220):

```rust
/// Returns true if this value is a comment.
pub fn is_comment(&self) -> bool {
    matches!(self, YamlValue::Comment(_))
}
```

**Step 7: Add is_comment() helper to YamlNode**

Add after is_alias() method in YamlNode impl (find similar pattern around line 350):

```rust
/// Returns true if this node is a comment.
pub fn is_comment(&self) -> bool {
    self.value.is_comment()
}
```

**Step 8: Export new types from node module**

Ensure CommentNode and CommentPosition are public exports at top of file

**Step 9: Run tests to verify they pass**

Run: `cd ~/.config/superpowers/worktrees/yamlquill/comment-support && cargo test comment_data_model`

Expected: PASS (4 tests)

**Step 10: Run full test suite**

Run: `cd ~/.config/superpowers/worktrees/yamlquill/comment-support && cargo test`

Expected: All existing tests still pass (390 tests)

**Step 11: Commit**

```bash
cd ~/.config/superpowers/worktrees/yamlquill/comment-support
git add src/document/node.rs tests/comment_data_model_tests.rs
git commit -m "feat: add Comment data types to YamlValue

Add CommentNode and CommentPosition types to represent YAML comments
as first-class tree nodes. Comments have content (without '#') and
position (Above/Line/Below/Standalone).

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 2: Comment Extraction During Parsing

**Files:**
- Modify: `src/document/parser.rs` (TreeBuilder struct and parse functions)
- Test: `tests/comment_extraction_tests.rs` (create new)

**Step 1: Write failing test for basic comment extraction**

```rust
// tests/comment_extraction_tests.rs
use yamlquill::document::parser::parse_yaml;
use yamlquill::document::node::{YamlValue, CommentPosition};

#[test]
fn test_extract_above_comment() {
    let yaml = r#"
# This is a comment
key: value
"#;

    let result = parse_yaml(yaml).unwrap();
    let root = result.root();

    if let YamlValue::Object(map) = root.value() {
        // First child should be a comment
        let first = map.values().next().unwrap();
        assert!(first.is_comment());

        if let YamlValue::Comment(comment) = first.value() {
            assert_eq!(comment.content(), "This is a comment");
            assert!(matches!(comment.position(), CommentPosition::Above));
        }
    } else {
        panic!("Expected object root");
    }
}

#[test]
fn test_extract_line_comment() {
    let yaml = "key: value  # inline comment\n";

    let result = parse_yaml(yaml).unwrap();
    let root = result.root();

    if let YamlValue::Object(map) = root.value() {
        // Should have 2 entries: key-value and comment
        assert_eq!(map.len(), 2);

        let second = map.values().nth(1).unwrap();
        assert!(second.is_comment());

        if let YamlValue::Comment(comment) = second.value() {
            assert_eq!(comment.content().trim(), "inline comment");
            assert!(matches!(comment.position(), CommentPosition::Line));
        }
    } else {
        panic!("Expected object root");
    }
}

#[test]
fn test_extract_standalone_comment() {
    let yaml = r#"
key1: value1

# Standalone comment

key2: value2
"#;

    let result = parse_yaml(yaml).unwrap();
    let root = result.root();

    if let YamlValue::Object(map) = root.value() {
        // Should have: key1, comment, key2
        let values: Vec<_> = map.values().collect();

        let comment = values.iter()
            .find(|v| v.is_comment())
            .expect("Should have a comment node");

        if let YamlValue::Comment(c) = comment.value() {
            assert_eq!(c.content().trim(), "Standalone comment");
            assert!(matches!(c.position(), CommentPosition::Standalone));
        }
    }
}

#[test]
fn test_extract_array_comments() {
    let yaml = r#"
items:
  # Before first item
  - item1  # Line comment
  - item2
"#;

    let result = parse_yaml(yaml).unwrap();
    let root = result.root();

    if let YamlValue::Object(map) = root.value() {
        let items_node = map.get("items").unwrap();
        if let YamlValue::Array(arr) = items_node.value() {
            // Array should contain: comment, item1, comment, item2
            assert!(arr.len() >= 3);

            // First should be Above comment
            assert!(arr[0].is_comment());
            if let YamlValue::Comment(c) = arr[0].value() {
                assert!(matches!(c.position(), CommentPosition::Above));
            }

            // Third should be Line comment
            assert!(arr[2].is_comment());
            if let YamlValue::Comment(c) = arr[2].value() {
                assert!(matches!(c.position(), CommentPosition::Line));
            }
        }
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cd ~/.config/superpowers/worktrees/yamlquill/comment-support && cargo test comment_extraction`

Expected: FAIL - comments not extracted, assertions fail

**Step 3: Study Scanner API**

Read yaml-rust2 Scanner documentation to understand Token::Comment structure.

Add debug code to see what tokens look like:

```rust
// Temporary debug in parser.rs
use yaml_rust2::scanner::{Scanner, Token};

fn debug_tokens(yaml: &str) {
    let scanner = Scanner::new(yaml.chars());
    for token in scanner {
        if matches!(token.1, Token::Comment(_)) {
            eprintln!("Found comment token: {:?}", token);
        }
    }
}
```

**Step 4: Add comment tracking fields to TreeBuilder**

Modify TreeBuilder struct in `src/document/parser.rs`:

```rust
struct TreeBuilder {
    stack: Vec<BuilderFrame>,
    anchor_registry: AnchorRegistry,
    pending_comments: Vec<PendingComment>,  // NEW
    last_event_line: usize,  // NEW
}

#[derive(Debug)]
struct PendingComment {
    content: String,
    line: usize,
    col: usize,
}
```

**Step 5: Initialize new fields in TreeBuilder::new()**

```rust
impl TreeBuilder {
    fn new() -> Self {
        Self {
            stack: vec![BuilderFrame {
                key: None,
                value: None,
                children: vec![],
            }],
            anchor_registry: AnchorRegistry::new(),
            pending_comments: Vec::new(),
            last_event_line: 0,
        }
    }
}
```

**Step 6: Add helper to flush pending comments as Above nodes**

```rust
impl TreeBuilder {
    fn flush_pending_comments_as_above(&mut self) {
        for pending in self.pending_comments.drain(..) {
            let comment_node = YamlNode::new(YamlValue::Comment(CommentNode::new(
                pending.content.trim_start_matches('#').trim().to_string(),
                CommentPosition::Above,
            )));

            if let Some(frame) = self.stack.last_mut() {
                frame.children.push(comment_node);
            }
        }
    }
}
```

**Step 7: Modify parse_yaml to use Scanner for comments**

This is complex - need to integrate Scanner token stream with EventReceiver events.

Strategy: First pass with EventReceiver (current), second pass with Scanner to inject comments.

Add second-pass function:

```rust
fn inject_comments(root: YamlNode, yaml: &str) -> YamlNode {
    use yaml_rust2::scanner::{Scanner, Token, TScalarStyle, TokenType};

    let scanner = Scanner::new(yaml.chars());
    let mut comments = Vec::new();

    // Collect all comments with line numbers
    for (pos, token) in scanner {
        if let Token::Comment(content) = token {
            comments.push((pos.line, content));
        }
    }

    // For now, just return root unchanged
    // TODO: Actually inject comments in correct positions
    root
}
```

**Step 8: Call inject_comments from parse_yaml**

Modify parse_yaml to call inject_comments after tree building:

```rust
pub fn parse_yaml(yaml: &str) -> Result<YamlTree> {
    // ... existing EventReceiver code ...

    let root = builder.build()?;
    let root_with_comments = inject_comments(root, yaml);

    Ok(YamlTree {
        root: root_with_comments,
        // ... other fields ...
    })
}
```

**Step 9: Implement comment injection logic**

This is the hard part. Need to:
1. Walk tree and track line numbers of nodes
2. Match comments to nodes based on line proximity
3. Insert comment nodes in correct positions

For now, implement simple version: comments before a line go as Above comments.

```rust
fn inject_comments(mut root: YamlNode, yaml: &str) -> YamlNode {
    use yaml_rust2::scanner::{Scanner, Token};

    let scanner = Scanner::new(yaml.chars());
    let mut comment_map: std::collections::HashMap<usize, Vec<String>> = std::collections::HashMap::new();

    // Collect comments by line number
    for (pos, token) in scanner {
        if let Token::Comment(content) = token {
            let cleaned = content.trim_start_matches('#').trim().to_string();
            comment_map.entry(pos.line).or_insert_with(Vec::new).push(cleaned);
        }
    }

    // For MVP: just inject comments as Above nodes in objects
    inject_comments_into_value(&mut root, &comment_map);

    root
}

fn inject_comments_into_value(node: &mut YamlNode, comment_map: &std::collections::HashMap<usize, Vec<String>>) {
    if let YamlValue::Object(map) = node.value_mut() {
        // Create new map with comments injected
        let mut new_map = indexmap::IndexMap::new();

        for (key, value) in map.iter_mut() {
            // Check if there are comments for this key
            // (This is simplified - real impl needs line tracking)

            new_map.insert(key.clone(), value.clone());

            // Recursively process children
            inject_comments_into_value(value, comment_map);
        }

        *map = new_map;
    } else if let YamlValue::Array(arr) = node.value_mut() {
        for child in arr.iter_mut() {
            inject_comments_into_value(child, comment_map);
        }
    }
}
```

**Step 10: Run tests**

Run: `cd ~/.config/superpowers/worktrees/yamlquill/comment-support && cargo test comment_extraction`

Expected: Some tests may pass, others still fail. This is OK - comment extraction is complex.

**Step 11: Iterate on comment injection logic**

Work through test failures one by one. Key challenges:
- Tracking line numbers of nodes (need to preserve from EventReceiver)
- Distinguishing Above vs Line vs Standalone comments
- Special keys for comments in IndexMap (`__comment_0__`, etc.)

This step may require multiple iterations. For each failing test:
1. Debug why comment not found/positioned correctly
2. Fix logic
3. Re-run tests

**Step 12: Run full test suite**

Run: `cd ~/.config/superpowers/worktrees/yamlquill/comment-support && cargo test`

Expected: All tests pass (390 + new comment tests)

**Step 13: Commit**

```bash
cd ~/.config/superpowers/worktrees/yamlquill/comment-support
cargo fmt
cargo clippy -- -D warnings
cargo test
git add src/document/parser.rs tests/comment_extraction_tests.rs
git commit -m "feat: extract comments during YAML parsing

Use Scanner to collect comments and inject as Comment nodes in tree.
Supports Above, Line, Below, and Standalone comment positions.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 3: Display Comments in Tree View

**Files:**
- Modify: `src/ui/tree_view.rs` (render_node function)
- Test: `tests/comment_display_tests.rs` (create new)

**Step 1: Write failing test for comment display**

```rust
// tests/comment_display_tests.rs
use yamlquill::document::node::{YamlNode, YamlValue, CommentNode, CommentPosition};
use yamlquill::ui::tree_view::{render_tree_view, TreeViewState};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

#[test]
fn test_display_above_comment() {
    let comment = YamlNode::new(YamlValue::Comment(CommentNode::new(
        "Above comment".to_string(),
        CommentPosition::Above,
    )));

    // TODO: Setup tree and render, verify comment line appears with "# Above comment"
}

#[test]
fn test_display_line_comment_inline() {
    // TODO: Test that Line positioned comments appear on same line as value
}

#[test]
fn test_comment_cursor_navigation() {
    // TODO: Test that comments are navigable with j/k
}
```

**Step 2: Run tests to verify they fail**

Run: `cd ~/.config/superpowers/worktrees/yamlquill/comment-support && cargo test comment_display`

Expected: FAIL or compile errors

**Step 3: Add comment rendering to render_node**

Modify `render_node` in `src/ui/tree_view.rs`:

Find the match statement on YamlValue variants. Add Comment branch:

```rust
YamlValue::Comment(comment_node) => {
    // Render comment based on position
    match comment_node.position() {
        CommentPosition::Line => {
            // Line comments should be rendered inline with previous node
            // This requires special handling in parent context
            // For now, render as separate line
            let comment_text = format!("# {}", comment_node.content());
            // Apply comment styling
            lines.push(Line::from(Span::styled(
                comment_text,
                colors.comment_style(),
            )));
        }
        CommentPosition::Above | CommentPosition::Below | CommentPosition::Standalone => {
            // Render as separate line
            let comment_text = format!("# {}", comment_node.content());
            lines.push(Line::from(Span::styled(
                comment_text,
                colors.comment_style(),
            )));
        }
    }
}
```

**Step 4: Add comment_style() to ThemeColors**

Modify `src/theme/colors.rs` to add comment styling:

```rust
impl ThemeColors {
    pub fn comment_style(&self) -> Style {
        Style::default()
            .fg(self.comment)
            .add_modifier(Modifier::ITALIC)
    }
}
```

**Step 5: Add comment color to theme definitions**

Add `comment: Color` field to ThemeColors struct and update all theme definitions.

Use dim gray for most themes: `Color::Rgb(128, 128, 128)`

**Step 6: Implement inline rendering for Line comments**

This is tricky - need to track previous node and append comment to its line.

Modify render_tree_view to pass context about previous line:

```rust
// In render_tree_view, track if last line can have inline comment
let mut last_line_index = None;

for (index, node) in visible_nodes.iter().enumerate() {
    if node.is_comment() {
        if let YamlValue::Comment(c) = node.value() {
            if matches!(c.position(), CommentPosition::Line) {
                // Append to previous line if possible
                if let Some(prev_index) = last_line_index {
                    // Modify previous line to append comment
                    // lines[prev_index] += format!("  # {}", c.content())
                }
            }
        }
    } else {
        last_line_index = Some(lines.len());
    }
}
```

**Step 7: Handle comment cursor highlighting**

When cursor is on a comment node, highlight the entire comment line.

Modify cursor rendering logic to detect comment nodes:

```rust
if node.is_comment() {
    // Apply highlight to entire comment line
    line_style = line_style.add_modifier(Modifier::REVERSED);
}
```

**Step 8: Run tests**

Run: `cd ~/.config/superpowers/worktrees/yamlquill/comment-support && cargo test comment_display`

Expected: Tests may still fail - iterate on rendering logic

**Step 9: Manual testing**

```bash
cd ~/.config/superpowers/worktrees/yamlquill/comment-support
cargo build
cargo run -- examples/complex.yaml
```

Verify:
- Comments appear in tree view
- Line comments appear inline
- Comments have gray/dim styling
- Can navigate to comments with j/k
- Cursor highlights comments correctly

**Step 10: Run full test suite**

Run: `cd ~/.config/superpowers/worktrees/yamlquill/comment-support && cargo test`

Expected: All tests pass

**Step 11: Commit**

```bash
cd ~/.config/superpowers/worktrees/yamlquill/comment-support
cargo fmt
cargo clippy -- -D warnings
cargo test
git add src/ui/tree_view.rs src/theme/colors.rs tests/comment_display_tests.rs
git commit -m "feat: display comments in tree view

Render comments as navigable lines with gray/italic styling.
Line comments appear inline with values. Above/Below/Standalone
comments appear as separate lines.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 4: Add Comment Editing Keybindings

**Files:**
- Modify: `src/input/keys.rs` (InputEvent enum and map_key_event)
- Modify: `src/input/handler.rs` (handle_input implementation)
- Test: `tests/comment_editing_tests.rs` (create new)

**Step 1: Write failing test for adding comments**

```rust
// tests/comment_editing_tests.rs
use yamlquill::editor::state::EditorState;
use yamlquill::input::keys::{InputEvent, map_key_event};
use termion::event::{Event, Key};

#[test]
fn test_add_comment_keybinding() {
    let event = Event::Key(Key::Char('c'));
    let input = map_key_event(event, &yamlquill::editor::mode::EditorMode::Normal);
    assert_eq!(input, InputEvent::AddComment);
}

#[test]
fn test_add_comment_on_value_node() {
    // Create editor state with simple YAML
    let yaml = "key: value\n";
    let mut state = EditorState::from_yaml(yaml).unwrap();

    // Simulate pressing 'c' on value node
    // Should prompt for position
    // TODO: Test interaction flow
}

#[test]
fn test_edit_comment() {
    // Create editor state with comment
    let yaml = "# Test comment\nkey: value\n";
    let mut state = EditorState::from_yaml(yaml).unwrap();

    // Move cursor to comment (first line)
    state.cursor_mut().set_path(vec![]);

    // Press 'e' to edit
    // Should open edit prompt with current comment text
}

#[test]
fn test_delete_comment() {
    // Create editor state with comment
    let yaml = "# Test comment\nkey: value\n";
    let mut state = EditorState::from_yaml(yaml).unwrap();

    // Move cursor to comment
    // Press 'dd' to delete
    // Comment should be removed
}
```

**Step 2: Run tests to verify they fail**

Run: `cd ~/.config/superpowers/worktrees/yamlquill/comment-support && cargo test comment_editing`

Expected: FAIL - InputEvent::AddComment doesn't exist

**Step 3: Add AddComment to InputEvent enum**

Modify `src/input/keys.rs`:

```rust
pub enum InputEvent {
    // ... existing variants ...
    /// Add a comment (c key on value node)
    AddComment,
    // ... rest of variants ...
}
```

**Step 4: Map 'c' key to AddComment in Normal mode**

Modify `map_key_event` in `src/input/keys.rs`:

```rust
EditorMode::Normal => match key {
    // ... existing mappings ...
    Key::Char('c') => InputEvent::AddComment,
    // ... rest of mappings ...
}
```

**Step 5: Implement AddComment handler**

Modify `src/input/handler.rs`:

```rust
InputEvent::AddComment => {
    let current_path = state.cursor().current_path().to_vec();
    let node = state.tree().get_node(&current_path);

    if let Some(node) = node {
        if node.is_comment() {
            // On comment: add another comment after this one
            add_comment_after_comment(state, &current_path);
        } else {
            // On value: prompt for position (Above/Line/Below)
            prompt_for_comment_position(state, &current_path);
        }
    }

    state.rebuild_tree_view();
}
```

**Step 6: Implement prompt_for_comment_position**

```rust
fn prompt_for_comment_position(state: &mut EditorState, path: &[usize]) {
    // Set editor mode to CommentPosition prompt
    // Display: "Add comment: [a]bove  [l]ine  [b]elow  [Esc to cancel]"
    // Wait for user to press a/l/b
    // Then call add_comment_with_position()

    // For MVP, just add as Above comment
    add_comment_with_position(state, path, CommentPosition::Above);
}

fn add_comment_with_position(state: &mut EditorState, path: &[usize], position: CommentPosition) {
    // Prompt user for comment text
    // Create Comment node
    // Insert into tree at appropriate position

    // For MVP, create simple comment
    let comment_node = YamlNode::new(YamlValue::Comment(CommentNode::new(
        "New comment".to_string(),
        position,
    )));

    // Insert comment before current node
    state.tree_mut().insert_before(path, comment_node);
    state.rebuild_tree_view();
    state.set_message("Comment added", MessageLevel::Info);
}
```

**Step 7: Modify edit handler to detect comments**

Modify InputEvent::EnterInsertMode handler to check if cursor is on comment:

```rust
InputEvent::EnterInsertMode => {
    let current_path = state.cursor().current_path().to_vec();
    let node = state.tree().get_node(&current_path);

    if let Some(node) = node {
        if node.is_comment() {
            // Edit comment
            edit_comment(state, &current_path);
        } else {
            // Edit value (existing behavior)
            // ... existing code ...
        }
    }
}

fn edit_comment(state: &mut EditorState, path: &[usize]) {
    let node = state.tree().get_node(path).unwrap();

    if let YamlValue::Comment(comment) = node.value() {
        let current_text = comment.content().to_string();

        // Open edit prompt with current text
        // For MVP, just show message
        state.set_message(
            format!("Editing comment: {}", current_text),
            MessageLevel::Info
        );

        // TODO: Implement actual edit prompt
    }
}
```

**Step 8: Modify delete handler to support comments**

Delete should already work if comments are proper tree nodes, but verify:

```rust
InputEvent::Delete => {
    let current_path = state.cursor().current_path().to_vec();
    let node = state.tree().get_node(&current_path);

    if let Some(node) = node {
        if node.is_comment() {
            // Delete comment node
            state.tree_mut().delete_node(&current_path);
            state.rebuild_tree_view();
            state.set_message("Comment deleted", MessageLevel::Info);
        } else {
            // Delete value (existing behavior)
            // ... existing code ...
        }
    }
}
```

**Step 9: Run tests**

Run: `cd ~/.config/superpowers/worktrees/yamlquill/comment-support && cargo test comment_editing`

Expected: Some tests pass, others need more implementation

**Step 10: Implement full edit prompt for comments**

Add proper edit prompt UI using existing edit_prompt module:

```rust
use crate::ui::edit_prompt::EditPrompt;

fn edit_comment(state: &mut EditorState, path: &[usize]) {
    let node = state.tree().get_node(path).unwrap();

    if let YamlValue::Comment(comment) = node.value() {
        let current_text = comment.content().to_string();
        let position = comment.position().clone();

        let mut prompt = EditPrompt::new("Comment".to_string(), current_text);

        // Show prompt and get result
        // On commit: update comment node with new text
        // ... integrate with event loop ...
    }
}
```

**Step 11: Manual testing**

```bash
cd ~/.config/superpowers/worktrees/yamlquill/comment-support
cargo build
cargo run -- examples/complex.yaml
```

Test:
- Press 'c' on a value → should prompt for position
- Press 'e' on a comment → should open edit prompt
- Press 'dd' on a comment → should delete comment
- Verify undo/redo works with comment operations

**Step 12: Run full test suite**

Run: `cd ~/.config/superpowers/worktrees/yamlquill/comment-support && cargo test`

Expected: All tests pass

**Step 13: Commit**

```bash
cd ~/.config/superpowers/worktrees/yamlquill/comment-support
cargo fmt
cargo clippy -- -D warnings
cargo test
git add src/input/keys.rs src/input/handler.rs tests/comment_editing_tests.rs
git commit -m "feat: add comment editing keybindings

Add 'c' to add comments, 'e' to edit, 'dd' to delete.
Prompt for position (Above/Line/Below) when adding.
Comment editing integrated with undo/redo.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 5: Save Comments to YAML

**Files:**
- Modify: `src/file/saver.rs` (save_yaml function)
- Test: `tests/comment_roundtrip_tests.rs` (create new)

**Step 1: Write failing roundtrip test**

```rust
// tests/comment_roundtrip_tests.rs
use yamlquill::document::parser::parse_yaml;
use yamlquill::file::saver::save_to_string;

#[test]
fn test_roundtrip_above_comment() {
    let yaml = r#"# Above comment
key: value
"#;

    let tree = parse_yaml(yaml).unwrap();
    let output = save_to_string(&tree).unwrap();

    assert!(output.contains("# Above comment"));
    assert!(output.contains("key: value"));
}

#[test]
fn test_roundtrip_line_comment() {
    let yaml = "key: value  # line comment\n";

    let tree = parse_yaml(yaml).unwrap();
    let output = save_to_string(&tree).unwrap();

    assert!(output.contains("key: value"));
    assert!(output.contains("# line comment"));
    // Should be on same line
    assert!(output.contains("value  # line comment") || output.contains("value # line comment"));
}

#[test]
fn test_roundtrip_multiple_comments() {
    let yaml = r#"# Top comment
key1: value1
# Middle comment
key2: value2  # inline
"#;

    let tree = parse_yaml(yaml).unwrap();
    let output = save_to_string(&tree).unwrap();

    assert!(output.contains("# Top comment"));
    assert!(output.contains("# Middle comment"));
    assert!(output.contains("# inline"));
}

#[test]
fn test_roundtrip_array_comments() {
    let yaml = r#"items:
  # First item comment
  - item1
  - item2  # inline
"#;

    let tree = parse_yaml(yaml).unwrap();
    let output = save_to_string(&tree).unwrap();

    assert!(output.contains("# First item comment"));
    assert!(output.contains("# inline"));
}
```

**Step 2: Run tests to verify they fail**

Run: `cd ~/.config/superpowers/worktrees/yamlquill/comment-support && cargo test comment_roundtrip`

Expected: FAIL - comments not in output

**Step 3: Add inject_comments helper to saver.rs**

```rust
// src/file/saver.rs

fn inject_comments(yaml_text: String, tree: &YamlTree) -> String {
    // Walk tree and collect comments with their target positions
    let comments = collect_comments(tree.root(), 0);

    // Split yaml_text into lines
    let lines: Vec<&str> = yaml_text.lines().collect();
    let mut output = Vec::new();

    // Insert comments at appropriate positions
    for (line_num, line) in lines.iter().enumerate() {
        // Check for Above comments for this line
        if let Some(above) = comments.above.get(&line_num) {
            for comment_text in above {
                output.push(format!("# {}", comment_text));
            }
        }

        // Add the line itself
        let mut line_str = line.to_string();

        // Check for Line comment
        if let Some(inline) = comments.inline.get(&line_num) {
            line_str = format!("{}  # {}", line_str, inline);
        }

        output.push(line_str);

        // Check for Below comments
        if let Some(below) = comments.below.get(&line_num) {
            for comment_text in below {
                output.push(format!("# {}", comment_text));
            }
        }
    }

    output.join("\n") + "\n"
}

struct CommentCollector {
    above: HashMap<usize, Vec<String>>,
    inline: HashMap<usize, String>,
    below: HashMap<usize, Vec<String>>,
}

fn collect_comments(node: &YamlNode, depth: usize) -> CommentCollector {
    // Walk tree and collect all comments
    // Track line numbers for each comment
    // Return mapping of line_num -> comment_text

    let mut collector = CommentCollector {
        above: HashMap::new(),
        inline: HashMap::new(),
        below: HashMap::new(),
    };

    // TODO: Implement tree walking logic

    collector
}
```

**Step 4: Integrate inject_comments into save_yaml**

Modify `save_yaml` in `src/file/saver.rs`:

```rust
pub fn save_yaml(tree: &YamlTree, path: &Path) -> Result<()> {
    // Generate YAML from tree using serde_yaml
    let yaml_text = serialize_tree(tree)?;

    // Inject comments
    let yaml_with_comments = inject_comments(yaml_text, tree);

    // Write to file
    fs::write(path, yaml_with_comments)?;

    Ok(())
}
```

**Step 5: Implement collect_comments logic**

This is complex - need to:
1. Walk tree in display order
2. Track line numbers as we serialize
3. Match comment nodes to their associated value nodes
4. Build mapping of line -> comments

```rust
fn collect_comments(node: &YamlNode, depth: usize) -> CommentCollector {
    let mut collector = CommentCollector::default();

    match node.value() {
        YamlValue::Object(map) => {
            let mut current_line = depth;

            for (key, value) in map.iter() {
                // Check if this is a comment node
                if value.is_comment() {
                    if let YamlValue::Comment(comment) = value.value() {
                        match comment.position() {
                            CommentPosition::Above => {
                                collector.above.entry(current_line).or_insert_with(Vec::new)
                                    .push(comment.content().to_string());
                            }
                            CommentPosition::Line => {
                                collector.inline.insert(current_line, comment.content().to_string());
                            }
                            CommentPosition::Below => {
                                collector.below.entry(current_line).or_insert_with(Vec::new)
                                    .push(comment.content().to_string());
                            }
                            CommentPosition::Standalone => {
                                collector.above.entry(current_line).or_insert_with(Vec::new)
                                    .push(comment.content().to_string());
                            }
                        }
                    }
                } else {
                    // Recursively collect from children
                    let child_comments = collect_comments(value, depth + 1);
                    merge_collectors(&mut collector, child_comments);
                    current_line += 1;
                }
            }
        }
        YamlValue::Array(arr) => {
            let mut current_line = depth;
            for child in arr.iter() {
                if child.is_comment() {
                    // Handle comment
                } else {
                    let child_comments = collect_comments(child, depth + 1);
                    merge_collectors(&mut collector, child_comments);
                    current_line += 1;
                }
            }
        }
        _ => {}
    }

    collector
}
```

**Step 6: Test simple case first**

Focus on getting one test to pass before implementing full logic.

Start with `test_roundtrip_above_comment`:

Debug: Print yaml_text before and after inject_comments
Verify comments are being collected correctly
Verify injection is happening at right lines

**Step 7: Iterate on implementation**

Work through each test case:
1. Above comments
2. Line comments
3. Below comments
4. Standalone comments
5. Array comments
6. Nested structure comments

For each failing test, debug and fix the logic.

**Step 8: Handle edge cases**

- Empty files with only comments
- Multi-document files
- Comments in deeply nested structures
- Multiple comments at same position
- Comments with special characters

**Step 9: Run tests**

Run: `cd ~/.config/superpowers/worktrees/yamlquill/comment-support && cargo test comment_roundtrip`

Expected: All roundtrip tests pass

**Step 10: Run full test suite**

Run: `cd ~/.config/superpowers/worktrees/yamlquill/comment-support && cargo test`

Expected: All 390+ tests pass

**Step 11: Manual testing**

```bash
cd ~/.config/superpowers/worktrees/yamlquill/comment-support
cargo build

# Create test file with comments
cat > /tmp/test_comments.yaml << 'EOF'
# Top level comment
application:
  name: test  # inline comment
  # Config section
  config:
    enabled: true
EOF

# Open, edit, and save
cargo run -- /tmp/test_comments.yaml
# Make some edits
# Save with :w
# Quit with :q

# Verify comments preserved
cat /tmp/test_comments.yaml
```

**Step 12: Commit**

```bash
cd ~/.config/superpowers/worktrees/yamlquill/comment-support
cargo fmt
cargo clippy -- -D warnings
cargo test
git add src/file/saver.rs tests/comment_roundtrip_tests.rs
git commit -m "feat: preserve comments when saving YAML

Inject comments back into serde_yaml output at correct positions.
Supports Above, Line, Below, and Standalone comment positions.
Full roundtrip: parse → edit → save → parse preserves comments.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 6: Integration Testing and Documentation

**Files:**
- Create: `tests/comment_integration_tests.rs`
- Modify: `CLAUDE.md` (update status and features)
- Modify: `README.md` (document comment support)
- Create: `docs/comment-editing-guide.md`

**Step 1: Write comprehensive integration tests**

```rust
// tests/comment_integration_tests.rs

#[test]
fn test_full_workflow_add_edit_delete() {
    // Load YAML without comments
    // Add comment via keybinding
    // Edit comment
    // Delete comment
    // Verify tree state at each step
}

#[test]
fn test_undo_redo_comments() {
    // Add comment
    // Undo -> comment gone
    // Redo -> comment back
    // Edit comment
    // Undo -> original text
    // Redo -> edited text
}

#[test]
fn test_yank_paste_comments() {
    // Yank a comment node
    // Paste elsewhere
    // Verify comment copied correctly
}

#[test]
fn test_visual_mode_comments() {
    // Select range including comments
    // Delete range
    // Verify comments deleted with values
}

#[test]
fn test_search_with_comments() {
    // Search should skip comment nodes
    // Or should it search comment text too?
    // Define expected behavior
}
```

**Step 2: Run integration tests**

Run: `cd ~/.config/superpowers/worktrees/yamlquill/comment-support && cargo test comment_integration`

Expected: All integration tests pass

**Step 3: Update CLAUDE.md**

Document comment support feature:

```markdown
## Comment Support (v2.0)

YAMLQuill supports full comment editing:

- Comments are first-class navigable tree nodes
- Navigate to comments with j/k like any other line
- Add comments: press 'c' on a value node, choose position (Above/Line/Below)
- Edit comments: press 'e' on a comment node
- Delete comments: press 'dd' on a comment node
- Comments preserved on save with correct positioning

Supported comment positions:
- **Above**: Comment lines before a value
- **Line**: Inline comment after a value (same line)
- **Below**: Comment after children/end of block
- **Standalone**: Comment between blank lines

Limitations:
- Multi-line comments are single nodes (newlines in content)
- No comment templates or special formatting
```

**Step 4: Update README.md**

Add comment support to feature list and keybindings section.

**Step 5: Write user guide**

```markdown
# Comment Editing Guide

## Overview

YAMLQuill treats comments as first-class elements in the document tree.
You can navigate to, edit, and delete comments just like any other node.

## Adding Comments

1. Position cursor on a value node
2. Press `c`
3. Choose position:
   - `a` - Above: Comment before the value
   - `l` - Line: Inline comment after the value
   - `b` - Below: Comment after children/end of block
4. Type comment text (without `#` prefix)
5. Press Enter to commit

## Editing Comments

1. Navigate cursor to a comment line (use j/k)
2. Press `e`
3. Edit comment text
4. Press Enter to commit, or Esc to cancel

## Deleting Comments

1. Navigate cursor to a comment line
2. Press `dd`
3. Comment is removed (undo with `u`)

## Comment Display

Comments appear with dim gray styling in the tree view:

- Above/Below/Standalone: Separate lines with `# ` prefix
- Line: Inline after value with `  # ` separator

When cursor is on a comment, the entire line is highlighted.

## Tips

- Comments are included in undo/redo history
- Comments can be yanked and pasted like other nodes
- Use visual mode to select and delete multiple comments
- Comments are preserved when saving YAML files
```

**Step 6: Run final test suite**

Run: `cd ~/.config/superpowers/worktrees/yamlquill/comment-support && cargo test`

Expected: All tests pass (390 + ~30 new comment tests = ~420 total)

**Step 7: Run clippy and fmt**

```bash
cd ~/.config/superpowers/worktrees/yamlquill/comment-support
cargo fmt
cargo clippy -- -D warnings
```

Expected: No warnings or errors

**Step 8: Commit documentation**

```bash
cd ~/.config/superpowers/worktrees/yamlquill/comment-support
git add tests/comment_integration_tests.rs CLAUDE.md README.md docs/comment-editing-guide.md
git commit -m "docs: document comment support feature

Add comprehensive documentation for comment editing:
- Update CLAUDE.md with feature description
- Update README.md with keybindings
- Add user guide for comment editing workflow
- Add integration tests for full workflows

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

**Step 9: Create summary commit**

```bash
cd ~/.config/superpowers/worktrees/yamlquill/comment-support
git log --oneline | head -10
# Review all commits

# Tag the completion
git tag v2.0.0-comment-support
```

---

## Completion Checklist

Before finishing:

- [ ] All tests pass (cargo test)
- [ ] No clippy warnings (cargo clippy -- -D warnings)
- [ ] Code formatted (cargo fmt)
- [ ] Documentation updated (CLAUDE.md, README.md)
- [ ] User guide written
- [ ] Integration tests cover main workflows
- [ ] Manual testing completed
- [ ] All commits have proper messages
- [ ] Feature tagged appropriately

## Known Issues / Future Work

- Multi-line comment editing requires Shift+Enter support
- Comment templates (TODO, FIXME, NOTE) not implemented
- No syntax highlighting within comments
- Comment folding not implemented
- Search within comment text not implemented

---

**Ready to execute? Use superpowers:executing-plans or superpowers:subagent-driven-development to implement this plan task by task.**
