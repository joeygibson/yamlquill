# YAMLQuill Phase 2: YAML Document Model - Design Document

**Date:** 2026-02-01
**Phase:** 2 of 5
**Status:** Design Complete, Ready for Implementation

## Overview

Phase 2 builds on Phase 1's baseline by validating and completing the YAML document model. The goal is to create a working editor for single-document YAML files with full editing capabilities, improved display, and solid test coverage.

## Goals

- Restore and update full test suite (fix 165 compilation errors)
- Implement complete YAML editing operations for scalar values
- Improve tree navigation and display with YAML-specific features
- Achieve 60-70% test coverage on core modules

## Overall Approach

**Core Strategy: Foundation-First Incremental Implementation**

We'll follow a bottom-up approach: fix the foundation (tree operations), implement core features (value editing), and validate with tests at each step.

### Three-Track Development

**Track 1: Core Tree Operations** (Week 1-2)
- Fix all tree.rs compilation errors and tests
- Implement missing tree mutation methods
- Validate navigation, path finding, node insertion/deletion
- Get tree.rs to 70%+ test coverage

**Track 2: Basic Value Editing** (Week 2-3)
- Implement scalar editing (strings, numbers, bools, null)
- Handle type conversions (string→number, etc.)
- Support YamlString variants (Plain/Literal/Folded)
- Fix editor state tests as we go

**Track 3: Display & Navigation** (Week 3-4)
- Improve tree_view.rs to show YAML types clearly
- Add YAML-specific visual indicators (quotes, `|`, `>`, int vs float)
- Enhance navigation (jump-to-parent, fold/unfold improvements)
- Fix UI tests and add new display tests

### Module-by-Module Test Fixing

- Start with tree.rs tests (foundation)
- Move to parser.rs and saver.rs tests (I/O validation)
- Then editor tests (registers, marks, undo/redo)
- Finally UI tests (tree_view, themes)

## Track 1: Core Tree Operations

### Phase 2a: Fix Tree Tests (Days 1-3)

Fix all compilation errors in tree.rs tests:
- `YamlValue::String(String)` → `YamlValue::String(YamlString::Plain(String))`
- `YamlValue::Number(f64)` → `YamlValue::Number(YamlNumber::Integer(i64))` or `YamlNumber::Float(f64)`
- Update test assertions to unwrap YamlString and YamlNumber enums

**Key test files to fix:**
- Tree navigation tests (get_node_at_path, find_parent, siblings)
- Tree mutation tests (insert_node, delete_node, replace_node)
- Path resolution tests (absolute paths, relative paths, edge cases)
- Expansion/collapse tests (expand_all, collapse_all, toggle)

### Phase 2b: Implement Missing Operations (Days 4-7)

**Navigation helpers:**
- `get_parent_path()` - Find parent of current node
- `get_sibling_paths()` - Get previous/next siblings
- `get_depth()` - Calculate nesting level for display

**Mutation operations:**
- `insert_value_at_path()` - Add new node at path
- `delete_at_path()` - Remove node and update indices
- `replace_value_at_path()` - Swap node with new value
- `move_node()` - Move subtree to different location (defer to Phase 3 if complex)

**YAML-specific helpers:**
- `can_convert_type()` - Check if type conversion is valid
- `convert_type()` - Convert between YAML types (string→number, etc.)
- Handle YamlString variant conversions (Plain↔Literal↔Folded)

**Validation:**
- All tree.rs tests pass
- Can navigate to any node in example YAML files
- Can insert/delete/replace nodes programmatically
- Tree mutations preserve IndexMap ordering

## Track 2: Basic Value Editing

### Phase 2c: Value Editing Infrastructure (Days 8-10)

**Edit prompt enhancements:**
- Update `edit_prompt.rs` to handle YamlString variants
- Add type indicator in prompt (e.g., "Edit string (plain):", "Edit integer:")
- Support multi-line input for Literal/Folded strings (Shift+Enter to add newline, Enter to commit)
- Validate input before committing (e.g., "42.5" is valid for number, "hello" is not)

**Type conversion UI:**
- Add `:convert` command (e.g., `:convert number`, `:convert string`)
- Show current type in status line
- Validate conversions (can't convert "hello" to number)
- Support YamlString style conversions (`:convert literal`, `:convert folded`)

**Scalar editing operations:**
- **Strings:** Edit content, preserve or change style (Plain/Literal/Folded)
- **Numbers:** Edit value, auto-detect Integer vs Float (has decimal → Float)
- **Booleans:** Toggle with `t` key, edit with `i` to type `true`/`false`
- **Null:** Show as `null`, allow changing to other types

### Phase 2d: Editor State Integration (Days 11-13)

**Registers (registers.rs):**
- Fix compilation errors (YamlString/YamlNumber)
- Test yank/paste with YAML nodes
- Ensure pasted nodes maintain type fidelity

**Undo/Redo:**
- Test undo after editing scalar values
- Test undo after type conversions
- Verify redo works correctly

**Visual mode:**
- Select and delete/yank subtrees
- Paste replaces selection

**Validation:**
- Can edit any scalar value in example YAML
- Type conversions work correctly
- Undo/redo preserves YAML types
- Registers store YAML nodes properly

## Track 3: Display & Navigation Improvements

### Phase 2e: YAML-Aware Display (Days 14-17)

**Type indicators:**
- **Strings:** Show quotes for Plain strings (`"hello"`), `|` prefix for Literal, `>` prefix for Folded
- **Numbers:** Show integers without decimals (`42`), floats with decimals (`42.0`)
- **Booleans:** Display as `true`/`false` (not `True`/False`)
- **Null:** Display as `null` (dimmed/grayed in theme)
- **Arrays:** Show length hint `[3 items]` when collapsed
- **Objects:** Show key count `{5 keys}` when collapsed

**Key-value formatting:**
- Object entries: `key: value` (colon-space like YAML)
- Array entries: `[0]: value` (keep JSONQuill's index style)
- Indent with 2 spaces (YAML convention)

**Multi-line string preview:**
- For Literal/Folded strings, show first line + `...` when collapsed
- When expanded, show all lines indented under the key
- Truncate very long lines with `...`

**Color coding:**
- Keys in one color (cyan/blue)
- String values in another (green)
- Numbers in another (yellow/magenta)
- Booleans/null in another (red/gray)
- Use existing theme system, just apply to YAML types

### Phase 2f: Navigation Enhancements (Days 18-20)

**Jump commands:**
- `gp` - Jump to parent node
- `[` - Jump to previous sibling
- `]` - Jump to next sibling
- `/` - Search for keys or values (already exists, verify works)

**Fold improvements:**
- `za` - Toggle fold at cursor (already exists)
- `zM` - Collapse all nodes
- `zR` - Expand all nodes
- `zc` - Collapse current subtree recursively

**Status line:**
- Show current path (e.g., `config.users[0].name`)
- Show node type (String/Number/Bool/Null/Array/Object)
- Show position (line X of Y)

**Validation:**
- Can distinguish YAML types visually
- Multi-line strings display correctly
- Navigation shortcuts work smoothly
- Status line provides useful context

## Test Coverage Strategy

**Goal:** Achieve 60-70% coverage with high-value tests that catch real bugs.

### Test Organization

**Unit Tests (in src/):**
- `tree.rs` - Navigation, mutation, path resolution (target 75%)
- `parser.rs` - YAML parsing, error handling (target 70%)
- `saver.rs` - Serialization, format preservation (target 65%)
- `node.rs` - YamlString/YamlNumber helpers (target 80%)
- `editor/registers.rs` - Yank/paste operations (target 60%)
- `editor/marks.rs` - Mark navigation (target 60%)

**Integration Tests (tests/):**
- `tests/basic_yaml.rs` - Parsing various YAML structures
- `tests/editing_operations.rs` - NEW: End-to-end editing scenarios
- `tests/type_conversion.rs` - NEW: Converting between YAML types
- `tests/tree_navigation.rs` - NEW: Navigation and display
- `tests/config_tests.rs` - Config loading (fix existing)
- `tests/theme_tests.rs` - Theme system (fix existing)

### High-Value Test Cases

**Tree Operations:**
- Navigate deeply nested structures (objects in arrays in objects)
- Insert/delete in middle of arrays (verify indices update)
- Empty arrays and objects (edge case)
- Single-element arrays (edge case)

**Value Editing:**
- Edit each scalar type (string, int, float, bool, null)
- Convert between types (string→number, number→string, etc.)
- Invalid conversions (should fail gracefully)
- Multi-line strings (Literal and Folded)
- Empty strings (edge case)

**Parser/Serializer:**
- Round-trip YAML (parse → edit → save → parse, should be identical)
- Preserve key ordering (IndexMap)
- Handle special characters in strings (quotes, colons, newlines)
- Large numbers (i64 boundary, f64 precision)

**Common Edge Cases:**
- Null values in arrays/objects
- Strings that look like numbers ("42", "true")
- Unicode strings
- Very long strings (>1000 chars)
- Deeply nested structures (>10 levels)

**Validation:**
- Run `cargo tarpaulin` or `cargo-llvm-cov` for coverage report
- All modified modules at 60%+ coverage
- Core modules (tree, parser, saver) at 65-75%
- Zero failing tests

## Implementation Workflow & Milestones

### Week 1: Foundation (Tree Operations)

**Days 1-3: Fix tree.rs tests**
- Fix all YamlString/YamlNumber compilation errors in tree tests
- Run `cargo test tree` until all pass
- Checkpoint: `cargo test tree` shows all green

**Days 4-7: Implement missing tree operations**
- Add navigation helpers (get_parent_path, get_sibling_paths, get_depth)
- Add mutation operations (insert_value_at_path, delete_at_path, replace_value_at_path)
- Add YAML-specific helpers (can_convert_type, convert_type)
- Checkpoint: Can programmatically navigate and mutate any example YAML file

### Week 2: Value Editing

**Days 8-10: Build editing infrastructure**
- Update edit_prompt.rs for YamlString variants
- Add type conversion UI (`:convert` command)
- Implement scalar editing for all types
- Checkpoint: Can manually edit strings and numbers through UI

**Days 11-13: Editor state integration**
- Fix registers.rs tests and implement YAML-aware yank/paste
- Fix undo/redo for YAML type changes
- Test visual mode with YAML nodes
- Checkpoint: `cargo test editor` passes, undo/redo works in manual testing

### Week 3: Display & Navigation

**Days 14-17: YAML-aware display**
- Add type indicators to tree_view.rs (quotes, `|`, `>`, int vs float)
- Improve key-value formatting
- Add multi-line string preview
- Apply color coding via theme system
- Checkpoint: example YAML files display clearly with visible type info

**Days 18-20: Navigation enhancements**
- Implement jump commands (gp, [, ])
- Improve fold commands (zM, zR, zc)
- Enhance status line (path, type, position)
- Checkpoint: Can navigate large YAML files efficiently

### Week 4: Testing & Polish

**Days 21-23: Comprehensive testing**
- Create new integration tests (editing_operations.rs, type_conversion.rs, tree_navigation.rs)
- Fix remaining test failures
- Run coverage analysis
- Checkpoint: 60%+ coverage, all tests green

**Days 24-25: Documentation & validation**
- Update CLAUDE.md with Phase 2 completion notes
- Create Phase 2 completion document
- Manual testing with diverse YAML files
- Checkpoint: Phase 2 complete, ready for Phase 3

### Daily Workflow

1. Morning: Run `cargo fmt && cargo clippy -- -D warnings && cargo test`
2. Implement feature/fix
3. Write/update tests
4. Evening: Verify all tests still pass, commit

## Success Criteria

Phase 2 is complete when:

### Functional Requirements
- ✅ Can load any single-document YAML file
- ✅ Can navigate entire tree structure with vim keybindings
- ✅ Can edit all scalar types (string, int, float, bool, null)
- ✅ Can convert between types (string→number, etc.)
- ✅ Can handle YamlString variants (Plain, Literal, Folded)
- ✅ Undo/redo works for all editing operations
- ✅ Registers (yank/paste) work with YAML nodes
- ✅ Visual mode selection and deletion works
- ✅ Can save edited YAML files correctly
- ✅ Tree display clearly shows YAML types
- ✅ Navigation shortcuts (gp, [, ], zM, zR) work

### Technical Requirements
- ✅ Zero compilation errors (`cargo build` succeeds)
- ✅ Zero clippy warnings (`cargo clippy -- -D warnings` passes)
- ✅ All tests pass (`cargo test` shows all green)
- ✅ 60-70% test coverage on core modules
- ✅ Code formatted (`cargo fmt`)

### Quality Requirements
- ✅ Round-trip YAML (load → edit → save → load preserves content)
- ✅ Key ordering preserved (IndexMap maintains insertion order)
- ✅ No data loss on save
- ✅ No crashes on valid YAML input
- ✅ Graceful error messages for invalid operations

## Deliverables

**Code:**
- Updated tree.rs with YAML-aware operations
- Updated parser.rs and saver.rs with validation
- Enhanced tree_view.rs with YAML type display
- Updated edit_prompt.rs for multi-line strings
- Fixed editor state modules (registers, marks, undo)

**Tests:**
- All existing tests fixed and passing
- New integration tests (editing_operations, type_conversion, tree_navigation)
- Coverage report showing 60-70%

**Documentation:**
- `docs/plans/2026-02-01-phase2-implementation.md` - Detailed implementation plan
- Updated CLAUDE.md with Phase 2 status
- Inline code comments for YAML-specific logic

**Git:**
- Clean commit history (feature commits, not "fix compilation" spam)
- Tag: `v0.2.0-phase2` on completion
- All work committed and pushed

## Out of Scope for Phase 2

- ❌ Multi-document YAML (Phase 3)
- ❌ Anchors and aliases (Phase 4)
- ❌ Advanced multi-line features (Phase 4)
- ❌ Structural operations (move nodes, copy subtrees) - deferred
- ❌ Comment support (v2+)

## Risks & Mitigation Strategies

### Risk 1: Test Fixing Takes Longer Than Expected
**Likelihood:** Medium | **Impact:** High

The 165 compilation errors might reveal deeper design issues requiring refactoring.

**Mitigation:**
- Start with a small module (tree.rs) to understand the pattern
- If major refactoring needed, reassess after fixing first module
- Time-box test fixing: if a module takes >2 days, document and move on
- Accept that some tests may be disabled temporarily

### Risk 2: YamlString Variants Complicate Everything
**Likelihood:** Medium | **Impact:** Medium

Plain/Literal/Folded variants might create awkward APIs everywhere.

**Mitigation:**
- Add helper methods like `as_str()`, `to_string()`, `is_multiline()`
- Create conversion functions for common operations
- Consider `impl Display for YamlString`
- If too painful, refactor to store style separately

### Risk 3: IndexMap Ordering Issues
**Likelihood:** Low | **Impact:** High

IndexMap might not preserve insertion order correctly.

**Mitigation:**
- Add explicit round-trip tests early
- Test with real-world YAML files
- Investigate serde_yaml serialization flags if needed
- Worst case: track original key order separately

### Risk 4: serde_yaml Limitations
**Likelihood:** Low | **Impact:** Medium

serde_yaml might not distinguish Literal vs Folded strings.

**Mitigation:**
- Accept that Phase 2 treats all multi-line strings as Plain
- Document limitation for Phase 4
- Phase 4 can use custom parser or events API
- Focus on correctness over format preservation in Phase 2

### Risk 5: UI/Display Changes Break Themes
**Likelihood:** Medium | **Impact:** Low

New color coding might not work with all 15 themes.

**Mitigation:**
- Reuse existing theme color slots
- Test with 2-3 diverse themes
- Note issues for Phase 5 polish
- Don't create new theme colors in Phase 2

### Risk 6: Scope Creep
**Likelihood:** High | **Impact:** Medium

Temptation to add features like multi-document or anchors.

**Mitigation:**
- Refer to "Out of Scope" list
- Create `docs/phase3-ideas.md` for future ideas
- Keep focused: tree operations, value editing, display only
- Time-box: if >4 weeks, cut scope

## Next Steps

After design approval:
1. Use `superpowers:using-git-worktrees` to create isolated workspace
2. Use `superpowers:writing-plans` to create detailed implementation plan
3. Begin Track 1: Fix tree.rs tests and implement tree operations
