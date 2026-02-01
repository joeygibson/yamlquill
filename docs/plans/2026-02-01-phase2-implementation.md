# Phase 2: YAML Document Model Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix all compilation errors, implement complete YAML editing operations, improve display/navigation, achieve 60-70% test coverage

**Architecture:** Bottom-up approach - fix foundation (tree.rs tests), add value editing infrastructure, enhance display, comprehensive testing

**Tech Stack:** Rust 2021, serde_yaml 0.9, ratatui 0.29, indexmap 2.0

---

## Task 1: Fix tree.rs Test Compilation Errors

**Files:**
- Modify: `src/document/tree.rs` (tests section at bottom)

**Step 1: Fix String type mismatches in tree.rs tests**

Find all occurrences of `YamlValue::String("...".to_string())` and replace with `YamlValue::String(YamlString::Plain("...".to_string()))`.

Location: Line 367 in tree.rs (and any other test functions).

```rust
// Before:
let root = YamlNode::new(YamlValue::String("test".to_string()));

// After:
use crate::document::node::YamlString;
let root = YamlNode::new(YamlValue::String(YamlString::Plain("test".to_string())));
```

**Step 2: Add YamlString import to test module**

At the top of the `#[cfg(test)]` mod tests section in tree.rs, add:

```rust
use crate::document::node::{YamlString, YamlNumber};
```

**Step 3: Run tree.rs tests**

Run: `cargo test --lib tree`
Expected: Should have fewer errors (tree.rs specific errors fixed)

**Step 4: Commit tree.rs test fixes**

```bash
git add src/document/tree.rs
git commit -m "fix: update tree.rs tests for YamlString enum

- Replace String with YamlString::Plain in test fixtures
- Add YamlString/YamlNumber imports to test module

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 2: Fix registers.rs Test Compilation Errors

**Files:**
- Modify: `src/editor/registers.rs` (tests section)

**Step 1: Fix String type mismatches in registers.rs tests**

Find all `YamlValue::String("...".to_string())` and `YamlValue::Number(f64)` in the tests.

```rust
// Before:
let node = YamlNode::new(YamlValue::String("test".to_string()));
let node = YamlNode::new(YamlValue::Number(42.0));

// After:
use crate::document::node::{YamlString, YamlNumber};
let node = YamlNode::new(YamlValue::String(YamlString::Plain("test".to_string())));
let node = YamlNode::new(YamlValue::Number(YamlNumber::Float(42.0)));
```

**Step 2: Add imports to test module**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::node::{YamlString, YamlNumber};
    // ... rest of tests
}
```

**Step 3: Run registers tests**

Run: `cargo test --lib registers`
Expected: registers.rs tests compile

**Step 4: Commit**

```bash
git add src/editor/registers.rs
git commit -m "fix: update registers.rs tests for YAML types

- Use YamlString::Plain for string test fixtures
- Use YamlNumber::Float for number test fixtures
- Add type imports to test module

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 3: Fix yamlpath/evaluator.rs Test Errors

**Files:**
- Modify: `src/yamlpath/evaluator.rs` (tests section)

**Step 1: Fix type mismatches in evaluator tests**

Find all test assertions with String and Number types:

```rust
// Before:
assert_eq!(results[0].value(), &YamlValue::String("test".to_string()));
assert_eq!(results[1].value(), &YamlValue::Number(42.0));

// After:
use crate::document::node::{YamlString, YamlNumber};
assert_eq!(results[0].value(), &YamlValue::String(YamlString::Plain("test".to_string())));
assert_eq!(results[1].value(), &YamlValue::Number(YamlNumber::Float(42.0)));
```

**Step 2: Add imports**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::node::{YamlString, YamlNumber};
    // ... tests
}
```

**Step 3: Run yamlpath tests**

Run: `cargo test --lib yamlpath`
Expected: yamlpath tests compile

**Step 4: Commit**

```bash
git add src/yamlpath/evaluator.rs
git commit -m "fix: update yamlpath tests for YAML types

- Update test assertions to use YamlString::Plain
- Update number assertions to use YamlNumber::Float
- Add type imports

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 4: Fix UI Test Compilation Errors

**Files:**
- Modify: `src/ui/tree_view.rs` (tests section)
- Modify: `src/ui/status_line.rs` (tests section)

**Step 1: Fix tree_view.rs tests**

Update all String and Number usages in tree_view tests:

```rust
use crate::document::node::{YamlString, YamlNumber};

// In test fixtures:
YamlValue::String(YamlString::Plain("value".to_string()))
YamlValue::Number(YamlNumber::Integer(42))
```

**Step 2: Fix status_line.rs tests**

Same pattern - update String/Number types.

**Step 3: Run UI tests**

Run: `cargo test --lib ui`
Expected: UI tests compile

**Step 4: Commit**

```bash
git add src/ui/tree_view.rs src/ui/status_line.rs
git commit -m "fix: update UI tests for YAML types

- Update tree_view test fixtures
- Update status_line test fixtures
- Add YamlString/YamlNumber imports

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 5: Fix Remaining Compilation Errors

**Files:**
- Modify: `src/document/node.rs` (if has tests)
- Modify: `src/file/saver.rs` (if has tests)

**Step 1: Check for remaining errors**

Run: `cargo test 2>&1 | grep "error\[E0308\]" | wc -l`
Expected: Should be 0 or very few

**Step 2: Fix any remaining type mismatches**

Apply same pattern: String → YamlString::Plain, f64 → YamlNumber::Float or Integer

**Step 3: Run full test suite**

Run: `cargo test`
Expected: Should compile, tests may fail but no compilation errors

**Step 4: Commit**

```bash
git add src/document/node.rs src/file/saver.rs
git commit -m "fix: resolve remaining YAML type compilation errors

- Fix all String → YamlString::Plain conversions
- Fix all Number → YamlNumber conversions
- All tests now compile successfully

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 6: Run Full Test Suite Baseline

**Files:**
- None (validation step)

**Step 1: Run all tests**

Run: `cargo test`
Expected: Tests compile, capture pass/fail count

**Step 2: Document baseline**

Create a note of which tests pass vs fail. This is our baseline before implementing new features.

**Step 3: Run formatter and linter**

Run: `cargo fmt && cargo clippy -- -D warnings`
Expected: Code formatted, may have clippy warnings to fix

**Step 4: Fix critical clippy warnings**

If clippy shows errors (not warnings), fix them. Warnings can be addressed later.

**Step 5: Commit if changes made**

```bash
git add -A
git commit -m "chore: format code and fix clippy errors

- Run cargo fmt
- Fix critical clippy errors
- Establish clean baseline for Phase 2

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 7: Add YamlString Helper Methods

**Files:**
- Modify: `src/document/node.rs`

**Step 1: Implement Display trait for YamlString**

Add after the YamlString enum definition:

```rust
impl std::fmt::Display for YamlString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
```

**Step 2: Add is_multiline method**

```rust
impl YamlString {
    pub fn is_multiline(&self) -> bool {
        match self {
            YamlString::Plain(s) => s.contains('\n'),
            YamlString::Literal(_) | YamlString::Folded(_) => true,
        }
    }

    pub fn line_count(&self) -> usize {
        self.as_str().lines().count()
    }
}
```

**Step 3: Write tests for helper methods**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yaml_string_display() {
        let plain = YamlString::Plain("hello".to_string());
        assert_eq!(format!("{}", plain), "hello");
    }

    #[test]
    fn test_yaml_string_is_multiline() {
        let plain_single = YamlString::Plain("hello".to_string());
        assert!(!plain_single.is_multiline());

        let plain_multi = YamlString::Plain("hello\nworld".to_string());
        assert!(plain_multi.is_multiline());

        let literal = YamlString::Literal("hello".to_string());
        assert!(literal.is_multiline());
    }

    #[test]
    fn test_yaml_string_line_count() {
        let single = YamlString::Plain("hello".to_string());
        assert_eq!(single.line_count(), 1);

        let multi = YamlString::Plain("hello\nworld\ntest".to_string());
        assert_eq!(multi.line_count(), 3);
    }
}
```

**Step 4: Run tests**

Run: `cargo test node::tests`
Expected: New tests pass

**Step 5: Commit**

```bash
git add src/document/node.rs
git commit -m "feat: add helper methods to YamlString

- Implement Display trait for easy formatting
- Add is_multiline() to detect multi-line strings
- Add line_count() for display purposes
- Add comprehensive tests

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 8: Add YamlNumber Helper Methods

**Files:**
- Modify: `src/document/node.rs`

**Step 1: Implement Display trait for YamlNumber**

```rust
impl std::fmt::Display for YamlNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            YamlNumber::Integer(i) => write!(f, "{}", i),
            YamlNumber::Float(fl) => write!(f, "{}", fl),
        }
    }
}
```

**Step 2: Add is_integer method**

```rust
impl YamlNumber {
    pub fn is_integer(&self) -> bool {
        matches!(self, YamlNumber::Integer(_))
    }

    pub fn is_float(&self) -> bool {
        matches!(self, YamlNumber::Float(_))
    }
}
```

**Step 3: Write tests**

```rust
#[test]
fn test_yaml_number_display() {
    let int = YamlNumber::Integer(42);
    assert_eq!(format!("{}", int), "42");

    let float = YamlNumber::Float(42.5);
    assert_eq!(format!("{}", float), "42.5");
}

#[test]
fn test_yaml_number_type_checks() {
    let int = YamlNumber::Integer(42);
    assert!(int.is_integer());
    assert!(!int.is_float());

    let float = YamlNumber::Float(42.0);
    assert!(float.is_float());
    assert!(!float.is_integer());
}
```

**Step 4: Run tests**

Run: `cargo test node::tests`
Expected: Tests pass

**Step 5: Commit**

```bash
git add src/document/node.rs
git commit -m "feat: add helper methods to YamlNumber

- Implement Display trait
- Add is_integer() and is_float() type checks
- Add tests for number helpers

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 9: Implement Type Conversion Helpers

**Files:**
- Modify: `src/document/node.rs`

**Step 1: Add can_convert_type function**

```rust
impl YamlValue {
    /// Check if this value can be converted to the target type
    pub fn can_convert_to(&self, target_type: &str) -> bool {
        match target_type {
            "string" => true, // Everything can convert to string
            "number" => self.can_parse_as_number(),
            "bool" => self.can_parse_as_bool(),
            "null" => true, // Can always convert to null (deletion)
            _ => false,
        }
    }

    fn can_parse_as_number(&self) -> bool {
        match self {
            YamlValue::Number(_) => true,
            YamlValue::String(s) => {
                s.as_str().parse::<i64>().is_ok() || s.as_str().parse::<f64>().is_ok()
            }
            YamlValue::Bool(b) => true, // true=1, false=0
            _ => false,
        }
    }

    fn can_parse_as_bool(&self) -> bool {
        match self {
            YamlValue::Bool(_) => true,
            YamlValue::Number(n) => {
                // 0 or 1
                matches!(n, YamlNumber::Integer(0) | YamlNumber::Integer(1))
            }
            YamlValue::String(s) => {
                matches!(s.as_str(), "true" | "false" | "yes" | "no")
            }
            _ => false,
        }
    }
}
```

**Step 2: Add convert_to function**

```rust
impl YamlValue {
    /// Convert this value to the target type
    pub fn convert_to(&self, target_type: &str) -> Option<YamlValue> {
        match target_type {
            "string" => Some(YamlValue::String(YamlString::Plain(self.to_string()))),
            "number" => self.to_number(),
            "bool" => self.to_bool(),
            "null" => Some(YamlValue::Null),
            _ => None,
        }
    }

    fn to_number(&self) -> Option<YamlValue> {
        match self {
            YamlValue::Number(n) => Some(YamlValue::Number(n.clone())),
            YamlValue::String(s) => {
                if let Ok(i) = s.as_str().parse::<i64>() {
                    Some(YamlValue::Number(YamlNumber::Integer(i)))
                } else if let Ok(f) = s.as_str().parse::<f64>() {
                    Some(YamlValue::Number(YamlNumber::Float(f)))
                } else {
                    None
                }
            }
            YamlValue::Bool(b) => {
                Some(YamlValue::Number(YamlNumber::Integer(if *b { 1 } else { 0 })))
            }
            _ => None,
        }
    }

    fn to_bool(&self) -> Option<YamlValue> {
        match self {
            YamlValue::Bool(b) => Some(YamlValue::Bool(*b)),
            YamlValue::Number(YamlNumber::Integer(0)) => Some(YamlValue::Bool(false)),
            YamlValue::Number(YamlNumber::Integer(1)) => Some(YamlValue::Bool(true)),
            YamlValue::String(s) => match s.as_str() {
                "true" | "yes" => Some(YamlValue::Bool(true)),
                "false" | "no" => Some(YamlValue::Bool(false)),
                _ => None,
            },
            _ => None,
        }
    }
}

impl std::fmt::Display for YamlValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            YamlValue::Null => write!(f, "null"),
            YamlValue::Bool(b) => write!(f, "{}", b),
            YamlValue::Number(n) => write!(f, "{}", n),
            YamlValue::String(s) => write!(f, "{}", s),
            YamlValue::Array(_) => write!(f, "[array]"),
            YamlValue::Object(_) => write!(f, "{{object}}"),
            YamlValue::Alias(a) => write!(f, "*{}", a),
            YamlValue::MultiDoc(_) => write!(f, "[multi-doc]"),
        }
    }
}
```

**Step 3: Write comprehensive tests**

```rust
#[test]
fn test_can_convert_to_number() {
    let string_num = YamlValue::String(YamlString::Plain("42".to_string()));
    assert!(string_num.can_convert_to("number"));

    let string_text = YamlValue::String(YamlString::Plain("hello".to_string()));
    assert!(!string_text.can_convert_to("number"));

    let num = YamlValue::Number(YamlNumber::Integer(42));
    assert!(num.can_convert_to("number"));
}

#[test]
fn test_convert_string_to_number() {
    let string_int = YamlValue::String(YamlString::Plain("42".to_string()));
    let result = string_int.convert_to("number").unwrap();
    assert!(matches!(result, YamlValue::Number(YamlNumber::Integer(42))));

    let string_float = YamlValue::String(YamlString::Plain("42.5".to_string()));
    let result = string_float.convert_to("number").unwrap();
    assert!(matches!(result, YamlValue::Number(YamlNumber::Float(f)) if (f - 42.5).abs() < 0.001));
}

#[test]
fn test_convert_to_bool() {
    let string_true = YamlValue::String(YamlString::Plain("true".to_string()));
    let result = string_true.convert_to("bool").unwrap();
    assert!(matches!(result, YamlValue::Bool(true)));

    let num_one = YamlValue::Number(YamlNumber::Integer(1));
    let result = num_one.convert_to("bool").unwrap();
    assert!(matches!(result, YamlValue::Bool(true)));
}

#[test]
fn test_convert_to_string() {
    let num = YamlValue::Number(YamlNumber::Integer(42));
    let result = num.convert_to("string").unwrap();
    match result {
        YamlValue::String(s) => assert_eq!(s.as_str(), "42"),
        _ => panic!("Expected string"),
    }
}
```

**Step 4: Run tests**

Run: `cargo test node::tests`
Expected: All conversion tests pass

**Step 5: Commit**

```bash
git add src/document/node.rs
git commit -m "feat: implement type conversion for YAML values

- Add can_convert_to() to check if conversion is valid
- Add convert_to() to perform type conversions
- Support string→number, string→bool, number→bool conversions
- Implement Display trait for YamlValue
- Comprehensive conversion tests

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 10: Add Tree Navigation Helper Methods

**Files:**
- Modify: `src/document/tree.rs`

**Step 1: Implement get_parent_path**

```rust
impl YamlTree {
    /// Get the parent path of the given path
    /// Returns None if path is root or invalid
    pub fn get_parent_path(&self, path: &str) -> Option<String> {
        if path.is_empty() || path == "$" {
            return None;
        }

        // Find last separator
        if let Some(last_dot) = path.rfind('.') {
            Some(path[..last_dot].to_string())
        } else if let Some(last_bracket) = path.rfind('[') {
            if last_bracket == 0 {
                None // Root array
            } else {
                Some(path[..last_bracket - 1].to_string()) // Remove trailing .
            }
        } else {
            None
        }
    }

    /// Get depth of a path (number of nesting levels)
    pub fn get_depth(&self, path: &str) -> usize {
        if path.is_empty() || path == "$" {
            return 0;
        }

        path.chars().filter(|c| *c == '.' || *c == '[').count()
    }
}
```

**Step 2: Write tests**

```rust
#[test]
fn test_get_parent_path() {
    let tree = YamlTree::new(YamlNode::new(YamlValue::Null));

    assert_eq!(tree.get_parent_path("$"), None);
    assert_eq!(tree.get_parent_path("name"), None);
    assert_eq!(tree.get_parent_path("config.timeout"), Some("config".to_string()));
    assert_eq!(tree.get_parent_path("users[0]"), Some("users".to_string()));
    assert_eq!(tree.get_parent_path("users[0].name"), Some("users[0]".to_string()));
}

#[test]
fn test_get_depth() {
    let tree = YamlTree::new(YamlNode::new(YamlValue::Null));

    assert_eq!(tree.get_depth("$"), 0);
    assert_eq!(tree.get_depth("name"), 0);
    assert_eq!(tree.get_depth("config.timeout"), 1);
    assert_eq!(tree.get_depth("users[0].name"), 2);
}
```

**Step 3: Run tests**

Run: `cargo test tree::tests::test_get_parent_path tree::tests::test_get_depth`
Expected: Tests pass

**Step 4: Commit**

```bash
git add src/document/tree.rs
git commit -m "feat: add tree navigation helpers

- Implement get_parent_path() for upward navigation
- Implement get_depth() for nesting level calculation
- Add tests for navigation helpers

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 11: Verify All Tests Pass

**Files:**
- None (verification step)

**Step 1: Run full test suite**

Run: `cargo test`
Expected: All tests compile and run

**Step 2: Check test results**

Count passing vs failing tests. Document the current state.

**Step 3: Run formatter and clippy**

Run: `cargo fmt && cargo clippy -- -D warnings`
Expected: Clean formatting, no clippy errors

**Step 4: Check build in release mode**

Run: `cargo build --release`
Expected: Successful build

**Step 5: Create checkpoint tag**

```bash
git tag -a phase2-checkpoint1 -m "Phase 2 Checkpoint 1: All compilation errors fixed

- Fixed 165 type mismatch errors
- Added YAML type helper methods
- Implemented type conversion infrastructure
- Added tree navigation helpers
- All tests compile successfully"
```

---

## Success Criteria

This implementation plan is complete when:

- ✅ Zero compilation errors (`cargo build` succeeds)
- ✅ All YamlString/YamlNumber type mismatches fixed
- ✅ Helper methods added (Display, is_multiline, conversions)
- ✅ Type conversion infrastructure works (can_convert_to, convert_to)
- ✅ Tree navigation helpers implemented (get_parent_path, get_depth)
- ✅ All tests compile (may have failures, but no compilation errors)
- ✅ Code formatted and passes clippy
- ✅ Checkpoint tag created

## Next Steps

After completing this plan:

1. Continue with Track 2: Value Editing Infrastructure
2. Implement edit_prompt.rs enhancements for multi-line strings
3. Add `:convert` command to editor
4. Build out comprehensive test suite

## Notes

- This plan focuses on Phase 2a: Fixing test compilation errors and adding foundation
- Phase 2b-2f will be implemented in subsequent plans
- Keep commits focused and atomic
- Run tests frequently to catch regressions early
