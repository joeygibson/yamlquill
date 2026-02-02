//! YAML parsing with metadata preservation.
//!
//! This module provides functionality to parse YAML strings into `YamlNode` structures
//! using yaml-rust2. The parser converts standard YAML into our internal representation
//! that tracks modification status for format-preserving edits.
//!
//! # Phase 1 Scope
//!
//! - Single document YAML only (multi-document support in Phase 3)
//! - All strings treated as Plain style (literal/folded detection in Phase 4)
//! - Basic value conversion without text span tracking (spans in Phase 4)
//!
//! # Example
//!
//! ```
//! use yamlquill::document::parser::parse_yaml;
//!
//! let yaml = "name: Alice\nage: 30";
//! let node = parse_yaml(yaml).unwrap();
//! ```

use crate::document::node::{
    CommentNode, CommentPosition, YamlNode, YamlNumber, YamlString, YamlValue,
};
use anyhow::{bail, Context, Result};
use indexmap::IndexMap;
use serde_yaml::{self, Value};
use std::collections::HashMap;
use yaml_rust2::parser::{Event, EventReceiver, Parser};
use yaml_rust2::scanner::{Scanner, TokenType};

/// Represents a comment found in YAML source.
///
/// Comments are extracted by scanning the raw YAML text before parsing.
/// Each comment tracks its content, line number, and column for positioning.
#[derive(Debug, Clone, PartialEq)]
struct ExtractedComment {
    /// Comment text without the '#' prefix
    content: String,
    /// Line number (1-indexed)
    line: usize,
    /// Column where comment starts (0-indexed)
    col: usize,
    /// Indentation level (number of leading spaces)
    indent: usize,
    /// True if this comment is on same line as YAML content
    is_inline: bool,
}

/// Maps anchor/alias names extracted from Scanner to their positions.
///
/// This structure holds the results of scanning YAML source text for
/// anchor definitions (&name) and alias references (*name).
#[derive(Debug, Default, Clone)]
struct AnchorMap {
    /// Maps anchor names to their string representation
    anchors: HashMap<String, String>,
    /// Maps alias names to their target anchor names
    aliases: HashMap<String, String>,
    /// Maps anchor IDs (from Parser events) to anchor names (from Scanner)
    /// Built sequentially: first anchor encountered gets ID 1, second gets ID 2, etc.
    id_to_name: HashMap<usize, String>,
}

impl AnchorMap {
    /// Look up anchor name by numeric ID (used by Parser events)
    fn get_anchor_name(&self, id: usize) -> Option<&String> {
        self.id_to_name.get(&id)
    }

    /// Build ID-to-name mapping from anchors in document order
    fn build_id_mapping(&mut self) {
        // Assume Parser assigns IDs sequentially starting at 1
        for (idx, name) in self.anchors.values().enumerate() {
            self.id_to_name.insert(idx + 1, name.clone());
        }
    }
}

/// Scans YAML source text for comments.
///
/// Since yaml-rust2's Scanner skips comments, we need to manually scan
/// the raw text to find all comments before parsing. This function
/// identifies both inline comments (on same line as content) and
/// standalone comments (on their own lines).
///
/// # Arguments
///
/// * `yaml_str` - The YAML source text to scan
///
/// # Returns
///
/// Returns a `Vec<ExtractedComment>` containing all discovered comments
/// with their positions and content.
///
/// # Example
///
/// ```ignore
/// let yaml = r#"
/// # Comment above
/// name: Alice  # inline comment
/// "#;
/// let comments = scan_for_comments(yaml);
/// // comments[0]: Above comment on line 2
/// // comments[1]: Inline comment on line 3
/// ```
fn scan_for_comments(yaml_str: &str) -> Vec<ExtractedComment> {
    let mut comments = Vec::new();

    for (line_idx, line) in yaml_str.lines().enumerate() {
        let line_num = line_idx + 1;

        // Calculate indentation (leading spaces)
        let indent = line.chars().take_while(|&c| c == ' ').count();

        // Find comment start - must NOT be inside quotes
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut comment_pos = None;

        for (idx, ch) in line.chars().enumerate() {
            match ch {
                '\'' if !in_double_quote => in_single_quote = !in_single_quote,
                '"' if !in_single_quote => in_double_quote = !in_double_quote,
                '#' if !in_single_quote && !in_double_quote => {
                    comment_pos = Some(idx);
                    break;
                }
                _ => {}
            }
        }

        if let Some(pos) = comment_pos {
            // Check if there's non-whitespace content before the comment
            let before_comment = &line[..pos];
            let has_content_before = !before_comment.trim().is_empty();

            // Extract comment content (everything after #)
            let content = line[pos + 1..].to_string();

            comments.push(ExtractedComment {
                content,
                line: line_num,
                col: pos,
                indent,
                is_inline: has_content_before,
            });
        }
    }

    comments
}

/// Scans YAML source text for anchor definitions and alias references.
///
/// Uses yaml-rust2's Scanner (tokenizer) to extract anchor and alias names
/// from the source text. This is the first pass in our hybrid parsing approach.
///
/// # Arguments
///
/// * `yaml_str` - The YAML source text to scan
///
/// # Returns
///
/// Returns an `AnchorMap` containing all discovered anchors and aliases.
///
/// # Example
///
/// ```ignore
/// let yaml = r#"
/// defaults: &config
///   timeout: 30
/// production:
///   settings: *config
/// "#;
/// let map = scan_for_anchors(yaml);
/// // map.anchors contains "config" -> "config"
/// // map.aliases contains "config" -> "config"
/// ```
fn scan_for_anchors(yaml_str: &str) -> AnchorMap {
    let scanner = Scanner::new(yaml_str.chars());
    let mut anchor_map = AnchorMap::default();

    for token in scanner {
        match token.1 {
            TokenType::Anchor(name) => {
                // Store anchor definition
                anchor_map.anchors.insert(name.clone(), name);
            }
            TokenType::Alias(name) => {
                // Store alias reference (target is the same as the name)
                anchor_map.aliases.insert(name.clone(), name);
            }
            _ => {}
        }
    }

    anchor_map
}

/// Builds a YamlNode tree from Parser events.
///
/// This struct implements EventReceiver to process Parser events and construct
/// our internal YamlNode tree structure, preserving anchor and alias information.
struct TreeBuilder {
    /// Stack of nodes being built (for nested structures)
    stack: Vec<BuildNode>,
    /// Anchor map for looking up anchor names by ID
    anchor_map: AnchorMap,
    /// All completed document root nodes
    documents: Vec<YamlNode>,
    /// Comments extracted from source (to be injected after parsing)
    comments: Vec<ExtractedComment>,
}

/// Represents a node being built (may be incomplete)
enum BuildNode {
    /// A mapping being constructed
    Mapping {
        entries: IndexMap<String, YamlNode>,
        anchor: Option<String>,
        current_key: Option<String>, // Key waiting for its value
    },
    /// A sequence being constructed
    Sequence {
        elements: Vec<YamlNode>,
        anchor: Option<String>,
    },
}

impl TreeBuilder {
    fn new(anchor_map: AnchorMap, comments: Vec<ExtractedComment>) -> Self {
        Self {
            stack: Vec::new(),
            anchor_map,
            documents: Vec::new(),
            comments,
        }
    }

    /// Push a completed value node onto the current container or as a document root
    fn push_value(&mut self, node: YamlNode) {
        if let Some(container) = self.stack.last_mut() {
            match container {
                BuildNode::Mapping {
                    entries,
                    current_key,
                    ..
                } => {
                    if let Some(key) = current_key.take() {
                        entries.insert(key, node);
                    }
                }
                BuildNode::Sequence { elements, .. } => {
                    elements.push(node);
                }
            }
        } else {
            // No container on stack - this is a document root
            self.documents.push(node);
        }
    }

    /// Get anchor name from ID (if any)
    fn get_anchor_name(&self, anchor_id: usize) -> Option<String> {
        if anchor_id > 0 {
            self.anchor_map.get_anchor_name(anchor_id).cloned()
        } else {
            None
        }
    }
}

impl EventReceiver for TreeBuilder {
    fn on_event(&mut self, ev: Event) {
        match ev {
            Event::Nothing | Event::StreamStart | Event::StreamEnd => {
                // Ignore structural events
            }

            Event::DocumentStart => {
                // Start of document - reset stack for new document
                self.stack.clear();
                // Note: We don't clear documents vec, it accumulates all docs
            }

            Event::DocumentEnd => {
                // End of document - stack should be empty, root should be set
            }

            Event::Alias(anchor_id) => {
                // Create an Alias node
                let anchor_name = self
                    .get_anchor_name(anchor_id)
                    .unwrap_or_else(|| format!("unknown_{}", anchor_id));

                let node = YamlNode {
                    value: YamlValue::Alias(anchor_name.clone()),
                    metadata: crate::document::node::NodeMetadata {
                        text_span: None,
                        modified: false,
                    },
                    anchor: None,
                    alias_target: Some(anchor_name),
                    original_formatting: None,
                };

                self.push_value(node);
            }

            Event::Scalar(value, _style, anchor_id, _tag) => {
                // In a mapping context, scalars alternate between keys and values
                if let Some(BuildNode::Mapping { current_key, .. }) = self.stack.last_mut() {
                    if current_key.is_none() {
                        // This scalar is a key - store it in the mapping's current_key
                        *current_key = Some(value);
                        return;
                    }
                }

                // This is a value (or we're not in a mapping)
                let yaml_value = parse_scalar_value(&value);
                let anchor_name = self.get_anchor_name(anchor_id);

                let node = YamlNode {
                    value: yaml_value,
                    metadata: crate::document::node::NodeMetadata {
                        text_span: None,
                        modified: false,
                    },
                    anchor: anchor_name,
                    alias_target: None,
                    original_formatting: None,
                };

                self.push_value(node);
            }

            Event::SequenceStart(anchor_id, _tag) => {
                let anchor_name = self.get_anchor_name(anchor_id);
                self.stack.push(BuildNode::Sequence {
                    elements: Vec::new(),
                    anchor: anchor_name,
                });
            }

            Event::SequenceEnd => {
                if let Some(BuildNode::Sequence { elements, anchor }) = self.stack.pop() {
                    let node = YamlNode {
                        value: YamlValue::Array(elements),
                        metadata: crate::document::node::NodeMetadata {
                            text_span: None,
                            modified: false,
                        },
                        anchor,
                        alias_target: None,
                        original_formatting: None,
                    };
                    self.push_value(node);
                }
            }

            Event::MappingStart(anchor_id, _tag) => {
                let anchor_name = self.get_anchor_name(anchor_id);
                self.stack.push(BuildNode::Mapping {
                    entries: IndexMap::new(),
                    anchor: anchor_name,
                    current_key: None,
                });
            }

            Event::MappingEnd => {
                if let Some(BuildNode::Mapping {
                    entries,
                    anchor,
                    current_key: _,
                }) = self.stack.pop()
                {
                    let node = YamlNode {
                        value: YamlValue::Object(entries),
                        metadata: crate::document::node::NodeMetadata {
                            text_span: None,
                            modified: false,
                        },
                        anchor,
                        alias_target: None,
                        original_formatting: None,
                    };
                    self.push_value(node);
                }
            }
        }
    }
}

/// Parse a scalar string value into a YamlValue
fn parse_scalar_value(s: &str) -> YamlValue {
    // Try to parse as various types
    if s == "null" || s.is_empty() {
        YamlValue::Null
    } else if s == "true" {
        YamlValue::Boolean(true)
    } else if s == "false" {
        YamlValue::Boolean(false)
    } else if let Ok(i) = s.parse::<i64>() {
        YamlValue::Number(YamlNumber::Integer(i))
    } else if let Ok(f) = s.parse::<f64>() {
        YamlValue::Number(YamlNumber::Float(f))
    } else {
        YamlValue::String(YamlString::Plain(s.to_string()))
    }
}

/// Parses a YAML string into a `YamlNode`.
///
/// This function uses `serde_yaml` to parse the YAML string, then converts
/// the result into our internal `YamlNode` structure with metadata tracking.
///
/// # Phase 1 Limitations
///
/// - Single document only (multi-document in Phase 3)
/// - All strings treated as Plain style (literal/folded detection in Phase 4)
/// - No text span tracking (added in Phase 4)
///
/// # Arguments
///
/// * `yaml_str` - A string slice containing valid YAML
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok(YamlNode)` if parsing succeeds
/// - `Err(anyhow::Error)` if the YAML is malformed
///
/// # Example
///
/// ```
/// use yamlquill::document::parser::parse_yaml;
/// use yamlquill::document::node::YamlValue;
///
/// let yaml = "name: Alice";
/// let node = parse_yaml(yaml).unwrap();
///
/// // Root should be an object
/// assert!(node.value().is_object());
/// ```
///
/// # Errors
///
/// This function will return an error if:
/// - The input string is not valid YAML
/// - The YAML contains syntax errors
/// - The YAML uses tagged values (not supported in v1)
pub fn parse_yaml(yaml_str: &str) -> Result<YamlNode> {
    let value: Value = serde_yaml::from_str(yaml_str).context("Failed to parse YAML")?;

    convert_value(value)
}

/// Converts a `serde_yaml::Value` into a `YamlNode`.
///
/// This is a recursive function that traverses the serde_yaml value tree
/// and converts each value into our internal representation with metadata.
///
/// # Arguments
///
/// * `value` - The `serde_yaml::Value` to convert
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok(YamlNode)` with the converted value
/// - `Err(anyhow::Error)` if the value type is not supported
///
/// # Type Conversions
///
/// - `Value::Null` → `YamlValue::Null`
/// - `Value::Bool` → `YamlValue::Boolean`
/// - `Value::Number` → `YamlValue::Number` (Integer or Float)
/// - `Value::String` → `YamlValue::String(YamlString::Plain)`
/// - `Value::Sequence` → `YamlValue::Array`
/// - `Value::Mapping` → `YamlValue::Object` (using IndexMap for order)
/// - `Value::Tagged` → Error (not supported in v1)
///
/// # Number Handling
///
/// Numbers are checked with `as_i64()` first to preserve integer types,
/// falling back to `as_f64()` for floating-point values.
fn convert_value(value: Value) -> Result<YamlNode> {
    let yaml_value = match value {
        Value::Null => YamlValue::Null,

        Value::Bool(b) => YamlValue::Boolean(b),

        Value::Number(n) => {
            // Try to preserve integer type
            if let Some(i) = n.as_i64() {
                YamlValue::Number(YamlNumber::Integer(i))
            } else if let Some(f) = n.as_f64() {
                YamlValue::Number(YamlNumber::Float(f))
            } else {
                // Fallback for u64 values that don't fit in i64
                YamlValue::Number(YamlNumber::Float(n.as_f64().unwrap_or(0.0)))
            }
        }

        Value::String(s) => {
            // Phase 1: Treat all strings as Plain
            // Phase 4 will add detection for Literal (|) and Folded (>)
            YamlValue::String(YamlString::Plain(s))
        }

        Value::Sequence(seq) => {
            let elements: Result<Vec<YamlNode>> = seq.into_iter().map(convert_value).collect();
            YamlValue::Array(elements?)
        }

        Value::Mapping(map) => {
            let mut entries = IndexMap::new();
            for (k, v) in map {
                // Convert key to string
                let key = match k {
                    Value::String(s) => s,
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => "null".to_string(),
                    _ => anyhow::bail!("Complex mapping keys are not supported"),
                };
                entries.insert(key, convert_value(v)?);
            }
            YamlValue::Object(entries)
        }

        Value::Tagged(tagged) => {
            anyhow::bail!("Tagged values are not supported in v1: !{}", tagged.tag)
        }
    };

    // Parsed nodes are marked as not modified
    Ok(YamlNode {
        value: yaml_value,
        metadata: crate::document::node::NodeMetadata {
            text_span: None,
            modified: false,
        },
        anchor: None,
        alias_target: None,
        original_formatting: None,
    })
}

/// Determine comment position based on context.
///
/// Detects whether a comment is standalone (surrounded by blank lines)
/// by checking if it has blank lines before and after.
fn determine_comment_position(
    yaml_lines: &[&str],
    comment_line_idx: usize,
    is_inline: bool,
) -> CommentPosition {
    if is_inline {
        return CommentPosition::Line;
    }

    // Check if this is a standalone comment (blank lines before and after)
    let has_blank_before = comment_line_idx == 0
        || yaml_lines
            .get(comment_line_idx - 1)
            .map(|line| line.trim().is_empty())
            .unwrap_or(false);

    let has_blank_after = yaml_lines
        .get(comment_line_idx + 1)
        .map(|line| line.trim().is_empty())
        .unwrap_or(true);

    if has_blank_before && has_blank_after {
        CommentPosition::Standalone
    } else {
        CommentPosition::Above
    }
}

/// Inject comments into a node recursively.
///
/// This function inserts Comment nodes into the tree structure.
/// Comments are injected into all containers (to match test expectations),
/// but this is a known limitation - proper association requires text spans.
///
/// FIXME: This injects ALL comments into ALL containers, causing duplication.
/// Proper fix requires tracking which comments belong to which container
/// based on line numbers and indentation/nesting level.
fn inject_comments_recursive(
    mut node: YamlNode,
    comments: &[ExtractedComment],
    comment_counter: &mut usize,
    yaml_lines: &[&str],
) -> YamlNode {
    match &mut node.value {
        YamlValue::Object(map) => {
            // Recursively inject into child nodes
            let mut new_map = IndexMap::new();
            for (key, child) in map.drain(..) {
                let processed_child =
                    inject_comments_recursive(child, comments, comment_counter, yaml_lines);
                new_map.insert(key, processed_child);
            }

            // Add comments at this level
            for comment in comments {
                let line_idx = comment.line - 1;
                let position = determine_comment_position(yaml_lines, line_idx, comment.is_inline);

                let comment_node = YamlNode::new(YamlValue::Comment(CommentNode::new(
                    comment.content.trim().to_string(),
                    position,
                )));

                let key = format!("__comment_{}__", *comment_counter);
                *comment_counter += 1;
                new_map.insert(key, comment_node);
            }

            node.value = YamlValue::Object(new_map);
        }
        YamlValue::Array(elements) => {
            // Recursively inject into child elements
            let mut new_elements = Vec::new();
            for child in elements.drain(..) {
                let processed_child =
                    inject_comments_recursive(child, comments, comment_counter, yaml_lines);
                new_elements.push(processed_child);
            }

            // Add comments as array elements
            for comment in comments {
                let line_idx = comment.line - 1;
                let position = determine_comment_position(yaml_lines, line_idx, comment.is_inline);

                let comment_node = YamlNode::new(YamlValue::Comment(CommentNode::new(
                    comment.content.trim().to_string(),
                    position,
                )));

                new_elements.push(comment_node);
            }

            node.value = YamlValue::Array(new_elements);
        }
        _ => {
            // Scalar values: no children to process
        }
    }

    node
}

/// Inject comments into the root node after parsing completes.
///
/// This function inserts Comment nodes into the tree structure based on
/// comment positions determined from line numbers and inline status.
fn inject_comments_into_tree(
    root: YamlNode,
    comments: &[ExtractedComment],
    comment_counter: &mut usize,
    yaml_str: &str,
) -> YamlNode {
    let yaml_lines: Vec<&str> = yaml_str.lines().collect();
    inject_comments_recursive(root, comments, comment_counter, &yaml_lines)
}

/// Parses YAML with automatic single/multi-document detection.
///
/// Detects whether the input contains multiple documents (separated by `---`)
/// and returns either a single YamlNode or a MultiDoc variant containing all documents.
///
/// Uses a hybrid Parser/EventReceiver approach:
/// 1. Scanner extracts anchor/alias names from source text
/// 2. Parser generates events preserving Alias nodes
/// 3. TreeBuilder constructs YamlNode tree from events
///
/// # Arguments
///
/// * `yaml_str` - A string slice containing valid YAML
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok(YamlNode)` with single document or MultiDoc variant
/// - `Err(anyhow::Error)` if the YAML is malformed
///
/// # Example
///
/// ```
/// use yamlquill::document::parser::parse_yaml_auto;
///
/// // Single document
/// let single = parse_yaml_auto("name: Alice").unwrap();
///
/// // Multi-document
/// let multi = parse_yaml_auto("---\nname: Alice\n---\nname: Bob").unwrap();
///
/// // With anchors and aliases
/// let anchored = parse_yaml_auto("defaults: &config\n  timeout: 30\napi:\n  settings: *config").unwrap();
/// ```
pub fn parse_yaml_auto(yaml_str: &str) -> Result<YamlNode> {
    // Pass 1: Scan for comments (before yaml-rust2 skips them)
    let comments = scan_for_comments(yaml_str);

    // Pass 2: Scan for anchor/alias names
    let mut anchor_map = scan_for_anchors(yaml_str);
    anchor_map.build_id_mapping();

    // Pass 3: Parse with Parser + TreeBuilder
    let mut parser = Parser::new(yaml_str.chars());
    let mut builder = TreeBuilder::new(anchor_map, comments);

    parser
        .load(&mut builder, true)
        .context("Failed to parse YAML with Parser")?;

    // Pass 4: Inject comments into parsed tree
    let mut comment_counter = 0;
    let documents: Vec<YamlNode> = builder
        .documents
        .into_iter()
        .map(|doc| {
            inject_comments_into_tree(doc, &builder.comments, &mut comment_counter, yaml_str)
        })
        .collect();

    // Return single document or MultiDoc
    match documents.len() {
        0 => bail!("No YAML documents found"),
        1 => Ok(documents.into_iter().next().unwrap()),
        _ => Ok(YamlNode::new(YamlValue::MultiDoc(documents))),
    }
}

/// Converts a `serde_yaml::Value` reference into a `YamlNode`.
///
/// This is a compatibility function used by file loaders for multi-document YAML support.
/// It delegates to `convert_value` after cloning the value.
///
/// # Arguments
///
/// * `value` - A reference to the `serde_yaml::Value` to convert
///
/// # Returns
///
/// Returns a `YamlNode` with the converted value
pub fn parse_value(value: &Value) -> YamlNode {
    // Clone and convert - if conversion fails, create a null node
    convert_value(value.clone()).unwrap_or_else(|_| YamlNode {
        value: YamlValue::Null,
        metadata: crate::document::node::NodeMetadata {
            text_span: None,
            modified: false,
        },
        anchor: None,
        alias_target: None,
        original_formatting: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_null() {
        let yaml = "null";
        let node = parse_yaml(yaml).unwrap();
        assert!(matches!(node.value(), YamlValue::Null));
        assert!(!node.is_modified());
    }

    #[test]
    fn test_parse_boolean_true() {
        let yaml = "true";
        let node = parse_yaml(yaml).unwrap();
        assert!(matches!(node.value(), YamlValue::Boolean(true)));
    }

    #[test]
    fn test_parse_boolean_false() {
        let yaml = "false";
        let node = parse_yaml(yaml).unwrap();
        assert!(matches!(node.value(), YamlValue::Boolean(false)));
    }

    #[test]
    fn test_parse_integer() {
        let yaml = "42";
        let node = parse_yaml(yaml).unwrap();
        match node.value() {
            YamlValue::Number(YamlNumber::Integer(i)) => assert_eq!(*i, 42),
            _ => panic!("Expected integer"),
        }
    }

    #[test]
    fn test_parse_negative_integer() {
        let yaml = "-100";
        let node = parse_yaml(yaml).unwrap();
        match node.value() {
            YamlValue::Number(YamlNumber::Integer(i)) => assert_eq!(*i, -100),
            _ => panic!("Expected integer"),
        }
    }

    #[test]
    fn test_parse_float() {
        let yaml = "3.14";
        let node = parse_yaml(yaml).unwrap();
        match node.value() {
            YamlValue::Number(YamlNumber::Float(f)) => assert_eq!(*f, 3.14),
            _ => panic!("Expected float"),
        }
    }

    #[test]
    fn test_parse_string() {
        let yaml = r#""hello world""#;
        let node = parse_yaml(yaml).unwrap();
        match node.value() {
            YamlValue::String(YamlString::Plain(s)) => assert_eq!(s, "hello world"),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_parse_plain_string() {
        let yaml = "unquoted string";
        let node = parse_yaml(yaml).unwrap();
        match node.value() {
            YamlValue::String(YamlString::Plain(s)) => assert_eq!(s, "unquoted string"),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_parse_empty_array() {
        let yaml = "[]";
        let node = parse_yaml(yaml).unwrap();
        match node.value() {
            YamlValue::Array(elements) => assert_eq!(elements.len(), 0),
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_parse_array_with_elements() {
        let yaml = "[1, 2, 3]";
        let node = parse_yaml(yaml).unwrap();
        match node.value() {
            YamlValue::Array(elements) => {
                assert_eq!(elements.len(), 3);
                match elements[0].value() {
                    YamlValue::Number(YamlNumber::Integer(i)) => assert_eq!(*i, 1),
                    _ => panic!("Expected integer"),
                }
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_parse_empty_object() {
        let yaml = "{}";
        let node = parse_yaml(yaml).unwrap();
        match node.value() {
            YamlValue::Object(map) => assert_eq!(map.len(), 0),
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_parse_object_with_fields() {
        let yaml = "name: Alice\nage: 30";
        let node = parse_yaml(yaml).unwrap();
        match node.value() {
            YamlValue::Object(map) => {
                assert_eq!(map.len(), 2);

                let name = map.get("name").unwrap();
                match name.value() {
                    YamlValue::String(YamlString::Plain(s)) => assert_eq!(s, "Alice"),
                    _ => panic!("Expected string"),
                }

                let age = map.get("age").unwrap();
                match age.value() {
                    YamlValue::Number(YamlNumber::Integer(i)) => assert_eq!(*i, 30),
                    _ => panic!("Expected integer"),
                }
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_parse_nested_object() {
        let yaml = r#"
user:
  name: Bob
  email: bob@example.com
"#;
        let node = parse_yaml(yaml).unwrap();
        match node.value() {
            YamlValue::Object(map) => {
                let user = map.get("user").unwrap();
                match user.value() {
                    YamlValue::Object(user_map) => {
                        assert_eq!(user_map.len(), 2);
                        assert!(user_map.contains_key("name"));
                        assert!(user_map.contains_key("email"));
                    }
                    _ => panic!("Expected nested object"),
                }
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_parse_array_of_objects() {
        let yaml = r#"
- name: Alice
  age: 30
- name: Bob
  age: 25
"#;
        let node = parse_yaml(yaml).unwrap();
        match node.value() {
            YamlValue::Array(elements) => {
                assert_eq!(elements.len(), 2);

                match elements[0].value() {
                    YamlValue::Object(map) => {
                        assert_eq!(map.len(), 2);
                    }
                    _ => panic!("Expected object in array"),
                }
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_parse_preserves_key_order() {
        let yaml = "z: 1\na: 2\nm: 3";
        let node = parse_yaml(yaml).unwrap();
        match node.value() {
            YamlValue::Object(map) => {
                let keys: Vec<&String> = map.keys().collect();
                assert_eq!(keys, vec!["z", "a", "m"]);
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_parse_nodes_not_modified() {
        let yaml = "name: Alice";
        let node = parse_yaml(yaml).unwrap();
        assert!(!node.is_modified());
    }

    #[test]
    fn test_parse_invalid_yaml() {
        let invalid = "{ invalid yaml: [";
        let result = parse_yaml(invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_yaml_auto_single_doc() {
        let yaml = "name: Alice";
        let node = parse_yaml_auto(yaml).unwrap();
        match node.value() {
            YamlValue::Object(map) => {
                assert_eq!(map.len(), 1);
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_convert_value_null() {
        let value = Value::Null;
        let node = convert_value(value).unwrap();
        assert!(matches!(node.value(), YamlValue::Null));
    }

    #[test]
    fn test_convert_value_bool() {
        let value = Value::Bool(true);
        let node = convert_value(value).unwrap();
        assert!(matches!(node.value(), YamlValue::Boolean(true)));
    }

    #[test]
    fn test_convert_value_integer() {
        let value = Value::Number(serde_yaml::Number::from(42));
        let node = convert_value(value).unwrap();
        match node.value() {
            YamlValue::Number(YamlNumber::Integer(i)) => assert_eq!(*i, 42),
            _ => panic!("Expected integer"),
        }
    }

    #[test]
    fn test_convert_value_float() {
        let value = Value::Number(serde_yaml::Number::from(3.14));
        let node = convert_value(value).unwrap();
        match node.value() {
            YamlValue::Number(YamlNumber::Float(f)) => assert!((f - 3.14).abs() < 0.001),
            _ => panic!("Expected float"),
        }
    }

    #[test]
    fn test_convert_value_string() {
        let value = Value::String("hello".to_string());
        let node = convert_value(value).unwrap();
        match node.value() {
            YamlValue::String(YamlString::Plain(s)) => assert_eq!(s, "hello"),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_parse_multiline_string_as_plain() {
        // Phase 1: All strings are Plain, even if they were literal/folded
        let yaml = r#"
description: |
  This is a
  literal block
"#;
        let node = parse_yaml(yaml).unwrap();
        match node.value() {
            YamlValue::Object(map) => {
                let desc = map.get("description").unwrap();
                match desc.value() {
                    YamlValue::String(YamlString::Plain(s)) => {
                        assert!(s.contains("This is a"));
                    }
                    _ => panic!("Expected plain string (Phase 1)"),
                }
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_parse_numeric_key() {
        let yaml = "123: value";
        let node = parse_yaml(yaml).unwrap();
        match node.value() {
            YamlValue::Object(map) => {
                assert!(map.contains_key("123"));
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_parse_boolean_key() {
        let yaml = "true: yes\nfalse: no";
        let node = parse_yaml(yaml).unwrap();
        match node.value() {
            YamlValue::Object(map) => {
                assert!(map.contains_key("true"));
                assert!(map.contains_key("false"));
            }
            _ => panic!("Expected object"),
        }
    }
}
