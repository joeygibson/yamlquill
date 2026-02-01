# YAMLQuill Phase 2d: Editor State Integration Testing - Design Document

**Date:** 2026-02-01
**Phase:** 2d of Phase 2
**Status:** Design Complete, Ready for Implementation

## Overview

Phase 2d validates that undo/redo, registers, and visual mode work correctly with YAML editing operations through comprehensive integration tests. The implementation already exists - this phase is primarily about testing and validation.

## Goals

- Validate undo/redo works correctly after editing YAML values
- Validate visual mode operations (yank, delete, paste) with YAML nodes
- Validate registers preserve YAML types correctly
- Create comprehensive integration tests for editor state features
- Fix any bugs discovered during testing

## Current State Assessment

**What's Already Working:**
- ✅ Registers implementation complete (registers.rs with unit tests)
- ✅ Undo/redo infrastructure exists (undo.rs with branching tree)
- ✅ Checkpoints created on edits (commit_editing calls checkpoint())
- ✅ Visual mode exists (select, yank, delete, paste)
- ✅ Input handlers wired up (yy, dd, p, u, Ctrl-r, v)
- ✅ Basic editor state tests pass (15 tests)

**What's Missing:**
- ❌ No integration tests for undo/redo after editing values
- ❌ No integration tests for visual mode with YAML nodes
- ❌ No tests validating type preservation across operations
- ❌ No end-to-end validation of editor state features

## Approach

Create a new integration test file `tests/editor_integration_tests.rs` with comprehensive test scenarios covering all editor state features.

### Test Categories

#### 1. Undo/Redo with Value Editing

**Goal:** Verify undo/redo correctly restores previous states after editing scalar values.

**Test Cases:**
- Edit string value, undo, verify original restored
- Edit number (integer), undo, verify original restored
- Edit number (float), undo, verify original restored
- Edit boolean, undo, verify original restored
- Edit null to value, undo, verify null restored
- Multi-step edits: edit value twice, undo twice, verify original
- Redo after undo: edit, undo, redo, verify edited value restored
- Undo/redo preserves YamlString styles (Plain/Literal/Folded)
- Undo after deleting node
- Redo after undoing delete

**Validation:**
- Original value exactly restored (including type)
- YamlString style preserved (Plain vs Literal vs Folded)
- YamlNumber type preserved (Integer vs Float)
- Cursor position correctly restored
- Tree structure intact after undo/redo

#### 2. Visual Mode Operations

**Goal:** Verify visual mode selection, yank, delete, and paste work correctly with YAML nodes.

**Test Cases:**
- Visual mode yank: select subtree, yank, verify in unnamed register
- Visual mode delete: select subtree, delete, verify removed and in unnamed register
- Visual mode paste: select node, paste, verify selection replaced
- Visual mode with multiple nodes: select range, delete all
- Visual mode yank to named register: "ayy in visual mode
- Exit visual mode after operation
- Visual mode operations mark document dirty

**Validation:**
- Yanked nodes stored in correct register
- Deleted nodes removed from tree and stored in register
- Paste replaces visual selection correctly
- Visual mode exits after operation
- Dirty flag set appropriately

#### 3. Register Type Preservation

**Goal:** Verify registers preserve YAML types exactly (no float→int conversion, etc.).

**Test Cases:**
- Yank string (Plain), paste, verify still Plain (not Literal or Folded)
- Yank string (Literal), paste, verify still Literal
- Yank string (Folded), paste, verify still Folded
- Yank integer, paste, verify still integer (not converted to float)
- Yank float, paste, verify still float
- Yank boolean, paste, verify boolean
- Yank null, paste, verify null
- Yank object, paste, verify structure and ordering intact
- Yank array, paste, verify elements preserved
- Named registers preserve types: "ayy then "ap
- Numbered registers preserve types: yank, delete, "0p gets yank, "1p gets delete

**Validation:**
- Exact type preservation (YamlString::Plain stays Plain)
- YamlNumber::Integer doesn't become Float
- Object key ordering preserved (IndexMap)
- Nested structures preserved correctly
- No data corruption or type coercion

#### 4. Edge Cases & Error Handling

**Goal:** Verify graceful handling of edge cases and error conditions.

**Test Cases:**
- Undo at root (oldest change) returns false, no crash
- Redo with no future returns false, no crash
- Paste with empty unnamed register returns error
- Paste from non-existent named register returns error
- Delete root node (only node in tree) behaves correctly
- Yank then delete same node, both in registers
- Multiple undo beyond initial state
- Redo after making new edit (no redo available)
- Visual mode at root node
- Visual mode with single node selection

**Validation:**
- No panics or crashes
- Appropriate error messages
- Editor remains in valid state
- Dirty flag correct after failed operations

## Implementation Strategy

### Step 1: Create Test File Structure

Create `tests/editor_integration_tests.rs` with:
- Helper functions to build test YAML trees
- Helper functions to create EditorState instances
- Test modules for each category

### Step 2: Implement Test Helpers

```rust
// Helper to create simple YAML tree for testing
fn create_test_tree() -> YamlTree { ... }

// Helper to create EditorState with test tree
fn create_test_editor() -> EditorState { ... }

// Helper to simulate editing a value
fn edit_value(state: &mut EditorState, new_value: &str) { ... }

// Helper to verify node value
fn assert_node_value(state: &EditorState, expected: YamlValue) { ... }
```

### Step 3: Write Tests Incrementally

**Day 1:** Undo/redo tests
- Basic undo after edit
- Basic redo after undo
- Multi-step undo/redo
- Type preservation tests

**Day 2:** Visual mode tests
- Visual yank
- Visual delete
- Visual paste
- Named register integration

**Day 3:** Register type preservation tests
- All scalar types
- Complex structures
- Named and numbered registers

**Day 4:** Edge cases and validation
- Error conditions
- Boundary cases
- Manual testing

### Step 4: Fix Issues as Found

If tests reveal bugs:
1. Document the bug in test comments
2. Create minimal reproduction case
3. Fix the bug in source code
4. Verify test passes
5. Commit fix with test

### Step 5: Manual Validation

After all tests pass:
1. Build editor: `cargo build`
2. Run with example YAML: `cargo run -- examples/sample.yaml`
3. Manually test:
   - Edit values and undo/redo
   - Visual mode selection and operations
   - Yank and paste with different types
4. Verify everything works smoothly in real usage

## Test File Structure

```rust
// tests/editor_integration_tests.rs

use yamlquill::document::node::{YamlNode, YamlValue, YamlString, YamlNumber};
use yamlquill::document::tree::YamlTree;
use yamlquill::editor::state::EditorState;
use indexmap::IndexMap;

// ============================================================================
// Test Helpers
// ============================================================================

fn create_simple_tree() -> YamlTree { ... }
fn create_test_editor() -> EditorState { ... }

// ============================================================================
// Undo/Redo Tests
// ============================================================================

#[test]
fn test_undo_after_edit_string() { ... }

#[test]
fn test_undo_after_edit_integer() { ... }

#[test]
fn test_redo_after_undo() { ... }

#[test]
fn test_undo_preserves_yaml_string_style() { ... }

// ============================================================================
// Visual Mode Tests
// ============================================================================

#[test]
fn test_visual_mode_yank() { ... }

#[test]
fn test_visual_mode_delete() { ... }

#[test]
fn test_visual_mode_paste_replaces_selection() { ... }

// ============================================================================
// Register Type Preservation Tests
// ============================================================================

#[test]
fn test_register_preserves_plain_string() { ... }

#[test]
fn test_register_preserves_integer_not_float() { ... }

#[test]
fn test_named_register_preserves_types() { ... }

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_undo_at_root_returns_false() { ... }

#[test]
fn test_paste_empty_register_errors() { ... }
```

## Success Criteria

Phase 2d is complete when:

### Functional Requirements
- ✅ Can undo/redo after editing any scalar value
- ✅ Undo/redo preserves exact YAML types (no type coercion)
- ✅ Visual mode yank/delete/paste work with YAML nodes
- ✅ Registers preserve YAML types exactly
- ✅ Named registers work correctly
- ✅ Edge cases handled gracefully (no crashes)

### Technical Requirements
- ✅ All new integration tests pass (20+ tests)
- ✅ No regressions (all existing tests still pass)
- ✅ Zero clippy warnings
- ✅ Code formatted with cargo fmt
- ✅ Manual testing validates real-world usage

### Quality Requirements
- ✅ Tests catch real bugs (not just passing)
- ✅ Test coverage on editor state increased
- ✅ No data corruption in any operation
- ✅ Editor remains in valid state after all operations

## Deliverables

**Code:**
- `tests/editor_integration_tests.rs` - New integration test file
- Bug fixes in source code (if issues found)

**Documentation:**
- This design document
- Implementation plan (via superpowers:writing-plans)
- Updated CLAUDE.md with Phase 2d completion status

**Git:**
- Commits for test file and any bug fixes
- Tag: `phase2d-complete` on completion

## Out of Scope

- ❌ Type conversion UI (`:convert` command) - not needed
- ❌ Multi-document support - Phase 3
- ❌ Anchors/aliases - Phase 4
- ❌ Performance optimization - Phase 5
- ❌ New editor features - focus is validation only

## Risks & Mitigation

### Risk 1: Tests Reveal Major Bugs
**Likelihood:** Medium | **Impact:** High

Integration tests might reveal that undo/redo or registers don't work correctly with YAML types.

**Mitigation:**
- Fix bugs immediately as discovered
- Create minimal reproduction cases
- May extend timeline if major refactoring needed
- Document any deferred fixes for later phases

### Risk 2: Type Preservation Issues
**Likelihood:** Low | **Impact:** Medium

YamlString/YamlNumber preservation might be broken in some cases.

**Mitigation:**
- Test all type combinations exhaustively
- Check both unit tests and integration tests
- Review clone/copy implementations for types
- Add explicit type assertions in tests

### Risk 3: Visual Mode Edge Cases
**Likelihood:** Low | **Impact:** Low

Visual mode might have edge cases with YAML structures.

**Mitigation:**
- Test with various tree structures
- Test single node, multiple nodes, subtrees
- Check visual mode state transitions
- Manual testing to catch UI issues

## Next Steps

After design approval:
1. Use `superpowers:writing-plans` to create detailed implementation plan
2. Create `tests/editor_integration_tests.rs`
3. Implement tests incrementally (undo/redo, visual mode, registers, edge cases)
4. Fix any bugs discovered
5. Manual validation
6. Update CLAUDE.md and commit with tag

---

**Estimated Timeline:** 1-2 days
- Day 1: Create test file, implement undo/redo and visual mode tests
- Day 2: Register type preservation tests, edge cases, bug fixes, validation
