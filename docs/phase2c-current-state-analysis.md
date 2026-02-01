# Phase 2c: Current State Analysis - Value Editing

**Date:** 2026-02-01
**Analyst:** Claude Code
**Method:** Code review and static analysis

## Summary

YAMLQuill inherited editing infrastructure from JSONQuill, but there are **critical bugs** with YAML-specific types that need fixing before the editor can work correctly.

---

## Current Implementation

### Editing Flow (`src/editor/state.rs:2850-2924`)

1. User presses `i` on a value → enters INSERT mode
2. Edit buffer initialized with current value
3. User types new value
4. User presses Enter → `commit_editing()` is called
5. New value is parsed based on **original type**
6. Node is updated in tree

### Type-Specific Parsing

```rust
// Line 2872: Strings always become Plain
YamlValue::String(_) => YamlValue::String(YamlString::Plain(buffer_content))

// Line 2873-2882: Numbers - smart parsing (int first, then float)
YamlValue::Number(_) => {
    if let Ok(i) = buffer_content.parse::<i64>() {
        YamlValue::Number(YamlNumber::Integer(i))
    } else {
        YamlValue::Number(YamlNumber::Float(num))
    }
}

// Line 2884-2890: Booleans - strict parsing
YamlValue::Boolean(_) => {
    match buffer_content.as_str() {
        "true" => true,
        "false" => false,
        _ => return Err(anyhow!("Boolean value must be true or false")),
    }
}
```

---

## Identified Issues

### 🔴 CRITICAL: String Style Loss (Line 2872)

**Problem:**
When editing a Literal (`|`) or Folded (`>`) string, it **always converts to Plain** style.

**Example:**
```yaml
# Before editing
description: |
  Multi-line
  content

# User presses 'i', changes "Multi" to "Many", presses Enter

# After editing - BUG!
description: Many-line\ncontent  # Lost the | style!
```

**Impact:**
- Users cannot edit multi-line strings without corrupting the YAML format
- Round-trip editing destroys formatting
- Critical for Phase 2 success criteria

**Root Cause:**
```rust
// This always creates Plain, ignoring original style:
YamlValue::String(_) => YamlValue::String(YamlString::Plain(buffer_content))
```

**Fix Required:**
Preserve the original YamlString variant:
```rust
YamlValue::String(original_style) => {
    match original_style {
        YamlString::Plain(_) => YamlValue::String(YamlString::Plain(buffer_content)),
        YamlString::Literal(_) => YamlValue::String(YamlString::Literal(buffer_content)),
        YamlString::Folded(_) => YamlValue::String(YamlString::Folded(buffer_content)),
    }
}
```

---

### 🟡 MEDIUM: No Multi-Line Input Support

**Problem:**
Edit buffer is a single-line `String`. Users cannot enter newlines when editing Literal/Folded strings.

**Example:**
```yaml
description: |
  Line 1
  Line 2

# User wants to add "Line 3"
# But pressing Enter commits the edit, can't add newline!
```

**Impact:**
- Cannot create new multi-line strings
- Cannot edit existing multi-line strings properly
- Workaround: Edit in external editor, reload file

**JSONQuill Comparison:**
JSONQuill doesn't have this issue because JSON strings use `\n` escape sequences. Users type `\n` literally as two characters.

**Fix Options:**
1. **Shift+Enter** adds newline, **Enter** commits (recommended)
2. **Ctrl+J** adds newline, **Enter** commits
3. Show multi-line editor widget when editing Literal/Folded strings

**Phase 2c Decision Needed:**
Is multi-line editing in scope? Or defer to Phase 4?

---

### 🟢 WORKING: Number Parsing

**Status:** ✅ Works correctly

Lines 2873-2882 already implement smart parsing:
- Tries integer first (`parse::<i64>()`)
- Falls back to float (`parse::<f64>()`)
- Example: `"42"` → Integer(42), `"42.5"` → Float(42.5)

**No issues found.**

---

### 🟢 WORKING: Boolean Parsing

**Status:** ✅ Works correctly (strict)

Lines 2884-2890 only accept `"true"` or `"false"` (lowercase).

**JSONQuill Comparison:**
JSONQuill likely has same restriction (JSON only allows `true`/`false`).

**Note:**
YAML also accepts `yes`/`no`, `on`/`off`, but strict parsing is safer. No change needed.

---

### 🟡 MEDIUM: No Type Conversion

**Problem:**
Cannot change a value's type during editing.

**Example:**
```yaml
count: 42  # number

# User wants to make it a string "42"
# User presses 'i', types "42", presses Enter
# Result: Still a number! No way to force string type.
```

**JSONQuill Comparison:**
JSONQuill has same limitation - type is determined by syntax (`42` vs `"42"`).
But JSON quotes are visible in the editor, YAML quotes are not.

**Fix Options:**
1. **No fix** - matches JSONQuill behavior, accept limitation
2. **Add quotes in buffer** - show `"42"` in buffer for strings, parse quotes to determine type
3. **Add :convert command** - explicit type change (previously discussed, rejected)

**Recommendation:**
Accept limitation for Phase 2c, document as known issue.

---

### 🟢 MINOR: Null Handling

**Status:** ✅ Works correctly

Line 2867: Special case for "null" always converts to Null type.

**No issues found.**

---

## Missing Functionality

### ❌ No Validation

**Issue:**
Invalid input is only caught when parsing fails. No pre-validation.

**Example:**
```yaml
count: 42

# User presses 'i', types "not a number", presses Enter
# Error: "Invalid number format" - but edit buffer is lost!
```

**Impact:**
Poor UX - user loses their input on validation error.

**Fix:**
Add validation before committing:
```rust
// Validate first
if let YamlValue::Number(_) = node.value() {
    if buffer_content.parse::<i64>().is_err()
        && buffer_content.parse::<f64>().is_err() {
        return Err(anyhow!("Invalid number: {}", buffer_content));
    }
}
// Then commit...
```

---

### ❌ No Type Information in Prompt

**Issue:**
Users don't know what type they're editing.

**Current:** `Edit: 42`
**Desired:** `Edit [integer]: 42`

**Impact:**
Users might not understand why certain inputs are rejected.

**Fix:**
Enhance `render_edit_prompt()` to show type (already in Phase 2c plan).

---

### ❌ No Tests for Editing

**Issue:**
No unit tests for `commit_editing()` function.

**Impact:**
Cannot verify fixes work, risk of regressions.

**Fix Required:**
Add comprehensive tests for:
- Editing each type (string, int, float, bool, null)
- **String style preservation** (Plain, Literal, Folded)
- Invalid input handling
- Round-trip editing (edit → save → load → verify)

---

## Test Coverage Gaps

### Missing Tests

1. `test_edit_plain_string()` - Edit Plain string, verify stays Plain
2. `test_edit_literal_string_preserves_style()` - ⚠️ **CRITICAL** - Verify Literal preserved
3. `test_edit_folded_string_preserves_style()` - ⚠️ **CRITICAL** - Verify Folded preserved
4. `test_edit_integer()` - Edit integer value
5. `test_edit_float()` - Edit float value
6. `test_edit_boolean_true_to_false()` - Edit boolean
7. `test_edit_invalid_number_fails()` - Validation error handling
8. `test_edit_invalid_boolean_fails()` - Validation error handling
9. `test_edit_null()` - Edit null value

### Existing Tests

✅ 7 tests in `editor::state::tests` (registers, yank/paste, delete, path)
❌ 0 tests for `commit_editing()`

---

## Phase 2c Scope Recommendation

### Must Fix (Critical)

1. **Fix string style preservation** (Bug fix, not new feature)
   - Update `commit_editing()` line 2872
   - Add test coverage
   - Verify round-trip editing works

2. **Add validation** (Better UX, prevent data loss)
   - Validate before committing
   - Keep edit buffer on error
   - Show clear error messages

3. **Add test coverage** (Quality assurance)
   - Test all scalar types
   - Test string style preservation
   - Test validation errors

### Nice to Have (Optional)

4. **Show type in prompt** (UX improvement)
   - `Edit [integer]: 42`
   - Helps users understand context

5. **Boolean toggle** (Convenience)
   - `t` key to flip true ↔ false
   - Quick editing without typing

### Defer to Later

6. **Multi-line input** (Complex, Phase 4)
   - Shift+Enter for newlines
   - Multi-line editor widget
   - Defer to Phase 4 (YAML-specific features)

7. **Type conversion** (Not in JSONQuill)
   - `:convert` command or similar
   - Out of scope, no JSONQuill equivalent

---

## Recommended Phase 2c Plan

### Task 1: Fix String Style Preservation ⚠️ CRITICAL

Fix the bug where Literal/Folded strings become Plain on edit.

**Files:** `src/editor/state.rs:2872`

**Change:**
```rust
// Before (BUG):
YamlValue::String(_) => YamlValue::String(YamlString::Plain(buffer_content))

// After (FIXED):
YamlValue::String(original_style) => {
    let new_string = match original_style {
        YamlString::Plain(_) => YamlString::Plain(buffer_content),
        YamlString::Literal(_) => YamlString::Literal(buffer_content),
        YamlString::Folded(_) => YamlString::Folded(buffer_content),
    };
    YamlValue::String(new_string)
}
```

**Tests:**
- Edit Plain string → stays Plain
- Edit Literal string → stays Literal
- Edit Folded string → stays Folded

---

### Task 2: Add Input Validation

Validate input before committing to avoid data loss.

**Files:** `src/editor/state.rs:2850-2924`

**Add validation function:**
```rust
fn validate_edit_input(buffer: &str, original_type: &YamlValue) -> Result<()> {
    match original_type {
        YamlValue::Number(_) => {
            if buffer.parse::<i64>().is_err() && buffer.parse::<f64>().is_err() {
                return Err(anyhow!("Invalid number: {}", buffer));
            }
        }
        YamlValue::Boolean(_) => {
            if !matches!(buffer, "true" | "false") {
                return Err(anyhow!("Boolean must be 'true' or 'false'"));
            }
        }
        _ => {} // Strings, null accept anything
    }
    Ok(())
}
```

**Tests:**
- Invalid number input rejected
- Invalid boolean input rejected
- Valid inputs accepted

---

### Task 3: Add Comprehensive Test Coverage

Write tests for all editing scenarios.

**Files:** `src/editor/state.rs` (test module)

**Minimum tests:**
- All scalar types (string, int, float, bool, null)
- String style preservation (Plain, Literal, Folded) ⚠️
- Validation (invalid inputs)
- Round-trip (edit → save → load)

---

### Task 4 (Optional): Show Type in Prompt

**Files:** `src/ui/edit_prompt.rs`, `src/editor/state.rs`

Enhance prompt to show type information.

---

### Task 5 (Optional): Boolean Toggle

**Files:** `src/input/handler.rs`, `src/input/keys.rs`

Add `t` key to toggle boolean values quickly.

---

## Success Criteria

Phase 2c is complete when:

1. ✅ String style preservation works (Literal/Folded preserved on edit)
2. ✅ Input validation prevents invalid edits
3. ✅ All editing tests pass (minimum 9 tests)
4. ✅ Round-trip editing preserves data and format
5. ✅ Zero clippy warnings
6. ✅ Manual testing confirms editing works for all types

## Estimated Effort

- **Task 1 (Critical bug fix):** 1-2 hours
- **Task 2 (Validation):** 1-2 hours
- **Task 3 (Tests):** 2-3 hours
- **Tasks 4-5 (Optional):** 2-3 hours

**Total:** 6-10 hours for must-fix items, 8-13 hours with optional features.

---

## Next Steps

**Recommend:**
1. User reviews this analysis
2. User decides which tasks are in scope for Phase 2c
3. Create focused implementation plan (smaller than original Phase 2c plan)
4. Execute and test
