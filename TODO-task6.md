# TODO: Task 6 - Integration Testing and Documentation

**Status:** Not started - feature is functionally complete but needs final polish

## What's Left to Do

### 1. Integration Tests (`tests/comment_integration_tests.rs`)

Create comprehensive workflow tests:

- [ ] `test_full_workflow_add_edit_delete()` - Load YAML, add comment, edit comment, delete comment, verify state at each step
- [ ] `test_undo_redo_comments()` - Add comment → undo → redo → edit → undo → redo
- [ ] `test_yank_paste_comments()` - Yank comment node, paste elsewhere, verify copy
- [ ] `test_visual_mode_comments()` - Select range with comments, delete range, verify comments deleted
- [ ] `test_search_with_comments()` - Define behavior (skip comments or search content?)

### 2. Documentation Updates

#### Update `CLAUDE.md`:
```markdown
## Comment Support (v2.0)

YAMLQuill supports full comment editing:

- Comments are first-class navigable tree nodes
- Navigate to comments with j/k like any other line
- Add comments: press 'c' on a value node, currently defaults to Above position
- Edit comments: press 'e' on a comment node
- Delete comments: press 'dd' on a comment node
- Comments preserved on save with correct positioning

Supported comment positions:
- **Above**: Comment lines before a value
- **Line**: Inline comment after a value (same line)
- **Below**: Comment after children/end of block (basic implementation)
- **Standalone**: Comment between blank lines (basic implementation)

Limitations:
- Position prompt deferred - currently defaults to Above
- Object entry comments not yet supported
- Comment duplication in containers (Phase 4 limitation)
- Below/Standalone positioning uses basic implementation
```

#### Update `README.md`:
Add to keybindings section:
- `c` - Add comment (currently defaults to Above position)
- `e` - Edit comment (when cursor on comment)
- `dd` - Delete comment (when cursor on comment)

#### Create `docs/comment-editing-guide.md`:
Complete user guide with:
- Overview of comment support
- How to add comments
- How to edit comments
- How to delete comments
- Comment display styling
- Tips and limitations

### 3. Final Quality Checks

- [ ] Run full test suite: `cargo test`
- [ ] Run clippy: `cargo clippy -- -D warnings`
- [ ] Run fmt: `cargo fmt --check`
- [ ] Verify all 281+ tests pass
- [ ] Manual smoke test with examples/complex.yaml

### 4. Tagging

- [ ] Review all commits (should be 9-10)
- [ ] Tag: `git tag v2.0.0-comment-support`
- [ ] Consider: `git tag -a v2.0.0-comment-support -m "Comment support feature complete"`

## Current State

**Commits made (9 total):**
1. feat: add Comment data types to YamlValue
2. feat: extract comments during YAML parsing
3. fix: resolve critical comment extraction bugs (quote handling)
4. feat: display comments in tree view with proper styling
5. feat: add comment editing keybindings
6. fix: add missing Escape key handler for AwaitingComment state
7. feat: implement comment preservation in YAML save operation
8. fix: resolve key matching bugs in comment injection

**Tests passing:** 281 unit tests + 79 doctests = 360 total

**Known limitations documented:**
- Comment position prompt (defaults to Above)
- Object entry comments not supported
- Comment duplication in nested containers (requires Phase 4 text spans)
- Below/Standalone positioning basic implementation

## Time Estimate

Task 6 should take approximately 1-2 hours:
- Integration tests: 30-45 minutes
- Documentation updates: 30-45 minutes
- Quality checks and tagging: 15-30 minutes

## Reference

See original plan: `docs/plans/2026-02-01-comment-support-implementation.md` (Task 6, lines 1325-1509)
