# Phase 2c: Fix Value Editing Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix critical editing bugs and add test coverage for value editing

**Architecture:** Fix string style preservation in commit_editing(), add validation to prevent data loss, comprehensive test coverage

**Tech Stack:** Rust 2021, existing editing infrastructure from JSONQuill

---

## Task 1: Fix String Style Preservation Bug

**Files:**
- Modify: `src/editor/state.rs:2872` (commit_editing function)

**Current Bug:**
When editing Literal or Folded strings, they always convert to Plain style, corrupting YAML formatting.

**Step 1: Locate the bug**

Run: `grep -n "YamlValue::String(_) =>" src/editor/state.rs`
Expected: Find line 2872 with the problematic code

**Step 2: Read the current implementation**

Run: `sed -n '2870,2875p' src/editor/state.rs`
Expected: See the code that always creates Plain strings

**Step 3: Fix the bug to preserve string style**

Change line 2872 from:
```rust
YamlValue::String(_) => YamlValue::String(YamlString::Plain(buffer_content)),
```

To:
```rust
YamlValue::String(original_style) => {
    // Preserve the original string style (Plain, Literal, or Folded)
    let new_string = match original_style {
        YamlString::Plain(_) => YamlString::Plain(buffer_content),
        YamlString::Literal(_) => YamlString::Literal(buffer_content),
        YamlString::Folded(_) => YamlString::Folded(buffer_content),
    };
    YamlValue::String(new_string)
}
```

**Step 4: Add missing import if needed**

Check if YamlString is imported. If not, add to the use statement at the top of commit_editing:
```rust
use crate::document::node::{YamlValue, YamlString, YamlNumber};
```

**Step 5: Build and check for errors**

Run: `cargo build`
Expected: Compiles successfully

**Step 6: Commit the fix**

```bash
git add src/editor/state.rs
git commit -m "fix: preserve YamlString style when editing

- Fix critical bug where Literal/Folded strings became Plain
- Now preserves original string style (Plain/Literal/Folded)
- Prevents corruption of multi-line YAML formatting

Fixes string style preservation issue identified in Phase 2c analysis.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 2: Add Input Validation

**Files:**
- Modify: `src/editor/state.rs` (add validation before commit_editing parses)

**Step 1: Add validation helper function**

Add this function before `commit_editing()` (around line 2840):

```rust
/// Validate that the edit buffer content is valid for the given value type.
/// Returns Ok(()) if valid, Err with message if invalid.
fn validate_edit_input(buffer: &str, original_value: &YamlValue) -> anyhow::Result<()> {
    use anyhow::anyhow;

    match original_value {
        YamlValue::Number(_) => {
            // Try parsing as integer or float
            if buffer.parse::<i64>().is_err() && buffer.parse::<f64>().is_err() {
                return Err(anyhow!("Invalid number format: '{}'", buffer));
            }
        }
        YamlValue::Boolean(_) => {
            // Must be exactly "true" or "false"
            if !matches!(buffer, "true" | "false") {
                return Err(anyhow!(
                    "Invalid boolean: '{}' (must be 'true' or 'false')",
                    buffer
                ));
            }
        }
        YamlValue::Alias(_) => {
            // Must start with *
            if !buffer.starts_with('*') {
                return Err(anyhow!("Alias must start with '*'"));
            }
        }
        // Strings and Null accept any input
        YamlValue::String(_) | YamlValue::Null => {}
        // Containers shouldn't be editable
        YamlValue::Object(_) | YamlValue::Array(_) | YamlValue::MultiDoc(_) => {
            return Err(anyhow!("Cannot edit container types"));
        }
    }

    Ok(())
}
```

**Step 2: Call validation at start of commit_editing**

In `commit_editing()`, add validation after getting the node (around line 2865):

```rust
let node = self
    .tree
    .get_node(path)
    .ok_or_else(|| anyhow!("Node not found at cursor"))?;

// Validate input before attempting to parse
validate_edit_input(&buffer_content, node.value())?;

// Special case: "null" always converts to Null regardless of original type
let new_value = if buffer_content == "null" {
    // ... rest of function
```

**Step 3: Build and test**

Run: `cargo build`
Expected: Compiles successfully

**Step 4: Manually test validation**

Create test file `test_validation.yaml`:
```yaml
count: 42
enabled: true
```

Run: `cargo run -- test_validation.yaml`

Test cases:
1. Edit "count" to "abc" → should show error, not crash
2. Edit "enabled" to "maybe" → should show error
3. Edit "count" to "123" → should work
4. Edit "enabled" to "false" → should work

**Step 5: Commit**

```bash
git add src/editor/state.rs
git commit -m "feat: add input validation before committing edits

- Add validate_edit_input() to check validity before parsing
- Validate numbers, booleans, and aliases
- Show clear error messages for invalid input
- Prevents data loss from invalid edits

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 3: Add Test Coverage for Editing

**Files:**
- Modify: `src/editor/state.rs` (add tests in #[cfg(test)] section)

**Step 1: Find the test module**

Run: `grep -n "^#\[cfg(test)\]" src/editor/state.rs`
Expected: Find the test module location (around line 4489)

**Step 2: Add test for editing plain string**

Add at the end of the test module:

```rust
#[test]
fn test_edit_plain_string() -> anyhow::Result<()> {
    use crate::document::node::{YamlNode, YamlValue, YamlString};
    use crate::document::tree::YamlTree;
    use indexmap::IndexMap;

    // Create tree with plain string
    let mut map = IndexMap::new();
    map.insert(
        "name".to_string(),
        YamlNode::new(YamlValue::String(YamlString::Plain("Alice".to_string()))),
    );
    let tree = YamlTree::new(YamlNode::new(YamlValue::Object(map)));
    let mut state = EditorState::new(tree, None);

    // Navigate to "name" field
    state.cursor_mut().push("name".to_string());

    // Enter edit mode
    state.edit_buffer = Some("Bob".to_string());

    // Commit edit
    state.commit_editing()?;

    // Verify: value changed and still Plain
    let node = state.tree().get_node("name").unwrap();
    match node.value() {
        YamlValue::String(YamlString::Plain(s)) if s == "Bob" => (),
        _ => panic!("Expected Plain string 'Bob', got {:?}", node.value()),
    }

    assert!(state.is_dirty());
    Ok(())
}
```

**Step 3: Add CRITICAL test for Literal string preservation**

```rust
#[test]
fn test_edit_literal_string_preserves_style() -> anyhow::Result<()> {
    use crate::document::node::{YamlNode, YamlValue, YamlString};
    use crate::document::tree::YamlTree;
    use indexmap::IndexMap;

    // Create tree with Literal string (multi-line with |)
    let mut map = IndexMap::new();
    map.insert(
        "description".to_string(),
        YamlNode::new(YamlValue::String(YamlString::Literal(
            "Line 1\nLine 2".to_string(),
        ))),
    );
    let tree = YamlTree::new(YamlNode::new(YamlValue::Object(map)));
    let mut state = EditorState::new(tree, None);

    state.cursor_mut().push("description".to_string());

    // Edit the content
    state.edit_buffer = Some("Line 1\nLine 2\nLine 3".to_string());

    // Commit edit
    state.commit_editing()?;

    // CRITICAL: Verify it's STILL Literal, not Plain!
    let node = state.tree().get_node("description").unwrap();
    match node.value() {
        YamlValue::String(YamlString::Literal(s)) if s == "Line 1\nLine 2\nLine 3" => (),
        YamlValue::String(YamlString::Plain(_)) => {
            panic!("BUG: Literal string became Plain after editing!")
        }
        _ => panic!("Expected Literal string, got {:?}", node.value()),
    }

    Ok(())
}
```

**Step 4: Add test for Folded string preservation**

```rust
#[test]
fn test_edit_folded_string_preserves_style() -> anyhow::Result<()> {
    use crate::document::node::{YamlNode, YamlValue, YamlString};
    use crate::document::tree::YamlTree;
    use indexmap::IndexMap;

    // Create tree with Folded string (multi-line with >)
    let mut map = IndexMap::new();
    map.insert(
        "text".to_string(),
        YamlNode::new(YamlValue::String(YamlString::Folded(
            "This is a long paragraph".to_string(),
        ))),
    );
    let tree = YamlTree::new(YamlNode::new(YamlValue::Object(map)));
    let mut state = EditorState::new(tree, None);

    state.cursor_mut().push("text".to_string());
    state.edit_buffer = Some("This is a modified paragraph".to_string());
    state.commit_editing()?;

    // Verify it's STILL Folded
    let node = state.tree().get_node("text").unwrap();
    match node.value() {
        YamlValue::String(YamlString::Folded(s)) if s == "This is a modified paragraph" => (),
        _ => panic!("Expected Folded string, got {:?}", node.value()),
    }

    Ok(())
}
```

**Step 5: Add test for integer editing**

```rust
#[test]
fn test_edit_integer() -> anyhow::Result<()> {
    use crate::document::node::{YamlNode, YamlValue, YamlNumber};
    use crate::document::tree::YamlTree;
    use indexmap::IndexMap;

    let mut map = IndexMap::new();
    map.insert(
        "count".to_string(),
        YamlNode::new(YamlValue::Number(YamlNumber::Integer(42))),
    );
    let tree = YamlTree::new(YamlNode::new(YamlValue::Object(map)));
    let mut state = EditorState::new(tree, None);

    state.cursor_mut().push("count".to_string());
    state.edit_buffer = Some("123".to_string());
    state.commit_editing()?;

    let node = state.tree().get_node("count").unwrap();
    match node.value() {
        YamlValue::Number(YamlNumber::Integer(123)) => (),
        _ => panic!("Expected integer 123, got {:?}", node.value()),
    }

    Ok(())
}
```

**Step 6: Add test for float editing**

```rust
#[test]
fn test_edit_float() -> anyhow::Result<()> {
    use crate::document::node::{YamlNode, YamlValue, YamlNumber};
    use crate::document::tree::YamlTree;
    use indexmap::IndexMap;

    let mut map = IndexMap::new();
    map.insert(
        "price".to_string(),
        YamlNode::new(YamlValue::Number(YamlNumber::Float(19.99))),
    );
    let tree = YamlTree::new(YamlNode::new(YamlValue::Object(map)));
    let mut state = EditorState::new(tree, None);

    state.cursor_mut().push("price".to_string());
    state.edit_buffer = Some("29.99".to_string());
    state.commit_editing()?;

    let node = state.tree().get_node("price").unwrap();
    match node.value() {
        YamlValue::Number(YamlNumber::Float(f)) if (*f - 29.99).abs() < 0.001 => (),
        _ => panic!("Expected float 29.99, got {:?}", node.value()),
    }

    Ok(())
}
```

**Step 7: Add test for boolean editing**

```rust
#[test]
fn test_edit_boolean() -> anyhow::Result<()> {
    use crate::document::node::{YamlNode, YamlValue};
    use crate::document::tree::YamlTree;
    use indexmap::IndexMap;

    let mut map = IndexMap::new();
    map.insert(
        "enabled".to_string(),
        YamlNode::new(YamlValue::Boolean(true)),
    );
    let tree = YamlTree::new(YamlNode::new(YamlValue::Object(map)));
    let mut state = EditorState::new(tree, None);

    state.cursor_mut().push("enabled".to_string());
    state.edit_buffer = Some("false".to_string());
    state.commit_editing()?;

    let node = state.tree().get_node("enabled").unwrap();
    assert_eq!(node.value(), &YamlValue::Boolean(false));

    Ok(())
}
```

**Step 8: Add test for invalid number validation**

```rust
#[test]
fn test_edit_invalid_number_rejected() {
    use crate::document::node::{YamlNode, YamlValue, YamlNumber};
    use crate::document::tree::YamlTree;
    use indexmap::IndexMap;

    let mut map = IndexMap::new();
    map.insert(
        "count".to_string(),
        YamlNode::new(YamlValue::Number(YamlNumber::Integer(42))),
    );
    let tree = YamlTree::new(YamlNode::new(YamlValue::Object(map)));
    let mut state = EditorState::new(tree, None);

    state.cursor_mut().push("count".to_string());
    state.edit_buffer = Some("not_a_number".to_string());

    // Should fail validation
    let result = state.commit_editing();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid number"));

    // Verify original value unchanged
    let node = state.tree().get_node("count").unwrap();
    assert_eq!(node.value(), &YamlValue::Number(YamlNumber::Integer(42)));
}
```

**Step 9: Add test for invalid boolean validation**

```rust
#[test]
fn test_edit_invalid_boolean_rejected() {
    use crate::document::node::{YamlNode, YamlValue};
    use crate::document::tree::YamlTree;
    use indexmap::IndexMap;

    let mut map = IndexMap::new();
    map.insert(
        "enabled".to_string(),
        YamlNode::new(YamlValue::Boolean(true)),
    );
    let tree = YamlTree::new(YamlNode::new(YamlValue::Object(map)));
    let mut state = EditorState::new(tree, None);

    state.cursor_mut().push("enabled".to_string());
    state.edit_buffer = Some("maybe".to_string());

    // Should fail validation
    let result = state.commit_editing();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid boolean"));

    // Verify original value unchanged
    let node = state.tree().get_node("enabled").unwrap();
    assert_eq!(node.value(), &YamlValue::Boolean(true));
}
```

**Step 10: Run all new tests**

Run: `cargo test test_edit -- --nocapture`
Expected: All 8 tests pass

```
test_edit_plain_string ... ok
test_edit_literal_string_preserves_style ... ok
test_edit_folded_string_preserves_style ... ok
test_edit_integer ... ok
test_edit_float ... ok
test_edit_boolean ... ok
test_edit_invalid_number_rejected ... ok
test_edit_invalid_boolean_rejected ... ok
```

**Step 11: Commit**

```bash
git add src/editor/state.rs
git commit -m "test: add comprehensive editing test coverage

- Test editing all scalar types (string, int, float, bool)
- Test CRITICAL string style preservation (Plain/Literal/Folded)
- Test input validation (reject invalid numbers/booleans)
- 8 new tests for editing scenarios

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 4: Verify All Tests Pass and Create Checkpoint

**Files:**
- None (validation step)

**Step 1: Run full test suite**

Run: `cargo test`
Expected: All tests pass including new editing tests

**Step 2: Count test results**

Run: `cargo test 2>&1 | grep "test result:"`
Expected: See total count (should be 312+ tests: 304 previous + 8 new)

**Step 3: Check code quality**

Run: `cargo fmt && cargo clippy -- -D warnings`
Expected: Clean, no warnings

**Step 4: Verify the critical bug is fixed**

Create test file `test_multiline.yaml`:
```yaml
description: |
  Line 1
  Line 2
```

Manual test:
1. Run: `cargo run -- test_multiline.yaml`
2. Navigate to "description" field
3. Press `i` to edit
4. Change "Line 1" to "Modified"
5. Press Enter to commit
6. Press `:w` to save
7. Press `:q` to quit
8. Run: `cat test_multiline.yaml`

Expected output (MUST still have `|`):
```yaml
description: |
  Modified
  Line 2
```

If it shows `description: Modified\nLine 2` (no `|`), the bug is NOT fixed!

**Step 5: Document test results**

Create `docs/phase2c-test-results.md`:
```markdown
# Phase 2c Test Results

**Date:** 2026-02-01
**Tasks Completed:** 3/3

## Test Summary

- **Total tests:** 312 (304 previous + 8 new)
- **Passing:** 312
- **Failing:** 0
- **Ignored:** 3 (Phase 3 multi-doc features)

## New Tests Added

1. ✅ test_edit_plain_string
2. ✅ test_edit_literal_string_preserves_style (CRITICAL)
3. ✅ test_edit_folded_string_preserves_style (CRITICAL)
4. ✅ test_edit_integer
5. ✅ test_edit_float
6. ✅ test_edit_boolean
7. ✅ test_edit_invalid_number_rejected
8. ✅ test_edit_invalid_boolean_rejected

## Manual Testing

- ✅ Literal string editing preserves `|` style
- ✅ Folded string editing preserves `>` style
- ✅ Invalid number input rejected with error
- ✅ Invalid boolean input rejected with error
- ✅ Round-trip editing works correctly

## Bugs Fixed

1. **String style preservation** - Literal/Folded strings now preserve style on edit
2. **Input validation** - Invalid inputs rejected before data loss

## Next Steps

Phase 2d: Editor State Integration (registers, undo/redo)
```

**Step 6: Create checkpoint tag**

```bash
git tag -a phase2c-complete -m "Phase 2c Complete: Fix Value Editing

- Fixed CRITICAL bug: string style preservation (Literal/Folded)
- Added input validation to prevent data corruption
- Added 8 comprehensive editing tests
- All 312 tests passing
- Zero clippy warnings

Ready for Phase 2d."
```

**Step 7: Push changes**

Run: `git push && git push --tags`
Expected: Success

---

## Success Criteria

This plan is complete when:

- ✅ String style preservation bug fixed (Literal/Folded preserved)
- ✅ Input validation prevents invalid edits
- ✅ 8+ new editing tests all pass
- ✅ Full test suite passes (312+ tests)
- ✅ Zero clippy warnings
- ✅ Manual testing confirms editing works correctly
- ✅ Round-trip editing preserves YAML format
- ✅ Checkpoint tag created

## Next Steps

After completing Phase 2c:

- Phase 2d: Editor State Integration (registers, undo/redo with YAML types)
- Phase 2e: YAML-Aware Display (type indicators, color coding)
- Phase 2f: Navigation Enhancements (jump commands, fold improvements)

## Notes

- This is a bug-fix focused phase, not feature addition
- String style preservation is CRITICAL for YAML correctness
- Validation prevents data loss from typos
- Test coverage ensures no regressions
- Follows JSONQuill's editing model (no type conversion, no multi-line input yet)
- Multi-line input support deferred to Phase 4 (YAML-specific features)
