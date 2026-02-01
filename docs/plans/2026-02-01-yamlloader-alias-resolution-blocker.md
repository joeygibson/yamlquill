# YamlLoader Alias Resolution Blocker

**Date:** 2026-02-01  
**Status:** BLOCKER for Phase 4b anchor/alias implementation

## Problem

`yaml_rust2::YamlLoader::load_from_str()` automatically resolves alias references (`*name`) into copies of the anchored data, making it impossible to preserve `Yaml::Alias` nodes in the parsed tree.

## Evidence

### Test Input
```yaml
defaults: &config
  timeout: 30

production:
  settings: *config
```

### Expected Behavior
```rust
production.settings => Yaml::Alias(id_pointing_to_config)
```

### Actual Behavior
```rust
production.settings => Yaml::Hash({
    "timeout": Yaml::Integer(30)
})  // Resolved copy, not an Alias node!
```

## Root Cause

YamlLoader is designed as a convenience API that resolves aliases during parsing. While the `Yaml::Alias(usize)` variant exists in the enum, YamlLoader never produces it in the output tree - it always resolves to copies.

From testing:
- Scanner correctly extracts anchor names: `&config`, `*config` ✅
- YamlLoader parses structure but resolves aliases ❌
- Result: No way to distinguish aliases from regular values in output

## Test Results

Created comprehensive test suite (`tests/anchor_alias_extraction_tests.rs`):
- ✅ 2/6 tests passing (non-alias cases work fine)
- ❌ 4/6 tests failing (all alias-related tests)

**Failing tests:**
- `test_parse_simple_alias` - Single alias reference resolved to copy
- `test_parse_multiple_aliases` - Multiple references all resolved
- `test_parse_nested_structure_with_alias` - Nested aliases resolved
- `test_parse_array_with_alias` - Array of aliases all resolved

## Impact

This blocker affects:
- ✅ Scanner extraction works (we can get anchor/alias names from source)
- ❌ Tree correlation impossible (YamlLoader hides alias nodes)
- ❌ Cannot populate `alias_target` field (no Alias nodes to detect)
- ❌ Cannot implement Phase 4b UI (need Alias nodes for display/navigation)

## Options Forward

### Option 1: Use Parser + EventReceiver (RECOMMENDED)

**Approach:** Build tree from yaml-rust2 Parser events instead of YamlLoader.

**Implementation:**
```rust
struct TreeBuilder {
    events: Vec<Event>,
    anchor_map: AnchorMap,
}

impl EventReceiver for TreeBuilder {
    fn on_event(&mut self, ev: Event) {
        match ev {
            Event::Alias(anchor_id) => {
                // Build Alias node with name from anchor_map
                let name = self.anchor_map.get_by_id(anchor_id);
                self.push_node(YamlNode::alias(name));
            }
            Event::MappingStart(anchor_id, tag) => {
                // Check if this mapping has an anchor
                if anchor_id > 0 {
                    let name = self.anchor_map.get_by_id(anchor_id);
                    self.current_node.set_anchor(name);
                }
            }
            // ... handle other events
        }
    }
}
```

**Pros:**
- Parser preserves Alias events (not resolved yet)
- Can correlate anchor IDs with Scanner names
- Full control over tree building
- Stays with yaml-rust2 (no new dependencies)

**Cons:**
- More complex than YamlLoader (need event → tree logic)
- Must handle all Event types (Mapping, Sequence, Scalar)
- Need to track parsing stack (current container, nesting level)
- Estimated effort: 2-3 days

**Status:** Not yet attempted, but Parser events likely preserve Alias

### Option 2: Try saphyr-parser Library

**Approach:** Evaluate saphyr (fork of yaml-rust with improvements).

**Research needed:**
1. Does saphyr's loader preserve Alias nodes?
2. API compatibility with yaml-rust2?
3. Maintenance status and YAML spec compliance?
4. Migration effort from yaml-rust2?

**Pros:**
- Might preserve aliases out-of-the-box
- Similar API to yaml-rust/yaml-rust2
- Active development (check crates.io)

**Cons:**
- Requires switching dependencies (already switched from serde_yaml → yaml-rust2)
- Unknown if it actually solves the problem
- API differences may require code changes

**Status:** Not yet researched, could be quick win if it works

### Option 3: Build Tree from Scanner Tokens

**Approach:** Parse YAML directly from Scanner token stream.

**Implementation:** Manually construct YamlNode tree by processing tokens:
- Track state machine (in_mapping, in_sequence, current_key)
- Build container stack for nesting
- Handle flow vs block style
- Manage indentation levels

**Pros:**
- Complete control
- Anchors/aliases available immediately
- No intermediate structures

**Cons:**
- **VERY COMPLEX** - essentially reimplementing yaml-rust2's parser
- Must handle all YAML syntax edge cases
- Error-prone and time-consuming
- Defeats purpose of using a YAML library
- Estimated effort: 1-2 weeks minimum

**Status:** NOT RECOMMENDED - too complex and risky

## Recommendation

**Try Option 1 (Parser + EventReceiver) first:**

1. Create prototype that builds tree from Parser events
2. Validate that Event::Alias is present in stream
3. Map anchor IDs to Scanner-extracted names
4. Handle all Event types to build YamlNode tree

**If Option 1 blocked (Parser also resolves):**

1. Quick research into saphyr-parser (Option 2)
2. If saphyr doesn't work, consider deferring Phase 4 to v2.0

**Time budget:** 1 day for Option 1 prototype. If not working, escalate decision.

## Files Modified So Far

- ✅ `src/document/parser.rs` - Added Scanner extraction, hybrid approach
- ✅ `tests/anchor_alias_extraction_tests.rs` - Comprehensive test suite
- ✅ All existing tests still pass (290 tests)

## Next Steps

1. Research Parser/EventReceiver approach
2. Create minimal prototype to verify Event::Alias is available
3. If viable, implement full tree building from events
4. If not viable, research saphyr-parser
5. Update task tracking and timeline

## References

- yaml-rust2 Parser docs: https://docs.rs/yaml-rust2/latest/yaml_rust2/parser/
- EventReceiver trait: https://docs.rs/yaml-rust2/latest/yaml_rust2/parser/trait.EventReceiver.html
- Event enum: https://docs.rs/yaml-rust2/latest/yaml_rust2/parser/enum.Event.html
- saphyr-parser: https://crates.io/crates/saphyr

## Conclusion

The hybrid Scanner + YamlLoader approach hits a fundamental limitation: YamlLoader resolves aliases. We need to use a lower-level API (Parser) or different library (saphyr) to preserve Alias nodes in the tree.

The Scanner-based anchor name extraction works perfectly. The blocker is purely in the tree building phase.
