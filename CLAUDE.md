# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**YAMLQuill** is a terminal-based structural YAML editor with vim-style keybindings, forked from [JSONQuill](https://github.com/joeygibson/jsonquill). The goal is to achieve full feature parity with JSONQuill while adding support for YAML-specific features including multi-document files, anchors/aliases, and multi-line strings.

**Status:** Phase 2e Complete - YAML-aware display with type indicators

## Development Workflow

This project uses **git worktrees** for isolated feature development. The main repository contains documentation and plans, while implementation happens in worktrees.

**Current worktree:**
- Location: `~/.config/superpowers/worktrees/yamlquill/initial-implementation`
- Branch: `feature/initial-implementation`
- Status: Phase 1 complete, tagged as `v0.1.0-phase1`

### Working in the Worktree

```bash
# Navigate to worktree
cd ~/.config/superpowers/worktrees/yamlquill/initial-implementation

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

## Pre-Commit Checklist

**CRITICAL: ALWAYS run these checks before committing. Never commit without passing all checks.**

```bash
cargo fmt && cargo clippy -- -D warnings && cargo test
```

All three must pass before creating a commit.

## Architecture

See `docs/plans/2026-01-31-yamlquill-design.md` for the complete design document.

### Core Concept

YAMLQuill is a structural fork of JSONQuill, replacing JSON parsing/serialization with YAML while preserving:
- Entire vim-style TUI editor architecture
- Theme system (all 15 themes)
- Input handling (vim keybindings, registers, marks, visual mode)
- Configuration system

### Key Design Decisions

- **Parser:** serde_yaml (v1), comment support deferred to v2
- **Multi-document:** Full support from start (similar to JSONL in JSONQuill)
- **Tree view:** Abstract representation matching JSONQuill (arrays as `[0]`, `[1]`)
- **Multi-line strings:** Shift+Enter inserts newline, Enter commits
- **Anchors/aliases:** Show as navigable reference nodes (`⟶ anchor_name`)

### Module Structure

```
src/
├── main.rs              # Entry point, terminal setup
├── lib.rs               # Library exports
├── document/            # YAML document representation
│   ├── parser.rs        # YAML → tree (using serde_yaml)
│   ├── node.rs          # YamlNode, YamlValue types
│   └── tree.rs          # Tree navigation, mutation
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

### Phase 2: YAML Document Model (In Progress)

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

**Next:**
- Phase 2f: Navigation Enhancements (jump commands, fold improvements)

### Phase 3: Multi-Document Support

**Goal:** Handle YAML files with multiple documents

**Tasks:**
- Add `YamlValue::MultiDoc` variant handling
- Parse multiple `---` separated documents
- Flat tree view rendering (like JSONL)
- Multi-doc save logic

### Phase 4: YAML-Specific Features

**Goal:** Anchors, aliases, and multi-line strings

**Tasks:**
- Anchor storage and display (`&name` badges)
- Alias nodes and navigation (`⟶ name`, jump on Enter)
- Multi-line string types (`|` and `>` preservation)
- Shift+Enter in INSERT mode for multi-line editing

### Phase 5: Polish & Parity

**Goal:** Complete feature parity with JSONQuill

**Tasks:**
- All 15 themes working correctly
- Help overlay updated for YAML
- Gzip support (.yaml.gz, .yml.gz)
- Format preservation for unmodified nodes
- Full test coverage (>80%)
- Documentation (README, user guide)

## Tech Stack

- **Language:** Rust 2021 edition
- **UI Framework:** ratatui 0.29 with termion 4.0 backend
- **YAML Parser:** serde_yaml 0.9
- **Data Structures:** indexmap 2.0 (ordered maps)
- **CLI:** clap 4.5
- **Config:** toml 0.8
- **Error Handling:** anyhow 1.0
- **Clipboard:** arboard 3.4
- **Compression:** flate2 1.0

## Key Features (Planned)

### v1.0 Features

- ✅ JSON→YAML conversion complete
- ✅ Basic YAML parsing and tree view
- ⏳ Multi-document YAML files
- ⏳ Anchors and aliases (navigate to anchor on Enter)
- ⏳ Multi-line strings with style preservation
- ⏳ All 15 themes from JSONQuill
- ⏳ Gzip compression support (.yaml.gz)
- ⏳ Format preservation for unmodified nodes

### v1.0 Known Limitations

- ❌ No comment support (serde_yaml limitation)
- ❌ Cannot create new anchors/aliases (preserve only)
- ❌ No tag editing (preserved but not editable)
- ❌ No advanced multi-line features (chomping, indentation indicators)

### v2.0+ Future Enhancements

- Comment support (requires custom parser)
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

**Current Status:** 44 tests passing (Phase 1 baseline)

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

### Finishing Work in a Worktree

When a phase is complete:

1. Ensure all tests pass
2. Tag the completion (e.g., `v0.1.0-phase1`)
3. Decide on integration strategy:
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
