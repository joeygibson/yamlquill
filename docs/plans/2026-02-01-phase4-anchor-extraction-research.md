# Phase 4: Anchor Name Extraction Research

**Date:** 2026-02-01
**Status:** Research Needed
**Blocker For:** Phase 4b (UI/Navigation) and Phase 4c (Editing Constraints)

## The Problem

**Completed:** Phase 4a successfully implemented the foundation for anchor/alias support:
- ✅ yaml-rust2 dependency added
- ✅ Data model extended (alias_target field, AnchorRegistry)
- ✅ Parser swapped from serde_yaml to yaml-rust2
- ✅ All 377 tests passing

**Blocker:** yaml-rust2 **automatically resolves anchors during parsing**, converting alias references to copies of the anchored data. This makes it impossible to extract the original anchor names (`&name`) and alias references (`*name`) from the parsed structure.

## Example of the Problem

**Input YAML:**
```yaml
defaults: &config
  timeout: 30
  retries: 3

production:
  settings: *config  # Alias reference
```

**What we need:**
- Node at path `["defaults"]` should have `anchor = Some("config")`
- Node at path `["production", "settings"]` should be `YamlValue::Alias("config")` with `alias_target = Some("config")`

**What yaml-rust2 gives us:**
- Node at path `["defaults"]` has the object `{timeout: 30, retries: 3}` but **no anchor name**
- Node at path `["production", "settings"]` is a **copy** of the object, not an alias reference

The `Yaml::Alias(usize)` variant uses a numeric ID that doesn't map to the original anchor name in the source text.

## Research Options

### Option A: Use yaml-rust2 Scanner Directly

**Approach:** Use yaml-rust2's lower-level scanner API instead of the high-level `YamlLoader`.

**Pros:**
- Stay with yaml-rust2 (already a dependency)
- Scanner should preserve anchor names
- Full YAML 1.2 compliance

**Cons:**
- Much more complex (manual event processing)
- Need to build our own tree structure from scanner events
- Significant development effort

**Next Steps:**
1. Read yaml-rust2 scanner documentation
2. Prototype event-based parsing
3. Map scanner events to our YamlNode structure
4. Test with complex anchor/alias scenarios

**Files to explore:**
- `yaml_rust2::scanner::Scanner`
- `yaml_rust2::scanner::ScanError`
- Look for anchor/alias events in the scanner API

### Option B: Use Different YAML Library

**Approach:** Evaluate alternative Rust YAML parsers that preserve anchor information.

**Candidates:**
1. **saphyr-parser** - Fork of yaml-rust with improvements
   - Check if it preserves anchor names better
   - URL: https://crates.io/crates/saphyr

2. **yaml-peg** - PEG-based parser
   - May have better anchor support
   - URL: https://crates.io/crates/yaml-peg

3. **yaml-rust** (original) - Check if behavior differs from yaml-rust2

**Pros:**
- Might have ready-made anchor preservation
- Could be simpler than Option A

**Cons:**
- Need to evaluate multiple libraries
- May not have anchor support either
- Switching libraries again is disruptive

**Next Steps:**
1. Research each library's anchor/alias handling
2. Create small prototypes with each
3. Compare API complexity vs yaml-rust2
4. Check maintenance status and YAML spec compliance

### Option C: Custom Anchor Scanner

**Approach:** Write a lightweight scanner that extracts anchor/alias positions from the YAML source text, then correlate with yaml-rust2's parsed structure.

**Implementation:**
```rust
/// Scans YAML source for anchor definitions and alias references
struct AnchorScanner {
    /// Maps line:column positions to anchor names
    anchor_positions: HashMap<(usize, usize), String>,

    /// Maps line:column positions to alias target names
    alias_positions: HashMap<(usize, usize), String>,
}

impl AnchorScanner {
    /// Scan YAML source text for &anchor and *alias patterns
    fn scan(yaml_str: &str) -> Result<Self>;
}
```

**Algorithm:**
1. Use regex or simple parser to find `&name` and `*name` patterns
2. Record line/column positions
3. Parse YAML with yaml-rust2 (which provides position info in some contexts)
4. Correlate scanner results with parsed tree using positions
5. Populate anchor/alias_target fields in YamlNode

**Pros:**
- Keep using yaml-rust2 for main parsing
- Focused solution for just the anchor problem
- Can handle YAML edge cases (anchors in flow vs block style)

**Cons:**
- Need to maintain two parallel parsing passes
- Position correlation might be fragile
- Must handle all YAML syntax quirks (quoted strings, comments, etc.)

**Next Steps:**
1. Research YAML anchor syntax rules
2. Prototype regex-based anchor detection
3. Check if yaml-rust2 provides position information
4. Test position correlation accuracy

## Recommended Approach

**Start with Option A** (yaml-rust2 scanner) for these reasons:

1. **Already using yaml-rust2** - No new dependencies
2. **Most robust** - Scanner events are the source of truth
3. **Learning opportunity** - Better understanding of YAML parsing
4. **Future-proof** - Can handle any YAML feature, not just anchors

**Fallback plan:** If Option A proves too complex, try Option C (custom scanner) as it's more contained.

**Avoid Option B** unless both A and C fail - switching libraries again is costly.

## Implementation Plan (Once Approach Chosen)

### Phase 4b: UI and Navigation (Blocked)

**Prerequisites:**
- Anchor name extraction working
- AnchorRegistry properly populated during parsing

**Tasks:**
- Display anchor badges in tree view (`&name` shown dimmed)
- Display alias nodes with distinct color (`*name`)
- Implement navigation (Enter/gd jumps from alias to anchor)
- Update collapsed preview to show anchors

### Phase 4c: Editing Constraints (Blocked)

**Prerequisites:**
- Phase 4b complete
- Navigation working

**Tasks:**
- Implement read-only alias enforcement (error on edit attempt)
- Implement delete protection (prevent deleting anchors with aliases)
- Add paste conflict handling (auto-rename duplicate anchors)
- Show helpful error messages

## Test Strategy

**Before starting research:**
- All existing tests must pass (baseline: 377 tests)

**During research:**
- Create spike branch for each option
- Test with real-world YAML files (Kubernetes, Docker Compose)
- Measure complexity (lines of code, API calls needed)

**Success criteria:**
- Can extract `&config` from `defaults: &config`
- Can identify `*config` as alias reference
- Can map alias to anchor definition
- Works with nested structures, arrays, multi-document files

## Timeline Considerations

**If research takes > 2 days:**
- Consider deferring full anchor/alias support to v2.0
- YAMLQuill v1.0 is already feature-complete without anchors
- This matches the original plan (anchors were a "nice to have")

**If quick solution found:**
- Complete Phase 4b and 4c
- Ship anchor/alias support in v1.1

## Files to Update After Research

Once anchor extraction is solved:

1. **src/document/parser.rs**
   - Remove `bail!("Alias support not yet implemented")`
   - Implement anchor name extraction in `convert_yaml_rust2()`
   - Populate `YamlNode.anchor` and `YamlNode.alias_target`
   - Register anchors/aliases in AnchorRegistry

2. **tests/anchor_alias_parser_tests.rs**
   - Remove `#[ignore]` from `test_parse_yaml_with_anchor`
   - Add tests for alias parsing
   - Add tests for complex anchor scenarios

3. **src/ui/tree_view.rs** (Phase 4b)
   - Implement anchor badge display
   - Implement alias node rendering
   - Add navigation support

4. **src/editor/state.rs** (Phase 4c)
   - Add read-only alias enforcement
   - Add delete protection
   - Add paste conflict handling

## Resources

- yaml-rust2 documentation: https://docs.rs/yaml-rust2
- YAML 1.2 spec (anchors): https://yaml.org/spec/1.2.2/#3222-anchors-and-aliases
- YAMLQuill design doc: `docs/plans/2026-02-01-phase4-anchor-alias-design.md`
- Implementation plan: `docs/plans/2026-02-01-phase4-implementation.md`

## Notes for Future Claude

**Current state:**
- Working directory: `/Users/jgibson/Projects/yamlquill` (main repo)
- Worktree: `/Users/jgibson/.config/superpowers/worktrees/yamlquill/phase4-anchor-alias` (can be cleaned up)
- Branch: `feature/phase4-anchor-alias` merged to `main`
- Tests: 377 passing, 4 ignored (including anchor parsing test)
- Commit: Phase 4a complete, blocker documented

**To resume research:**
1. Create new branch: `feature/phase4-anchor-extraction-research`
2. Try Option A first (yaml-rust2 scanner)
3. Create prototype in separate file to avoid breaking existing code
4. Test with: `cargo test test_parse_yaml_with_anchor --ignored`

**Key insight:** The infrastructure is solid. Only the anchor name extraction mechanism needs solving. Everything else is ready to go.
