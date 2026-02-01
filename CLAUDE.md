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
