# Phase 4: Anchor & Alias Support Design

**Date:** 2026-02-01
**Status:** Approved for Implementation

## Overview

Add support for YAML anchors (`&name`) and aliases (`*name`) to YAMLQuill, enabling users to navigate reference relationships and edit anchored values while maintaining data integrity.

## Goals

- Parse and preserve YAML anchors and aliases
- Display anchors and aliases clearly in the tree view
- Enable navigation from alias to anchor definition
- Prevent accidental breaking of alias references through editing constraints
- Maintain YAMLQuill's structural editing philosophy

## Non-Goals (Deferred)

- Comment preservation (requires additional parser features)
- Creating new anchors/aliases through the UI (preserve-only for v1)
- Multi-line string editing with Shift+Enter (terminal limitations)
- Force delete (`dd!`) for anchors with aliases (future enhancement)

## Architecture

### Core Components

1. **Parser Module** (`src/document/parser.rs`)
   - Replace serde_yaml with yaml-rust2
   - Build existing `YamlNode` tree structure with anchor/alias tracking
   - Populate `anchor` and `alias_target` fields during parsing

2. **Data Model Extensions**
   - `YamlNode.anchor: Option<String>` - already exists, now populated
   - `YamlNode.alias_target: Option<String>` - NEW field for alias nodes
   - `YamlValue::Alias(String)` - already exists, now used
   - `AnchorRegistry` - NEW component for tracking anchor/alias relationships

3. **Anchor Registry** (new struct in `YamlTree`)
   - Maps anchor names to paths
   - Maps alias paths to anchor names
   - Enables validation and navigation

4. **UI/Display** (`src/ui/tree_view.rs`)
   - Inline badge display for anchors and aliases
   - Distinct colors for visual distinction
   - Navigation support (Enter/gd to jump to anchor)

5. **Editing Constraints** (`src/editor/state.rs`)
   - Read-only aliases (show error on edit attempt)
   - Prevent deletion of anchors with existing aliases
   - Auto-rename anchors on paste conflicts

## Data Model

### YamlNode Structure

```rust
pub struct YamlNode {
    pub(crate) value: YamlValue,
    pub(crate) metadata: NodeMetadata,
    pub(crate) anchor: Option<String>,        // Existing - will populate
    pub(crate) alias_target: Option<String>,  // NEW - for alias nodes
    pub(crate) original_formatting: Option<String>,
}
```

### AnchorRegistry

```rust
pub struct AnchorRegistry {
    // anchor_name -> path to the node with that anchor
    anchor_definitions: HashMap<String, Vec<usize>>,

    // path -> anchor_name (for nodes that reference an alias)
    alias_references: HashMap<Vec<usize>, String>,
}

impl AnchorRegistry {
    pub fn register_anchor(&mut self, name: String, path: Vec<usize>);
    pub fn register_alias(&mut self, path: Vec<usize>, target: String);
    pub fn get_anchor_path(&self, name: &str) -> Option<&Vec<usize>>;
    pub fn get_aliases_for(&self, anchor: &str) -> Vec<&Vec<usize>>;
    pub fn can_delete_anchor(&self, name: &str) -> bool;
}
```

## Parser Implementation

### yaml-rust2 Integration

Replace `serde_yaml::Deserializer` with `yaml_rust2::YamlLoader`:

```rust
use yaml_rust2::{YamlLoader, Yaml};

pub fn parse_yaml_auto(yaml_str: &str) -> Result<YamlNode> {
    let docs = YamlLoader::load_from_str(yaml_str)?;

    if docs.len() == 1 {
        convert_yaml_to_node(&docs[0], &mut AnchorRegistry::new())
    } else if docs.is_empty() {
        bail!("No YAML documents found")
    } else {
        let mut registry = AnchorRegistry::new();
        let nodes: Vec<YamlNode> = docs.iter()
            .map(|doc| convert_yaml_to_node(doc, &mut registry))
            .collect::<Result<_>>()?;
        Ok(YamlNode::new(YamlValue::MultiDoc(nodes)))
    }
}
```

### Anchor/Alias Conversion

```rust
fn convert_yaml_to_node(yaml: &Yaml, registry: &mut AnchorRegistry) -> Result<YamlNode> {
    match yaml {
        Yaml::Alias(anchor_id) => {
            let anchor_name = resolve_anchor_name(anchor_id);
            Ok(YamlNode {
                value: YamlValue::Alias(anchor_name.clone()),
                alias_target: Some(anchor_name),
                ..Default::default()
            })
        }
        // Handle other types with anchor tracking
        _ => { /* conversion logic */ }
    }
}
```

**Challenge:** yaml-rust2 uses numeric anchor IDs; need to maintain mapping to actual anchor names from source text.

## Display and Navigation

### Tree View Display

```
definitions:
  ▸ default_config: {...} &default     <- anchor badge (dimmed)
  ▸ prod_config: {...} &prod
services:
  ▸ api:
      port: 8080
      settings: *default               <- alias (distinct color)
  ▸ web:
      port: 3000
      settings: *prod
```

### Display Implementation

```rust
fn format_node_with_anchor(node: &YamlNode, key: Option<&str>) -> String {
    let mut display = String::new();

    if let Some(k) = key {
        display.push_str(&format!("{}: ", k));
    }

    display.push_str(&get_value_preview(node.value()));

    if let Some(anchor) = &node.anchor {
        display.push_str(&format!(" &{}", anchor).dimmed());
    }

    display
}

fn format_alias_node(target: &str) -> String {
    format!("*{}", target).with_color(alias_color)
}
```

### Navigation Behavior

- **Enter** or **gd** on alias node: Jump to anchor definition
- Uses existing jumplist infrastructure (Ctrl-o to return)
- Lookup via `AnchorRegistry.get_anchor_path()`

## Editing Constraints

### Read-Only Aliases

```rust
pub fn start_editing(&mut self) -> Result<()> {
    let node = self.tree.get_node(self.cursor.path())?;

    if matches!(node.value(), YamlValue::Alias(_)) {
        bail!("Cannot edit alias - navigate to anchor definition (&{}) to edit",
              node.alias_target.as_ref().unwrap());
    }

    self.mode = EditorMode::Insert;
    // ... existing edit setup
}
```

### Delete Protection

```rust
pub fn delete_current_node(&mut self) -> Result<()> {
    let node = self.tree.get_node(self.cursor.path())?;

    if let Some(anchor_name) = &node.anchor {
        let aliases = self.tree.anchor_registry().get_aliases_for(anchor_name);
        if !aliases.is_empty() {
            bail!(
                "Cannot delete anchor '{}' - {} alias(es) reference it\n\
                 Delete aliases first or use force delete (not yet implemented)",
                anchor_name,
                aliases.len()
            );
        }
    }

    // Update registry and proceed with deletion
    self.tree.anchor_registry_mut().remove_node(self.cursor.path());
    // ... existing deletion logic
}
```

### Paste Behavior

When pasting a node with an anchor:
- Check for anchor name conflicts
- Auto-rename if needed: `&default` → `&default_2`
- Show warning: "Renamed anchor 'default' to 'default_2' (name conflict)"

## Testing Strategy

### Unit Tests

1. **Parser Tests** (`src/document/parser.rs`)
   - Parse YAML with anchors
   - Parse YAML with aliases
   - Parse YAML with both anchors and aliases
   - Multi-document with anchors/aliases
   - Invalid alias references (missing anchor)
   - Anchor name conflicts

2. **Registry Tests** (`src/document/tree.rs`)
   - Register anchor and query path
   - Register multiple aliases for one anchor
   - Query aliases for anchor
   - Remove node updates registry
   - Can/cannot delete anchor validation

3. **Display Tests** (`src/ui/tree_view.rs`)
   - Anchor badge rendering
   - Alias node rendering with distinct color
   - Format preserves badges in collapsed view

4. **Editing Tests** (`tests/anchor_alias_tests.rs`)
   - Edit alias attempt → error
   - Delete anchor with aliases → error with count
   - Delete anchor without aliases → success
   - Navigate from alias to anchor
   - Paste anchored node → auto-rename

### Integration Tests

```rust
#[test]
fn test_roundtrip_anchors_and_aliases() {
    let yaml = r#"
defaults: &config
  timeout: 30
api:
  settings: *config
"#;
    // Parse → modify anchor → save → reparse → verify
}
```

### Manual Testing

- [ ] Load Kubernetes manifests with anchors
- [ ] Load Docker Compose files with anchors
- [ ] Navigate from alias to anchor with Enter
- [ ] Try to edit alias (should error)
- [ ] Delete anchor with aliases (should block)
- [ ] Edit anchor value (all aliases reflect on reload)

## Implementation Phases

### Phase 4a: Parser Swap
- Add yaml-rust2 dependency
- Rewrite `parse_yaml_auto()` and `convert_yaml_to_node()`
- Implement `AnchorRegistry`
- Add to `YamlTree`
- Basic parsing tests

### Phase 4b: UI/Navigation
- Update tree view display with badges
- Implement navigation (Enter/gd on alias)
- Add distinct colors for anchors/aliases
- Display tests

### Phase 4c: Editing Constraints
- Implement read-only alias enforcement
- Implement delete protection
- Add paste conflict handling
- Editing constraint tests

## Migration Strategy

- Incremental: Each phase is independently testable
- Backward compatible: Existing YAML without anchors works unchanged
- No breaking changes to data structures (only additions)
- All existing tests should continue to pass

## Success Criteria

- [ ] Parse and display real-world YAML files with anchors/aliases (K8s, Docker Compose)
- [ ] Navigate from alias to anchor definition seamlessly
- [ ] Editing an anchor updates all aliases on reload
- [ ] Cannot accidentally delete anchors with existing aliases
- [ ] Cannot edit alias nodes directly
- [ ] All 290+ existing tests still pass
- [ ] New anchor/alias tests cover edge cases

## Future Enhancements (v2.0+)

- Create new anchors through UI (`:anchor` command)
- Create new aliases through UI (`:alias` command)
- Force delete for anchors (`dd!` to delete anchor and convert aliases to copies)
- Comment preservation (requires parser enhancement)
- Live update of all aliases when editing anchor (currently requires reload)
