# Comment Support Design

**Date:** 2026-02-01
**Status:** Approved
**Target:** v2.0

## Overview

Add full comment editing support to YAMLQuill. Comments become first-class navigable nodes in the tree structure, allowing users to add, edit, and delete comments with vim-style keybindings.

## Requirements

- Full comment editing capabilities (not just preservation)
- Support all comment positions: above, inline, below, standalone
- Inline display in tree view where appropriate
- Vim-style keybindings:
  - `c` on value node: add comment (prompt for position)
  - `e` on comment node: edit comment
  - `dd` on comment node: delete comment
- Comments preserved on save
- Navigate to comments with j/k like any other node

## Design

### Part 1: Data Model

Comments are first-class nodes in the tree structure.

**Core Changes:**

Add Comment variant to YamlValue enum:

```rust
pub enum YamlValue {
    Null,
    Bool(bool),
    Number(YamlNumber),
    String(YamlString),
    Array(Vec<YamlNode>),
    Object(IndexMap<String, YamlNode>),
    Alias(String),
    MultiDoc(Vec<YamlNode>),
    Comment(CommentNode),  // NEW
}

pub struct CommentNode {
    pub content: String,        // Comment text without '#'
    pub position: CommentPosition,
}

pub enum CommentPosition {
    Above,      // Comment line(s) before a value
    Line,       // Inline comment after a value
    Below,      // Comment after children/end of block
    Standalone, // Comment between blank lines
}
```

**Tree Structure:**

Comments are siblings in arrays or objects. Example:

```yaml
# Above comment
key: value  # line comment
```

Becomes tree structure:
- Comment node (position: Above, content: "Above comment")
- Key-value node
- Comment node (position: Line, content: "line comment")

**Display Logic:**

The renderer knows to display `Line` positioned comments inline with the previous node.

**No metadata fields needed** - comments are just nodes like any other.

### Part 2: Comment Extraction

Insert Comment nodes directly into tree structure during parsing.

**Extraction Process:**

1. **Integrated Parsing**: Modify `TreeBuilder` in `src/document/parser.rs` to track pending comments
   - As Scanner tokens arrive, accumulate comments in a buffer
   - When a value/key token arrives, flush buffered comments as nodes

2. **Comment Position Detection**:
   - **Above**: Comments on consecutive lines before a key/value → insert as siblings before the value node with `CommentPosition::Above`
   - **Line**: Comment on same line as a value (track line numbers from Scanner) → insert immediately after value node with `CommentPosition::Line`
   - **Below**: Comments after last child but before next sibling → insert as last child with `CommentPosition::Below`
   - **Standalone**: Comments surrounded by blank lines → insert as siblings with `CommentPosition::Standalone`

3. **Tree Insertion**:
   - For objects: Comments become entries in the IndexMap (need special key like `__comment_N__`)
   - For arrays: Comments are elements in the Vec
   - Top-level: Comments are siblings of root nodes

4. **Scanner Integration**:
   ```rust
   let mut pending_comments = Vec::new();
   for token in scanner {
       match token {
           Token::Comment(content, line, col) => {
               pending_comments.push((content, line, col));
           }
           Token::Scalar(_, _, line, col) => {
               // Flush pending_comments as Above
               // Create value node
               // Check for same-line comment as Line
           }
           // ... other tokens
       }
   }
   ```

### Part 3: Display Rendering

Comments are selectable nodes in the tree view.

**Visual Layout:**

1. **Line Comments** (special inline rendering):
   ```
   > name: "fleet-manager"  # Production config
     port: 8080  # HTTP port
   ```
   - Rendered on same line as preceding value node
   - Gray color (theme's comment style)
   - When cursor is on the comment node, highlight extends to include the `#` and text
   - Two spaces before `#` for separation

2. **Above/Below/Standalone Comments** (normal line rendering):
   ```
   > # Database configuration
     # Updated 2026-01-15
     database:
       host: "localhost"
   ```
   - Each comment is a selectable line
   - `>` cursor indicator when selected
   - Gray color, italic if supported
   - Indented to match their context level

**Cursor Behavior:**

- `j`/`k` navigate to comment nodes like any other node
- When cursor is on a comment:
  - `e` → edit comment text
  - `dd` → delete comment node
  - `i` → insert new value node after comment
  - `c` → insert new comment after current comment
- When cursor is on a value node:
  - `c` → insert new comment (prompt for Above/Line/Below position)

**Implementation:**

- Modify `src/ui/tree_view.rs::render_node()`
- Add `is_comment()` helper to YamlNode
- Special case: render Line positioned comments on same line as previous node
- Track "previous node" during rendering to handle Line comments

### Part 4: Editing Workflow

Complete workflow for adding, editing, and deleting comments.

**Adding Comments** (cursor on value node, press `c`):

1. Prompt user for position:
   ```
   Add comment: [a]bove  [l]ine  [b]elow  [Esc to cancel]
   ```
2. User selects position (a/l/b)
3. Open edit prompt: `Comment: _`
4. User types comment text (without `#` prefix)
5. Insert new Comment node at selected position relative to current node
6. Cursor moves to newly created comment node

**Editing Comments** (cursor on comment node, press `e`):

1. Open edit prompt with current text: `Comment: existing text_`
2. User edits text
3. Update Comment node content
4. Cursor stays on comment node

**Editing Values** (cursor on value node, press `e`):

- Works as before - edits the value
- No change to existing behavior

**Deleting Comments** (cursor on comment node, press `dd`):

1. Remove Comment node from tree
2. Cursor moves to next sibling or parent
3. No confirmation needed (matches vim behavior)

**Deleting Values** (cursor on value node, press `dd`):

- Works as before - deletes the value and any associated Line comments
- If value has Above/Below comments, they remain as orphaned Standalone comments

**Multi-line Comments**:

- Single comment node can contain newlines
- Edit prompt supports multi-line input (Shift+Enter for newline, Enter to commit)
- Display shows each line separately but they're one node

### Part 5: Save/Serialization

Preserve comments when saving YAML files.

**Serialization Process:**

1. **Two-Phase Save**:
   - Phase 1: Use serde_yaml to serialize non-comment nodes to YAML text
   - Phase 2: Walk tree and inject comments at correct positions

2. **Comment Injection**:
   - Build a map of line numbers to comment nodes during tree walk
   - Parse serde_yaml output line-by-line
   - Insert comment lines at appropriate positions based on CommentPosition

3. **Position Handling**:
   - **Above**: Insert comment lines immediately before the value's line
   - **Line**: Append ` # comment` to end of value's line
   - **Below**: Insert comment lines after the value's closing line (after children)
   - **Standalone**: Insert comment lines at their tracked position

4. **Line Number Tracking**:
   - As we serialize, track which tree node corresponds to which output line
   - Match Comment nodes to their associated value nodes
   - Calculate injection points

5. **Implementation Location**:
   - Modify `src/file/saver.rs::save_yaml()`
   - New helper: `inject_comments(yaml_text: String, tree: &YamlTree) -> String`
   - Walk tree in display order, collect comments with target line numbers
   - Reconstruct YAML with comments interleaved

**Edge Cases:**
- Empty file with only comments → write comments as-is
- Multi-document files → track document boundaries, inject per-document
- Line comments on multi-line values → attach to first line

## Implementation Plan

1. Add CommentNode and CommentPosition types to `src/document/node.rs`
2. Add YamlValue::Comment variant
3. Modify TreeBuilder to extract comments during parsing
4. Update tree_view.rs for comment display
5. Add comment keybindings to input/keys.rs and input/handler.rs
6. Implement comment injection in file/saver.rs
7. Add comprehensive tests for all comment positions and operations

## Testing Strategy

- Unit tests for comment extraction from various YAML structures
- Tests for each comment position (Above, Line, Below, Standalone)
- Round-trip tests (parse → edit → save → parse)
- Navigation tests (cursor movement to/from comments)
- Editing tests (add, edit, delete operations)
- Multi-document comment tests

## Migration Path

- Existing YAML files without comments work unchanged
- Files with comments can be loaded, edited, and saved
- No breaking changes to existing functionality
- Comments are optional - tree works with or without them

## Future Enhancements

- Syntax highlighting in multi-line comments
- Comment templates (e.g., "TODO:", "FIXME:", "NOTE:")
- Fold/unfold comment blocks
- Search within comments
