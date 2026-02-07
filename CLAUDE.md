# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**YAMLQuill** is a terminal-based structural YAML editor with vim-style keybindings, forked from [JSONQuill](https://github.com/joeygibson/jsonquill). The goal is to achieve full feature parity with JSONQuill while adding support for YAML-specific features including multi-document files, anchors/aliases, and multi-line strings.

**Status:** Phase 5 Complete (v1.1) - Full feature parity with comment support, section-level format preservation, and anchor/alias display/navigation

## Development Workflow

This project uses **git worktrees** for isolated feature development. The main repository contains documentation and plans, while implementation happens in worktrees.

### Working in the Main Repository

```bash
# Navigate to repository
cd ~/Projects/yamlquill

# Build
cargo build

# Run tests
cargo test

# Run with example
cargo run -- examples/sample.yaml

# Format code
cargo fmt

# Lint
cargo clippy -- -D warnings
```

## Pre-Commit Requirements

**CRITICAL: These checks are MANDATORY before ANY commit. Never offer to commit without running all checks first.**

### Required Checks (in order)

1. **Format verification:**
   ```bash
   cargo fmt --check
   ```
   If fails: Run `cargo fmt` to fix, then verify again.

2. **Clippy linting:**
   ```bash
   cargo clippy -- -D warnings
   ```
   Must pass with zero warnings. Fix all issues before proceeding.

3. **Test suite:**
   ```bash
   cargo test
   ```
   All tests must pass (currently ~437 tests expected).

4. **Documentation updates:**
   - Update CLAUDE.md if implementation changes project status
   - Update relevant plan documents if approach changes
   - Add research findings if investigation was performed

### Quick Check Command

```bash
cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

**Process:** Run all checks → Fix any failures → Verify again → THEN offer to commit.

Never skip these steps. Never assume code is clean without verification.

## Architecture

See `docs/plans/2026-01-31-yamlquill-design.md` for the complete design document.

### Core Concept

YAMLQuill is a structural fork of JSONQuill, replacing JSON parsing/serialization with YAML while preserving:
- Entire vim-style TUI editor architecture
- Theme system (all 15 themes)
- Input handling (vim keybindings, registers, marks, visual mode)
- Configuration system

### Key Design Decisions

- **Parser:** yaml-rust2 Parser/EventReceiver for anchor/alias support, serde_yaml for serialization
- **Multi-document:** Full support (similar to JSONL in JSONQuill)
- **Tree view:** Abstract representation matching JSONQuill (arrays as `[0]`, `[1]`)
- **Multi-line strings:** Multi-line string type preservation (Literal `|` and Folded `>`)
- **Anchors/aliases:** Display anchors with `&name` badges, aliases as `*name` nodes with read-only enforcement

### Module Structure

```
src/
├── main.rs              # Entry point, terminal setup
├── lib.rs               # Library exports
├── document/            # YAML document representation
│   ├── parser.rs        # YAML → tree (using yaml-rust2 Parser/EventReceiver)
│   ├── node.rs          # YamlNode, YamlValue types
│   └── tree.rs          # Tree navigation, mutation, AnchorRegistry
├── editor/              # Editor state (modes, cursor, undo/redo)
│   ├── registers.rs     # Named registers a-z, 0-9
│   ├── marks.rs
│   └── jumplist.rs
├── input/               # Input handling (vim keybindings)
├── ui/                  # Terminal UI rendering
│   ├── tree_view.rs     # YAML tree display
│   ├── edit_prompt.rs   # Multi-line editing support
│   └── theme_picker.rs  # Theme selection UI
├── theme/               # Color themes (all 15 themes)
├── file/                # File I/O (YAML + gzip support)
├── yamlpath/            # YAMLPath/JSONPath queries
└── config/              # Configuration system
```

### Data Structures

**Core types (`src/document/node.rs`):**

```rust
pub struct YamlNode {
    pub value: YamlValue,
    pub expanded: bool,
    pub anchor: Option<String>,        // &anchor_name
    pub original_formatting: Option<String>, // Format preservation
}

pub enum YamlValue {
    Null,
    Bool(bool),
    Number(YamlNumber),               // Integer or Float
    String(YamlString),               // Plain, Literal, or Folded
    Array(Vec<YamlNode>),
    Object(IndexMap<String, YamlNode>), // Ordered key-value pairs
    Alias(String),                     // *alias_name reference
    MultiDoc(Vec<YamlNode>),           // Multiple documents
}

pub enum YamlString {
    Plain(String),                     // Regular string
    Literal(String),                   // | style, preserves newlines
    Folded(String),                    // > style, folds newlines
}

pub enum YamlNumber {
    Integer(i64),
    Float(f64),
}
```

## Implementation Phases

### Phase 1: Core Structure ✅ COMPLETE

**Status:** Tagged as `v0.1.0-phase1` in worktree

**Completed:**
- ✅ Copied JSONQuill codebase
- ✅ Renamed json→yaml throughout
- ✅ Updated dependencies (serde_yaml, indexmap)
- ✅ Implemented basic YAML parser
- ✅ Added YAML-specific data structures
- ✅ Basic file I/O with gzip support
- ✅ Compiling and runnable baseline
- ✅ 44 tests passing

**See:** `docs/plans/2026-01-31-phase1-core-structure.md`

### Phase 2: YAML Document Model ✅ COMPLETE

**Goal:** Full editing operations for single-document YAML

**Completed:**
- ✅ Phase 2a: Core structure and compilation fixes (checkpoint: phase2-checkpoint1)
- ✅ Phase 2b: Test suite fixes (checkpoint: phase2-checkpoint2)
- ✅ Phase 2c: Value editing bug fixes (checkpoint: phase2c-complete)
  - Fixed CRITICAL bug: string style preservation (Literal/Folded)
  - Added input validation to prevent data corruption
  - Added 8 comprehensive editing tests
  - All 315 tests passing
- ✅ Phase 2d: Editor State Integration (checkpoint: phase2d-complete)
  - Added 19 integration tests for undo/redo, registers, visual mode
  - Validated undo/redo after editing all scalar types
  - Validated YamlString style preservation (Plain/Literal/Folded)
  - Validated register type preservation (Integer vs Float, etc.)
  - All 334 tests passing
- ✅ Phase 2e: YAML-Aware Display (checkpoint: phase2e-complete)
  - Added type indicators for YamlString styles (Plain: "text", Literal: | text, Folded: > text)
  - Distinguished Integer vs Float display (42 vs 3.14)
  - Added 16 display tests validating type indicators
  - All 350 tests passing
- ✅ Phase 2f: Navigation Enhancements (checkpoint: phase2f-complete)
  - Added vim-style navigation shortcuts ([ ], gp for siblings/parent)
  - Added fold commands (za, zM, zR, zc)
  - Added 10 navigation tests
  - All 360 tests passing

**Phase 2 COMPLETE!**

### Phase 3: Multi-Document Support ✅ COMPLETE

**Goal:** Handle YAML files with multiple documents

**Completed:**
- ✅ Added multi-document parsing using `serde_yaml::Deserializer`
- ✅ Single documents remain unwrapped, multiple docs use `YamlValue::MultiDoc`
- ✅ Updated saver to output proper YAML multi-document format with `---` separators
- ✅ Added 8 comprehensive multi-document tests including round-trip validation
- ✅ All 201 tests passing

**Phase 3 COMPLETE!**

### Phase 4: YAML-Specific Features

**Phase 4a: Parser Investigation ✅ COMPLETE**
- ✅ Researched yaml-rust2 capabilities (Scanner, Parser, YamlLoader)
- ✅ Documented YamlLoader alias resolution blocker
- ✅ Validated Parser/EventReceiver approach preserves anchors and aliases

**Phase 4b: Anchor/Alias Implementation ✅ COMPLETE**
- ✅ Implemented Parser/EventReceiver-based tree builder
- ✅ Added Scanner for extracting anchor/alias names
- ✅ Created AnchorRegistry for tracking anchor definitions and alias references
- ✅ Automatic registry population when creating YamlTree
- ✅ Display anchors with `&name` badges in tree view
- ✅ Display aliases as `*name` nodes (already supported by existing code)
- ✅ All 390 tests passing (including 13 anchor/alias tests)

**Phase 4c: Editing Constraints ✅ COMPLETE**
- ✅ Read-only alias enforcement (cannot edit alias values directly)
- ✅ Anchor delete protection (prevents deleting anchors with active aliases)
- ✅ Registry updates on node deletion to maintain synchronization

**Phase 4d: Anchor Navigation ✅ COMPLETE**
- ✅ Jump to anchor definition with Enter key on alias nodes
- ✅ Integrated with jump list (Ctrl-o to jump back)
- ✅ User-friendly messages for successful navigation and errors
- ✅ Added 2 navigation tests

**Deferred to v2.0:**
- ❌ Create/edit anchors and aliases via UI

**Already implemented (Phase 2):**
- ✅ Multi-line string type preservation (Literal `|` and Folded `>` styles preserved)
- ✅ Type-aware display showing string styles

### Phase 5: Polish & Parity ✅ COMPLETE

**Goal:** Complete feature parity with JSONQuill v1.0

**Completed:**
- ✅ All 15 themes working correctly (verified in tests)
- ✅ Help overlay updated for YAML (shows "YAMLQuill Help")
- ✅ Gzip support (.yaml.gz, .yml.gz) working
- ✅ Format preservation for unmodified nodes implemented
- ✅ Section-level format preservation for structural edits (medium path)
- ✅ Anchor/alias preservation on save via line-level patching
- ✅ Comment support: parse, display, add, and save comments
- ✅ Test coverage: ~358 unit/integration + 79 doctests = ~437 total
- ✅ Documentation updated (README.md reflects current state)

**v1.0 Feature Parity Achieved! v1.1 adds comment support and format preservation improvements.**

## Tech Stack

- **Language:** Rust 2021 edition
- **UI Framework:** ratatui 0.29 with termion 4.0 backend
- **YAML Parser:** yaml-rust2 0.10 (Parser/EventReceiver for parsing, Scanner for tokenization)
- **YAML Serialization:** serde_yaml 0.9 (for saving files)
- **Data Structures:** indexmap 2.0 (ordered maps)
- **CLI:** clap 4.5
- **Config:** toml 0.8
- **Error Handling:** anyhow 1.0
- **Clipboard:** arboard 3.4
- **Compression:** flate2 1.0

## Key Features

### v1.0 Features ✅ COMPLETE

- ✅ JSON→YAML conversion complete
- ✅ Full YAML parsing and tree view
- ✅ Multi-document YAML files
- ✅ Anchors and aliases (display with `&name` badges and `*name` nodes)
- ✅ Anchor navigation (press Enter on `*alias` to jump to `&anchor` definition)
- ✅ Multi-line strings with style preservation (Literal `|` and Folded `>`)
- ✅ All 15 themes from JSONQuill
- ✅ Gzip compression support (.yaml.gz)
- ✅ Format preservation for unmodified nodes
- ✅ Section-level format preservation for structural edits
- ✅ Anchor/alias preservation on save via line-level patching
- ✅ Comment support (parse, display, add, save)
- ✅ Anchor delete protection and read-only alias enforcement

### v1.1 Known Limitations

- Anchors/aliases are preserved for value edits, key additions, and comment additions via line-level patching; only complex structural changes (e.g., deleting keys, adding array items) in anchor sections fall back to serde_yaml which resolves aliases to inline values
- Cannot create new anchors/aliases via UI (display, navigate, and enforce only)
- No tag editing (preserved but not editable)
- No advanced multi-line features (chomping, indentation indicators)

### v2.0+ Future Enhancements

- Create/edit anchors and aliases via UI
- YAML tag editing
- Advanced multi-line string controls

## Testing

### Running Tests

```bash
# All tests
cargo test

# Specific test file
cargo test --test basic_yaml

# With output
cargo test -- --nocapture

# Single test
cargo test test_parse_simple_yaml
```

### Test Organization

- `tests/basic_yaml.rs` - YAML parsing validation
- `tests/config_tests.rs` - Configuration system
- `tests/input_tests.rs` - Key mapping
- `tests/jumplist_tests.rs` - Navigation history
- `tests/marks_tests.rs` - Mark system
- `tests/theme_tests.rs` - Theme system
- `tests/anchor_alias_extraction_tests.rs` - Anchor/alias parsing and editing constraints
- `tests/multi_document_tests.rs` - Multi-document YAML support
- `tests/navigation_tests.rs` - Tree navigation
- `tests/yaml_display_tests.rs` - Type-aware display
- `tests/yaml_editing_tests.rs` - Value editing operations

**Current Status:** ~437 tests passing (~358 unit/integration + 79 doctests)

### Test Coverage Goals

- **Phase 2:** >60% coverage on core functionality
- **Phase 5:** >80% coverage on business logic

## Common Development Tasks

### Building and Running

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run with example file
cargo run -- examples/sample.yaml

# Run with custom file
cargo run -- /path/to/your/file.yaml

# Run with stdin
cat file.yaml | cargo run

# Run with gzip file
cargo run -- data.yaml.gz
```

### Code Quality

```bash
# Format code
cargo fmt

# Check formatting without changing
cargo fmt --check

# Run clippy linter
cargo clippy

# Clippy with warnings as errors (pre-commit requirement)
cargo clippy -- -D warnings

# Check without building
cargo check
```

### Version Management

**Note:** Version management will be added in Phase 5

## Configuration

YAMLQuill will support configuration at `~/.config/yamlquill/config.toml` (not yet implemented in Phase 1).

**Planned config options:**
- Theme selection
- Line numbers (absolute/relative)
- Mouse support
- Backup file creation
- Undo limit
- Format preservation settings

## Documentation

### Design Documents

- `docs/plans/2026-01-31-yamlquill-design.md` - Complete design specification
- `docs/plans/2026-01-31-phase1-core-structure.md` - Phase 1 implementation plan

### User Documentation

**Phase 5 will include:**
- README.md with feature list and keybindings
- User guide
- Architecture documentation
- Contributing guide

## Worktree Workflow

### Creating a New Worktree

```bash
# For new features
git worktree add ~/.config/superpowers/worktrees/yamlquill/feature-name -b feature/feature-name

# Navigate to worktree
cd ~/.config/superpowers/worktrees/yamlquill/feature-name
```

### Commit Workflow

**MANDATORY PROCESS - Never deviate from this sequence:**

1. **Verify code quality:**
   ```bash
   cargo fmt --check && cargo clippy -- -D warnings && cargo test
   ```

2. **Update documentation** (if needed):
   - CLAUDE.md (project status, phase completion)
   - Plan documents (implementation details, decisions)
   - Research findings (investigation results)

3. **Stage changes:**
   ```bash
   git add <files>
   ```

4. **Verify what's staged:**
   ```bash
   git status
   git diff --cached
   ```

5. **Commit with descriptive message:**
   ```bash
   git commit -m "type: description

   - Detail 1
   - Detail 2

   Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
   ```

**Never skip step 1.** Never offer to commit without completing all verification steps first.

### Finishing Work in a Worktree

When a phase is complete:

1. Run pre-commit checks (see Commit Workflow above)
2. Commit any remaining changes
3. Tag the completion (e.g., `v0.1.0-phase1`)
4. Decide on integration strategy:
   - Merge to main
   - Create pull request
   - Continue development in worktree

Use `@superpowers:finishing-a-development-branch` skill for guided cleanup.

### Removing a Worktree

```bash
# List all worktrees
git worktree list

# Remove a worktree
git worktree remove ~/.config/superpowers/worktrees/yamlquill/feature-name

# Prune stale worktree references
git worktree prune
```

## Troubleshooting

### Build Issues

**Problem:** `cargo build` fails with dependency errors

**Solution:** Update dependencies
```bash
cargo update
cargo clean
cargo build
```

**Problem:** Clippy warnings fail the build

**Solution:** Fix warnings or add `#[allow()]` for intentional cases
```bash
cargo clippy --fix
```

### Test Issues

**Problem:** Tests fail after making changes

**Solution:** Update tests to match new API
- Check for YamlString vs String mismatches
- Verify YamlNumber vs f64 changes
- Update assertions for new data structures

### Runtime Issues

**Problem:** Editor doesn't start

**Solution:** Verify you have a controlling terminal
```bash
# Won't work in non-interactive contexts
echo '{}' | yamlquill  # ❌ Fails

# Works in terminal
yamlquill file.yaml  # ✅ Success
```

## Links

- **JSONQuill (parent project):** https://github.com/joeygibson/jsonquill
- **YAMLQuill repository:** https://github.com/joeygibson/yamlquill (when published)
- **Design document:** `docs/plans/2026-01-31-yamlquill-design.md`

## License

MIT License - see LICENSE.md file for details

---

**For detailed implementation instructions, see the phase-specific plans in `docs/plans/`**
