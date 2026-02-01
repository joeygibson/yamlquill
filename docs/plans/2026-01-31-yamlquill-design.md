# YAMLQuill Design Document

**Date:** 2026-01-31
**Status:** Design Complete, Ready for Implementation

## Overview

YAMLQuill is a terminal-based structural YAML editor with vim-style keybindings, based on the architecture of JSONQuill. The goal is to achieve full feature parity with JSONQuill while adding support for YAML-specific features including multi-document files, anchors/aliases, and multi-line strings.

## Core Concept

YAMLQuill will be a structural fork of JSONQuill, replacing JSON parsing/serialization with YAML while preserving the entire vim-style TUI editor architecture, theme system, and user experience.

## Design Decisions

### Scope
- **Full feature parity** with JSONQuill's extensive feature set
- **Full YAML feature support** including anchors, aliases, multi-line strings, and multi-document files
- **Comment support deferred** to v2 (limitation of serde_yaml parser)

### Technical Choices
- **Parser:** serde_yaml (pragmatic choice for v1, accepts comment limitation)
- **Multi-document:** Full support from start (similar to JSONL in JSONQuill)
- **Tree view display:** Abstract representation matching JSONQuill (arrays as `[0]`, `[1]`, etc.)
- **Multi-line strings:** Shift+Enter inserts newline, Enter commits edit
- **Anchors/aliases:** Show as navigable reference nodes (`⟶ anchor_name`)

### Reusable Components from JSONQuill

**Complete reuse (minimal changes):**
- Terminal UI framework (ratatui + termion)
- Editor state management (modes, cursor, undo/redo)
- Input handling (vim keybindings, count prefixes)
- Registers system (named registers a-z, A-Z, numbered 0-9)
- Marks and jump list
- Theme system (all 15 themes)
- Theme picker UI and tab completion
- Configuration system (config file, `:set` commands)
- Visual mode
- Search functionality
- Help overlay system

**YAML-specific replacements:**
- Document parsing (serde_json → serde_yaml)
- Serialization (YAML output formatting)
- Tree representation (add YAML-specific node types)
- File I/O (adapt for YAML)

## Architecture

### Module Structure

```
src/
├── main.rs              # Entry point, terminal setup
├── lib.rs               # Library exports
├── document/            # YAML document representation
│   ├── mod.rs
│   ├── parser.rs        # YAML → tree (using serde_yaml)
│   ├── node.rs          # YamlNode, YamlValue types
│   └── tree.rs          # Tree navigation, mutation
├── editor/              # Editor state (REUSE from jsonquill)
│   ├── mod.rs
│   ├── cursor.rs
│   ├── modes.rs
│   ├── undo.rs
│   ├── registers.rs     # Named registers a-z, 0-9
│   ├── marks.rs
│   └── jumplist.rs
├── input/               # Input handling (REUSE from jsonquill)
│   ├── mod.rs
│   ├── handler.rs
│   └── keys.rs
├── ui/                  # Terminal UI (REUSE from jsonquill)
│   ├── mod.rs
│   ├── tree_view.rs     # Minor tweaks for YAML display
│   ├── status_line.rs
│   ├── help_overlay.rs  # Update keybinding docs
│   ├── message_area.rs
│   ├── edit_prompt.rs   # Add Shift+Enter for newlines
│   ├── layout.rs
│   └── theme_picker.rs  # Theme selection UI
├── theme/               # Theme system (REUSE from jsonquill)
│   ├── mod.rs           # All 15 themes
│   └── colors.rs
├── file/                # File I/O (ADAPT for YAML)
│   ├── mod.rs
│   ├── loader.rs        # YAML loading, gzip support
│   └── saver.rs         # YAML saving, format preservation
├── yamlpath/            # YAMLPath/JSONPath queries (ADAPT)
│   ├── mod.rs
│   ├── parser.rs
│   ├── evaluator.rs
│   ├── ast.rs
│   └── error.rs
└── config/              # Configuration (REUSE from jsonquill)
    └── mod.rs
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
    Number(YamlNumber),               // Handle integers vs floats
    String(YamlString),               // Track literal/folded style
    Array(Vec<YamlNode>),
    Object(IndexMap<String, YamlNode>), // Preserve key order
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

**Design points:**
- **Anchors:** Stored on any `YamlNode`, displayed as badge in tree view
- **Aliases:** Special `YamlValue::Alias(name)` variant, renders as `⟶ anchor_name`
- **Multi-line strings:** `YamlString` enum tracks literal vs folded style
- **Multi-doc:** `YamlValue::MultiDoc` is the root type (like `JsonlRoot`)
- **Format preservation:** `original_formatting` preserves YAML text for unmodified nodes

## Key Workflows

### Multi-Document Handling

**On load:**
- Detect multiple `---` separated documents
- Parse into `YamlValue::MultiDoc(vec![doc1, doc2, doc3])`
- Display flat list with collapsed previews
- No root container (like JSONL in JSONQuill)

**Tree view example:**
```
--- Document 1  (3) {name: "prod", host: "prod.example.com", ...}
--- Document 2  (2) {name: "dev", host: "dev.example.com"}
--- Document 3  (5) {users: [...], config: {...}}
```

**Navigation:**
- `l` / `→` to expand a document
- `h` / `←` to collapse back to preview
- All editing works within expanded documents

### Anchor and Alias Workflow

**Display:**
- Anchored node: `defaults &default_settings { timeout: 30, retry: 3 }`
- Alias reference: `⟶ default_settings`

**Navigation:**
- Cursor on alias → press `Enter` or `l` → jumps to anchor definition
- Jump recorded in jump list (use `Ctrl-o` to return)

**Editing:**
- Edit anchored node → changes preserved with anchor
- Delete anchored node → aliases show as broken: `⟶ default_settings [broken]` (red)
- **V1 limitation:** Cannot create new anchors/aliases via UI (preserve existing only)

### Multi-line String Editing

**INSERT mode behavior:**
- Regular editing: type, press `Enter` to commit (same as JSONQuill)
- Multi-line: type, press `Shift+Enter` for newline, continue editing, `Enter` to commit
- Preserve original style:
  - If original was `|` (literal) → keep literal style
  - If original was `>` (folded) → keep folded style
  - If creating new multi-line string → default to `|` (literal)

**Visual indicator:**
- Multi-line strings show line count in preview: `description: (3 lines)`

### Tree View Display

**Display conventions (abstract representation like JSONQuill):**
- Array elements: `[0]`, `[1]`, `[2]` (even though YAML uses `-`)
- Object keys: `key:` (matches YAML syntax)
- Collapsed previews: `{name: "Alice", age: 30}`
- Alias nodes: `⟶ default_settings` (special indicator)
- Anchored nodes: Show `&name` badge next to key

## Testing Strategy

### Unit Tests
- `document/parser.rs` - YAML parsing, multi-doc detection, anchor/alias resolution
- `document/tree.rs` - Tree mutations, navigation with aliases
- `file/saver.rs` - YAML serialization, format preservation, multi-line string styles
- `yamlpath/` - Query parsing and evaluation

### Integration Tests
- Load/edit/save round-trips with format preservation
- Multi-document file handling
- Anchor/alias preservation
- Multi-line string style preservation
- Gzip compression (.yaml.gz)

### Test Data
- Mirror structure from `jsonquill/tests/` and `jsonquill/examples/`
- Create `examples/sample.yaml`, `examples/multi-doc.yaml`, `examples/anchors.yaml`
- Create `tests/` with YAML-specific test cases

## Implementation Phases

### Phase 1: Core Structure
**Goal:** Copy and establish baseline

- Copy entire jsonquill codebase to yamlquill directory
- Rename crate in Cargo.toml (`jsonquill` → `yamlquill`)
- Update dependencies: `serde_json` → `serde_yaml`
- Global find/replace: `json` → `yaml`, `Json` → `Yaml`, `JSON` → `YAML`
- Verify compilation (expect errors, establish baseline)

**Deliverable:** Compiling codebase with basic YAML structure

### Phase 2: YAML Document Model
**Goal:** Basic single-document YAML editing

- Implement `YamlNode`, `YamlValue`, `YamlString` enums
- YAML parser using serde_yaml
- Basic tree representation (no anchors/multi-doc yet)
- Simple load/save for single-document YAML files
- Basic editing operations (add, edit, delete values)

**Deliverable:** Working editor for simple single-document YAML files

### Phase 3: Multi-Document Support
**Goal:** Handle YAML files with multiple documents

- Add `YamlValue::MultiDoc` variant
- Parse multiple `---` separated documents
- Flat tree view rendering (like JSONL)
- Multi-doc save logic (preserve `---` separators)

**Deliverable:** Full multi-document YAML file support

### Phase 4: YAML-Specific Features
**Goal:** Anchors, aliases, and multi-line strings

- Anchor storage and display (`&name` badges)
- Alias nodes and navigation (`⟶ name`, jump on Enter)
- Multi-line string types (`|` and `>` preservation)
- Shift+Enter in INSERT mode for multi-line editing
- Broken alias detection and display

**Deliverable:** Full YAML feature support (except comments)

### Phase 5: Polish & Parity
**Goal:** Complete feature parity with JSONQuill

- All 15 themes working correctly
- Theme picker UI with tab completion
- Help overlay updated for YAML keybindings
- Gzip support (.yaml.gz, .yml.gz)
- Format preservation for unmodified nodes
- Full test coverage (>80%)
- Documentation updates (README, CLAUDE.md)

**Deliverable:** Production-ready YAMLQuill v1.0

## Future Enhancements (v2+)

### Comment Support
- Requires custom YAML parser or serde_yaml fork
- Preserve comments in tree structure
- Allow editing/adding comments
- Display comments in tree view

### Advanced Anchor/Alias Features
- Create new anchors via UI (`:anchor` command or keybinding)
- Create new aliases pointing to existing anchors
- Rename anchors (update all aliases automatically)
- Show all aliases for an anchor

### YAML Tags
- Display explicit tags (`!!str`, `!!int`, etc.)
- Allow editing/changing tags
- Custom tag support

### Advanced Multi-line Editing
- Block chomping indicators (`|+`, `|-`, etc.)
- Block indentation indicators (`|2`, `>4`, etc.)
- Full-screen editor mode for long multi-line strings

## Success Criteria

**v1.0 is successful when:**
1. ✅ Can load, edit, and save YAML files with format preservation
2. ✅ All JSONQuill features work with YAML (undo/redo, registers, visual mode, marks, search)
3. ✅ Multi-document YAML files are fully supported
4. ✅ Anchors and aliases are preserved and navigable
5. ✅ Multi-line strings (literal and folded) work correctly
6. ✅ All 15 themes and theme picker work
7. ✅ Test coverage >80% on core functionality
8. ✅ No regressions from JSONQuill's stability and usability

## Known Limitations (v1.0)

- ❌ **No comment support** (serde_yaml limitation)
- ❌ **Cannot create new anchors/aliases** (preserve only)
- ❌ **No tag editing** (preserved but not editable)
- ❌ **No advanced multi-line features** (chomping, indentation indicators)

These limitations are acceptable for v1.0 and will be addressed in future versions.
