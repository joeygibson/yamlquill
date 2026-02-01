# YAMLQuill Phase 1: Core Structure Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Copy JSONQuill codebase, rename to yamlquill, update dependencies, and establish a compiling baseline.

**Architecture:** Fork JSONQuill's entire codebase structure, perform systematic renaming (json→yaml), swap serde_json for serde_yaml, and verify basic compilation.

**Tech Stack:** Rust 2021, serde_yaml, ratatui, termion, clap

---

## Task 1: Copy JSONQuill Codebase

**Files:**
- Copy: `/Users/jgibson/Projects/jsonquill/*` → `/Users/jgibson/.config/superpowers/worktrees/yamlquill/initial-implementation/`

**Step 1: Copy entire jsonquill directory structure**

```bash
# Copy all files except .git directory
rsync -av --exclude='.git' --exclude='target' --exclude='.claude' \
  /Users/jgibson/Projects/jsonquill/ \
  /Users/jgibson/.config/superpowers/worktrees/yamlquill/initial-implementation/
```

Expected: All source files, Cargo.toml, examples/, tests/, scripts/ copied

**Step 2: Verify copied structure**

```bash
ls -la
```

Expected: See Cargo.toml, src/, tests/, examples/, scripts/, docs/

**Step 3: Remove jsonquill-specific files**

```bash
rm -f CHANGELOG.md test_features.json
```

Expected: Clean slate for yamlquill-specific files

**Step 4: Verify git status**

```bash
git status
```

Expected: Many untracked files (the copied source)

---

## Task 2: Update Cargo.toml Package Metadata

**Files:**
- Modify: `Cargo.toml`

**Step 1: Update package name and description**

Open `Cargo.toml` and change:

```toml
[package]
name = "yamlquill"
version = "0.1.0"
edition = "2021"
license = "MIT"
authors = ["Joey Gibson"]
description = "YAMLQuill - A terminal-based structural YAML editor with vim-style keybindings"
repository = "https://github.com/joeygibson/yamlquill"
readme = "README.md"
keywords = ["yaml", "editor", "tui", "vim", "terminal"]
categories = ["command-line-utilities", "text-editors"]
```

**Step 2: Update dependencies (replace serde_json with serde_yaml)**

```toml
[dependencies]
ratatui = { version = "0.29", default-features = false, features = ["termion"] }
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
indexmap = { version = "2.0", features = ["serde"] }
termion = "4.0"
clap = { version = "4.5", features = ["derive"] }
toml = "0.8"
anyhow = "1.0"
arboard = "3.4"
dirs = "5.0"
flate2 = "1.0"

[dev-dependencies]
tempfile = "3.13"
```

**Step 3: Verify Cargo.toml syntax**

```bash
cargo metadata --format-version 1 > /dev/null
```

Expected: No errors, valid Cargo.toml

**Step 4: Commit**

```bash
git add Cargo.toml
git commit -m "chore: update Cargo.toml for yamlquill

- Rename package from jsonquill to yamlquill
- Replace serde_json with serde_yaml
- Add indexmap dependency for ordered maps
- Reset version to 0.1.0

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 3: Rename Source Files (json → yaml)

**Files:**
- Rename: `src/document/` files
- Rename: `src/jsonpath/` → `src/yamlpath/`

**Step 1: Rename jsonpath directory to yamlpath**

```bash
git mv src/jsonpath src/yamlpath
```

Expected: Directory renamed

**Step 2: Commit directory rename**

```bash
git commit -m "refactor: rename jsonpath module to yamlpath

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 4: Global Find/Replace (json → yaml)

**Files:**
- Modify: All `.rs` files in `src/`

**Step 1: Find and replace json → yaml (case-sensitive)**

This is a manual step due to the large number of files. Use your editor's find/replace or:

```bash
# Preview what would be changed (dry run)
find src -name "*.rs" -exec grep -l "json" {} \;
```

**Key replacements needed:**
- `json` → `yaml`
- `Json` → `Yaml`
- `JSON` → `YAML`
- `Jsonl` → `YamlMultiDoc` (conceptual rename)
- `JsonlRoot` → `MultiDoc`

**Important exceptions (DO NOT replace):**
- `serde_json` references in comments/docs
- JSON format references in help text

**Step 2: Systematic file-by-file replacement**

Starting with core types:

**File: `src/document/node.rs`**

Replace all occurrences:
- `JsonNode` → `YamlNode`
- `JsonValue` → `YamlValue`
- `json_node` → `yaml_node`
- `json_value` → `yaml_value`

**File: `src/document/parser.rs`**

Replace:
- `parse_json` → `parse_yaml`
- `JsonParser` → `YamlParser`
- `serde_json::` → `serde_yaml::`

**File: `src/document/tree.rs`**

Replace:
- `JsonTree` → `YamlTree`
- `json_tree` → `yaml_tree`

**File: `src/document/mod.rs`**

Update module exports to use Yaml* names

**Continue this pattern for all files in:**
- `src/editor/`
- `src/file/`
- `src/ui/`
- `src/input/`
- `src/yamlpath/`
- `src/config/`
- `src/theme/`
- `src/main.rs`
- `src/lib.rs`

**Step 3: Update binary name in main.rs**

In `src/main.rs`, update help text and app name:

```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "yamlquill")]
#[command(about = "A terminal-based structural YAML editor", long_about = None)]
struct Args {
    /// YAML file to open
    #[arg(value_name = "FILE")]
    file: Option<String>,

    // ... rest of args
}
```

**Step 4: Attempt compilation to find missed renames**

```bash
cargo check 2>&1 | head -50
```

Expected: Many errors, but they'll point to missed renames

**Step 5: Fix compilation errors iteratively**

Go through each error and fix remaining json→yaml renames

**Step 6: Commit all renames**

```bash
git add src/
git commit -m "refactor: global rename json → yaml throughout codebase

- JsonNode → YamlNode
- JsonValue → YamlValue
- JsonTree → YamlTree
- json → yaml in all variable names
- Update binary name to yamlquill
- Update help text for YAML

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 5: Update Data Structures for YAML

**Files:**
- Modify: `src/document/node.rs`

**Step 1: Define YamlString enum**

Add before `YamlValue`:

```rust
/// Represents different YAML string styles
#[derive(Debug, Clone, PartialEq)]
pub enum YamlString {
    /// Regular string (no special formatting)
    Plain(String),
    /// Literal block scalar (| style, preserves newlines)
    Literal(String),
    /// Folded block scalar (> style, folds newlines)
    Folded(String),
}

impl YamlString {
    pub fn as_str(&self) -> &str {
        match self {
            YamlString::Plain(s) | YamlString::Literal(s) | YamlString::Folded(s) => s,
        }
    }

    pub fn to_string(self) -> String {
        match self {
            YamlString::Plain(s) | YamlString::Literal(s) | YamlString::Folded(s) => s,
        }
    }
}
```

**Step 2: Define YamlNumber enum**

```rust
/// Represents YAML numbers (integer or float)
#[derive(Debug, Clone, PartialEq)]
pub enum YamlNumber {
    Integer(i64),
    Float(f64),
}

impl YamlNumber {
    pub fn as_f64(&self) -> f64 {
        match self {
            YamlNumber::Integer(i) => *i as f64,
            YamlNumber::Float(f) => *f,
        }
    }
}
```

**Step 3: Update YamlValue enum**

Replace the existing enum with:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum YamlValue {
    Null,
    Bool(bool),
    Number(YamlNumber),
    String(YamlString),
    Array(Vec<YamlNode>),
    Object(IndexMap<String, YamlNode>),
    /// Alias reference to an anchor (e.g., *anchor_name)
    Alias(String),
    /// Multiple documents in one file (like JSONL in jsonquill)
    MultiDoc(Vec<YamlNode>),
}
```

**Step 4: Update YamlNode struct**

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct YamlNode {
    pub value: YamlValue,
    pub expanded: bool,
    /// Anchor name if this node has an anchor (e.g., &anchor_name)
    pub anchor: Option<String>,
    /// Original YAML formatting for this node (for format preservation)
    pub original_formatting: Option<String>,
}

impl YamlNode {
    pub fn new(value: YamlValue) -> Self {
        Self {
            value,
            expanded: false,
            anchor: None,
            original_formatting: None,
        }
    }

    pub fn with_anchor(mut self, anchor: String) -> Self {
        self.anchor = Some(anchor);
        self
    }
}
```

**Step 5: Add IndexMap import**

At top of file:

```rust
use indexmap::IndexMap;
```

**Step 6: Attempt compilation**

```bash
cargo check 2>&1 | head -100
```

Expected: Errors about String vs YamlString mismatches throughout codebase

**Step 7: Commit data structure changes**

```bash
git add src/document/node.rs
git commit -m "feat: add YAML-specific data structures

- Add YamlString enum (Plain, Literal, Folded)
- Add YamlNumber enum (Integer, Float)
- Add Alias and MultiDoc variants to YamlValue
- Add anchor field to YamlNode
- Add original_formatting for format preservation

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 6: Update Parser for serde_yaml

**Files:**
- Modify: `src/document/parser.rs`

**Step 1: Update imports**

```rust
use serde_yaml::{self, Value};
use indexmap::IndexMap;
use crate::document::node::{YamlNode, YamlValue, YamlString, YamlNumber};
use anyhow::{Result, Context};
```

**Step 2: Write basic parse function**

```rust
/// Parse YAML string into YamlNode tree
pub fn parse_yaml(yaml_str: &str) -> Result<YamlNode> {
    let value: Value = serde_yaml::from_str(yaml_str)
        .context("Failed to parse YAML")?;

    convert_value(value)
}

fn convert_value(value: Value) -> Result<YamlNode> {
    let yaml_value = match value {
        Value::Null => YamlValue::Null,
        Value::Bool(b) => YamlValue::Bool(b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                YamlValue::Number(YamlNumber::Integer(i))
            } else if let Some(f) = n.as_f64() {
                YamlValue::Number(YamlNumber::Float(f))
            } else {
                anyhow::bail!("Invalid number in YAML")
            }
        }
        Value::String(s) => {
            // For now, treat all strings as Plain
            // Multi-line detection will be added in Phase 4
            YamlValue::String(YamlString::Plain(s))
        }
        Value::Sequence(arr) => {
            let nodes: Result<Vec<YamlNode>> = arr
                .into_iter()
                .map(convert_value)
                .collect();
            YamlValue::Array(nodes?)
        }
        Value::Mapping(map) => {
            let mut object = IndexMap::new();
            for (k, v) in map {
                let key = match k {
                    Value::String(s) => s,
                    _ => anyhow::bail!("Non-string key in YAML object"),
                };
                object.insert(key, convert_value(v)?);
            }
            YamlValue::Object(object)
        }
        Value::Tagged(_) => {
            // V1: Ignore tags, just parse the underlying value
            anyhow::bail!("Tagged values not supported in v1")
        }
    };

    Ok(YamlNode::new(yaml_value))
}
```

**Step 3: Add multi-document detection stub**

```rust
/// Parse YAML, detecting single vs multi-document files
pub fn parse_yaml_auto(yaml_str: &str) -> Result<YamlNode> {
    // V1: Single document only
    // Phase 3 will add multi-document support
    parse_yaml(yaml_str)
}
```

**Step 4: Attempt compilation**

```bash
cargo check 2>&1 | grep -A5 "parser.rs"
```

Expected: Should compile, but other modules will have errors

**Step 5: Commit parser changes**

```bash
git add src/document/parser.rs
git commit -m "feat: implement basic YAML parser using serde_yaml

- Parse YAML to YamlNode tree structure
- Convert serde_yaml::Value to YamlValue
- Handle integers vs floats in YamlNumber
- Treat all strings as Plain (multi-line detection in Phase 4)
- Single document only (multi-doc in Phase 3)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 7: Update File Loader for YAML

**Files:**
- Modify: `src/file/loader.rs`

**Step 1: Update load function signature**

```rust
use crate::document::parser::parse_yaml_auto;
use std::fs;
use std::path::Path;
use anyhow::{Result, Context};

pub fn load_yaml_file(path: &Path) -> Result<YamlNode> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    parse_yaml_auto(&contents)
}
```

**Step 2: Update gzip support**

```rust
use flate2::read::GzDecoder;
use std::io::Read;

pub fn load_yaml_file_auto(path: &Path) -> Result<YamlNode> {
    let contents = if path.extension().and_then(|s| s.to_str()) == Some("gz") {
        // Decompress gzip file
        let file = fs::File::open(path)
            .with_context(|| format!("Failed to open gzip file: {}", path.display()))?;
        let mut decoder = GzDecoder::new(file);
        let mut contents = String::new();
        decoder.read_to_string(&mut contents)
            .context("Failed to decompress gzip file")?;
        contents
    } else {
        fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?
    };

    parse_yaml_auto(&contents)
}
```

**Step 3: Commit loader changes**

```bash
git add src/file/loader.rs
git commit -m "feat: update file loader for YAML

- Load YAML files using parse_yaml_auto
- Support gzip compressed files (.yaml.gz)
- Auto-detect and decompress .gz files

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 8: Update File Saver for YAML

**Files:**
- Modify: `src/file/saver.rs`

**Step 1: Implement basic YAML serialization**

```rust
use crate::document::node::{YamlNode, YamlValue, YamlString, YamlNumber};
use serde_yaml::Value;
use indexmap::IndexMap;
use std::fs;
use std::path::Path;
use anyhow::{Result, Context};

pub fn save_yaml_file(path: &Path, root: &YamlNode) -> Result<()> {
    let yaml_value = convert_to_serde_value(root)?;
    let yaml_str = serde_yaml::to_string(&yaml_value)
        .context("Failed to serialize YAML")?;

    // Atomic write: write to temp file, then rename
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, yaml_str)
        .context("Failed to write temporary file")?;
    fs::rename(&temp_path, path)
        .context("Failed to rename temporary file")?;

    Ok(())
}

fn convert_to_serde_value(node: &YamlNode) -> Result<Value> {
    Ok(match &node.value {
        YamlValue::Null => Value::Null,
        YamlValue::Bool(b) => Value::Bool(*b),
        YamlValue::Number(n) => match n {
            YamlNumber::Integer(i) => Value::Number((*i).into()),
            YamlNumber::Float(f) => {
                Value::Number(serde_yaml::Number::from(*f))
            }
        },
        YamlValue::String(s) => {
            // V1: Always output as plain string
            // Phase 4 will preserve literal/folded style
            Value::String(s.as_str().to_string())
        }
        YamlValue::Array(arr) => {
            let values: Result<Vec<Value>> = arr
                .iter()
                .map(convert_to_serde_value)
                .collect();
            Value::Sequence(values?)
        }
        YamlValue::Object(obj) => {
            let mut map = serde_yaml::Mapping::new();
            for (k, v) in obj {
                map.insert(
                    Value::String(k.clone()),
                    convert_to_serde_value(v)?
                );
            }
            Value::Mapping(map)
        }
        YamlValue::Alias(_) => {
            // V1: Cannot serialize aliases
            anyhow::bail!("Cannot serialize alias nodes in v1")
        }
        YamlValue::MultiDoc(_) => {
            // V1: Cannot serialize multi-doc
            anyhow::bail!("Cannot serialize multi-document in v1")
        }
    })
}
```

**Step 2: Add gzip compression support**

```rust
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;

pub fn save_yaml_file_auto(path: &Path, root: &YamlNode) -> Result<()> {
    if path.extension().and_then(|s| s.to_str()) == Some("gz") {
        // Compress with gzip
        let yaml_value = convert_to_serde_value(root)?;
        let yaml_str = serde_yaml::to_string(&yaml_value)
            .context("Failed to serialize YAML")?;

        let temp_path = path.with_extension("tmp.gz");
        let file = fs::File::create(&temp_path)
            .context("Failed to create temporary gzip file")?;
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(yaml_str.as_bytes())
            .context("Failed to write compressed data")?;
        encoder.finish()
            .context("Failed to finish compression")?;

        fs::rename(&temp_path, path)
            .context("Failed to rename temporary file")?;
        Ok(())
    } else {
        save_yaml_file(path, root)
    }
}
```

**Step 3: Commit saver changes**

```bash
git add src/file/saver.rs
git commit -m "feat: implement YAML serialization

- Convert YamlNode tree to serde_yaml::Value
- Atomic file writes (temp file + rename)
- Gzip compression support for .gz files
- V1 limitation: Cannot serialize Alias or MultiDoc

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 9: Fix String Type Mismatches Throughout Codebase

**Files:**
- Modify: `src/ui/tree_view.rs`, `src/editor/*.rs`, and other files with String usage

**Step 1: Update tree_view.rs to handle YamlString**

Wherever we display or compare strings, use:

```rust
// Old: value matches String
match value {
    YamlValue::String(s) => { /* use s directly */ }
}

// New: value matches YamlString
match value {
    YamlValue::String(yaml_str) => {
        let s = yaml_str.as_str(); // Get &str
        // use s
    }
}
```

**Step 2: Update editor operations to create YamlString::Plain**

When creating new string values:

```rust
// Old:
YamlValue::String(input_value)

// New:
YamlValue::String(YamlString::Plain(input_value))
```

**Step 3: Update search to work with YamlString**

```rust
// In search functions, extract string content:
match &node.value {
    YamlValue::String(yaml_str) => {
        if yaml_str.as_str().contains(pattern) {
            // match found
        }
    }
    // ...
}
```

**Step 4: Iteratively fix compilation errors**

```bash
cargo check 2>&1 | head -20
```

Fix each error related to String vs YamlString

**Step 5: Commit string fixes**

```bash
git add src/
git commit -m "fix: update codebase to use YamlString enum

- Use YamlString::Plain when creating new strings
- Use .as_str() to get string content
- Update tree_view, editor, and search to handle YamlString

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 10: Create Basic Test Suite

**Files:**
- Create: `tests/basic_yaml.rs`
- Create: `examples/sample.yaml`

**Step 1: Create sample YAML file**

Create `examples/sample.yaml`:

```yaml
name: Test Document
version: 1.0
config:
  timeout: 30
  retry: 3
  enabled: true
users:
  - name: Alice
    age: 30
  - name: Bob
    age: 25
```

**Step 2: Write basic integration test**

Create `tests/basic_yaml.rs`:

```rust
use yamlquill::document::parser::parse_yaml;
use yamlquill::document::node::{YamlValue, YamlString};

#[test]
fn test_parse_simple_yaml() {
    let yaml = r#"
name: Test
count: 42
enabled: true
"#;

    let node = parse_yaml(yaml).expect("Failed to parse YAML");

    match &node.value {
        YamlValue::Object(obj) => {
            assert_eq!(obj.len(), 3);

            // Check name
            let name = obj.get("name").expect("name field missing");
            match &name.value {
                YamlValue::String(s) => assert_eq!(s.as_str(), "Test"),
                _ => panic!("name should be string"),
            }

            // Check count
            let count = obj.get("count").expect("count field missing");
            match &count.value {
                YamlValue::Number(n) => assert_eq!(n.as_f64(), 42.0),
                _ => panic!("count should be number"),
            }

            // Check enabled
            let enabled = obj.get("enabled").expect("enabled field missing");
            match &enabled.value {
                YamlValue::Bool(b) => assert!(*b),
                _ => panic!("enabled should be bool"),
            }
        }
        _ => panic!("Root should be object"),
    }
}

#[test]
fn test_parse_array() {
    let yaml = r#"
- Alice
- Bob
- Carol
"#;

    let node = parse_yaml(yaml).expect("Failed to parse YAML");

    match &node.value {
        YamlValue::Array(arr) => {
            assert_eq!(arr.len(), 3);
            match &arr[0].value {
                YamlValue::String(s) => assert_eq!(s.as_str(), "Alice"),
                _ => panic!("Array element should be string"),
            }
        }
        _ => panic!("Root should be array"),
    }
}
```

**Step 3: Run tests**

```bash
cargo test
```

Expected: Tests should pass

**Step 4: Commit tests**

```bash
git add tests/basic_yaml.rs examples/sample.yaml
git commit -m "test: add basic YAML parsing tests

- Test simple object parsing
- Test array parsing
- Add sample.yaml example file

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 11: Attempt Full Compilation

**Files:**
- All source files

**Step 1: Run full compilation check**

```bash
cargo build 2>&1 | tee build.log
```

**Step 2: Address remaining compilation errors**

Go through build.log and fix:
- Remaining json→yaml renames
- Type mismatches (String vs YamlString, etc.)
- Missing imports
- Function signature changes

**Step 3: Iteratively fix until clean build**

```bash
cargo build
```

Expected: Successful compilation (with warnings OK)

**Step 4: Run formatter and clippy**

```bash
cargo fmt
cargo clippy -- -D warnings
```

Fix any clippy warnings

**Step 5: Commit final fixes**

```bash
git add src/
git commit -m "fix: resolve remaining compilation errors

- Fix missed json→yaml renames
- Resolve all type mismatches
- Clean up imports
- Pass cargo clippy

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 12: Test Basic Functionality

**Files:**
- Test: Binary execution

**Step 1: Build release binary**

```bash
cargo build --release
```

Expected: Successful build

**Step 2: Test loading sample YAML**

```bash
./target/release/yamlquill examples/sample.yaml
```

Expected: Editor opens, displays YAML tree (may have bugs, that's OK for Phase 1)

**Step 3: Test basic navigation**

In the editor:
- Press `j` to move down
- Press `k` to move up
- Press `l` to expand node
- Press `h` to collapse node
- Press `:q` to quit

Expected: Basic vim-style navigation works

**Step 4: Document known issues**

Create notes on what doesn't work yet (to be fixed in Phase 2+)

---

## Task 13: Update Documentation Stubs

**Files:**
- Create: `README.md`
- Create: `CLAUDE.md`

**Step 1: Create basic README**

```markdown
# YAMLQuill

A terminal-based structural YAML editor with vim-style keybindings.

**Status:** Phase 1 Complete - Basic structure implemented

Based on [JSONQuill](https://github.com/joeygibson/jsonquill) architecture.

## Current Status

✅ Phase 1: Core structure copied and compiling
⏳ Phase 2: YAML document model (in progress)
⏳ Phase 3: Multi-document support
⏳ Phase 4: YAML-specific features
⏳ Phase 5: Polish & parity

## Building

```bash
cargo build --release
```

## Usage

```bash
./target/release/yamlquill examples/sample.yaml
```

## License

MIT License - see LICENSE.md
```

**Step 2: Create basic CLAUDE.md**

```markdown
# CLAUDE.md

This file provides guidance to Claude Code when working in this repository.

## Project Overview

YAMLQuill is a terminal-based structural YAML editor, forked from JSONQuill.

## Development Commands

```bash
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

```bash
cargo fmt && cargo clippy -- -D warnings && cargo test
```

## Architecture

See `docs/plans/2026-01-31-yamlquill-design.md` for full design.

**Module structure:**
- `src/document/` - YAML parsing and tree representation
- `src/editor/` - Editor state, modes, undo/redo
- `src/ui/` - Terminal UI rendering
- `src/file/` - File I/O, gzip support
- `src/theme/` - Color themes
- `src/yamlpath/` - Query support

## Implementation Phases

Currently in Phase 1 (Core Structure).

See `docs/plans/2026-01-31-phase1-core-structure.md` for detailed tasks.
```

**Step 3: Commit documentation**

```bash
git add README.md CLAUDE.md
git commit -m "docs: add initial README and CLAUDE.md

- Basic project overview
- Build and usage instructions
- Development commands
- Architecture overview

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 14: Final Verification and Phase 1 Complete

**Files:**
- All project files

**Step 1: Run full test suite**

```bash
cargo test
```

Expected: All tests pass

**Step 2: Verify binary works**

```bash
cargo run -- examples/sample.yaml
```

Expected: Editor loads and displays YAML

**Step 3: Check git status**

```bash
git status
```

Expected: Clean working directory (all changes committed)

**Step 4: Create Phase 1 completion tag**

```bash
git tag -a v0.1.0-phase1 -m "Phase 1 Complete: Core Structure

- Copied JSONQuill codebase
- Renamed json→yaml throughout
- Updated dependencies (serde_yaml, indexmap)
- Implemented basic YAML parser
- Added YAML-specific data structures
- Basic file I/O with gzip support
- Compiling and runnable baseline

Next: Phase 2 - YAML Document Model"

git log --oneline --graph -10
```

**Step 5: Verify implementation plan complete**

Review all tasks in this plan - all should be ✅

---

## Success Criteria

Phase 1 is complete when:

✅ JSONQuill codebase copied to yamlquill worktree
✅ All json→yaml renaming complete
✅ Cargo.toml updated with yamlquill metadata
✅ serde_yaml replaces serde_json
✅ YamlNode, YamlValue, YamlString, YamlNumber data structures defined
✅ Basic YAML parser implemented
✅ File loader/saver work for simple YAML
✅ Project compiles without errors
✅ Basic tests pass
✅ Binary runs and loads YAML files
✅ All changes committed to git
✅ Documentation stubs created

## Next Steps

After Phase 1 completion:

1. Create Phase 2 implementation plan (YAML Document Model)
2. Focus on full editing operations for single-document YAML
3. Improve tree navigation and display
4. Add comprehensive test coverage
