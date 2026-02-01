//! YAML node representation with metadata tracking.
//!
//! This module provides the core data structures for representing YAML documents
//! in yamlquill. Each YAML value is wrapped in a `YamlNode` that tracks metadata
//! such as modification status and original formatting, enabling format-preserving
//! edits and efficient change tracking.
//!
//! # Example
//!
//! ```
//! use yamlquill::document::node::{YamlNode, YamlValue};
//!
//! // Create a simple string node
//! let mut node = YamlNode::new(YamlValue::String("hello".to_string()));
//! assert!(node.is_modified()); // New nodes are marked as modified
//!
//! // Create a complex nested structure
//! let object = YamlNode::new(YamlValue::Object(vec![
//!     ("name".to_string(), YamlNode::new(YamlValue::String("yamlquill".to_string()))),
//!     ("version".to_string(), YamlNode::new(YamlValue::Number(1.0))),
//! ]));
//!
//! // Modify a value
//! if let YamlValue::Object(ref mut fields) = node.value_mut() {
//!     fields.push(("key".to_string(), YamlNode::new(YamlValue::Null)));
//! }
//! ```

/// A byte range in the original YAML source.
///
/// TextSpan tracks the position of a node's text in the original YAML string,
/// enabling exact format preservation for unmodified nodes.
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct TextSpan {
    /// Start byte offset in original YAML
    pub start: usize,
    /// End byte offset in original YAML (exclusive)
    pub end: usize,
}

/// A YAML value without metadata.
///
/// This enum represents the core YAML types: objects, arrays, strings, numbers,
/// booleans, and null. Objects and arrays contain `YamlNode` instances to preserve
/// metadata throughout the tree structure.
#[derive(Debug, Clone, PartialEq)]
pub enum YamlValue {
    /// A YAML object containing key-value pairs
    Object(Vec<(String, YamlNode)>),
    /// A YAML array containing ordered values
    Array(Vec<YamlNode>),
    /// A YAML string
    String(String),
    /// A YAML number (represented as f64)
    Number(f64),
    /// A YAML boolean
    Boolean(bool),
    /// A YAML null value
    Null,
    /// A multi-document YAML file (each document is a YamlNode)
    MultiDoc(Vec<YamlNode>),
}

/// A YAML value wrapped with metadata for tracking changes and formatting.
///
/// `YamlNode` is the primary type used throughout yamlquill to represent YAML data.
/// It wraps a `YamlValue` with `NodeMetadata` to track whether the node has been
/// modified and preserve original formatting information for format-preserving edits.
#[derive(Debug, Clone, PartialEq)]
pub struct YamlNode {
    pub(crate) value: YamlValue,
    pub(crate) metadata: NodeMetadata,
}

/// Metadata associated with a YAML node.
///
/// This structure tracks information about a node beyond its value, including
/// whether it has been modified since loading and its byte position in the
/// original source for format preservation.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeMetadata {
    /// Byte range in the original YAML string (for unmodified nodes)
    pub text_span: Option<TextSpan>,
    /// Whether this node has been modified
    pub modified: bool,
}

impl YamlValue {
    /// Returns true if this value is an object.
    ///
    /// # Example
    ///
    /// ```
    /// use yamlquill::document::node::YamlValue;
    ///
    /// let obj = YamlValue::Object(vec![]);
    /// assert!(obj.is_object());
    ///
    /// let num = YamlValue::Number(42.0);
    /// assert!(!num.is_object());
    /// ```
    pub fn is_object(&self) -> bool {
        matches!(self, YamlValue::Object(_))
    }

    /// Returns true if this value is an array.
    ///
    /// # Example
    ///
    /// ```
    /// use yamlquill::document::node::YamlValue;
    ///
    /// let arr = YamlValue::Array(vec![]);
    /// assert!(arr.is_array());
    ///
    /// let num = YamlValue::Number(42.0);
    /// assert!(!num.is_array());
    /// ```
    pub fn is_array(&self) -> bool {
        matches!(self, YamlValue::Array(_))
    }

    /// Returns true if this value is a container (object, array, or multi-doc root).
    ///
    /// # Example
    ///
    /// ```
    /// use yamlquill::document::node::YamlValue;
    ///
    /// let obj = YamlValue::Object(vec![]);
    /// assert!(obj.is_container());
    ///
    /// let arr = YamlValue::Array(vec![]);
    /// assert!(arr.is_container());
    ///
    /// let num = YamlValue::Number(42.0);
    /// assert!(!num.is_container());
    /// ```
    pub fn is_container(&self) -> bool {
        matches!(
            self,
            YamlValue::Object(_) | YamlValue::Array(_) | YamlValue::MultiDoc(_)
        )
    }
}

impl YamlNode {
    /// Creates a new `YamlNode` with the given value.
    ///
    /// The node is marked as modified by default since it's newly created.
    /// The text_span field is set to None.
    ///
    /// # Example
    ///
    /// ```
    /// use yamlquill::document::node::{YamlNode, YamlValue};
    ///
    /// let node = YamlNode::new(YamlValue::Number(42.0));
    /// assert!(node.is_modified());
    /// ```
    pub fn new(value: YamlValue) -> Self {
        Self {
            value,
            metadata: NodeMetadata {
                text_span: None,
                modified: true,
            },
        }
    }

    /// Returns an immutable reference to the node's value.
    ///
    /// # Example
    ///
    /// ```
    /// use yamlquill::document::node::{YamlNode, YamlValue};
    ///
    /// let node = YamlNode::new(YamlValue::Boolean(true));
    /// assert!(matches!(node.value(), YamlValue::Boolean(true)));
    /// ```
    pub fn value(&self) -> &YamlValue {
        &self.value
    }

    /// Returns a mutable reference to the node's value.
    ///
    /// Calling this method automatically marks the node as modified,
    /// even if the value is not actually changed.
    ///
    /// # Example
    ///
    /// ```
    /// use yamlquill::document::node::{YamlNode, YamlValue};
    ///
    /// let mut node = YamlNode::new(YamlValue::String("old".to_string()));
    /// *node.value_mut() = YamlValue::String("new".to_string());
    /// assert!(node.is_modified());
    /// ```
    pub fn value_mut(&mut self) -> &mut YamlValue {
        self.metadata.modified = true;
        &mut self.value
    }

    /// Returns whether this node has been modified.
    ///
    /// A node is considered modified if it was newly created or if
    /// `value_mut()` has been called on it.
    ///
    /// # Example
    ///
    /// ```
    /// use yamlquill::document::node::{YamlNode, YamlValue};
    ///
    /// let node = YamlNode::new(YamlValue::Null);
    /// assert!(node.is_modified());
    /// ```
    pub fn is_modified(&self) -> bool {
        self.metadata.modified
    }
}

#[cfg(test)]
mod text_span_tests {
    use super::*;

    #[test]
    fn test_text_span_creation() {
        let span = TextSpan { start: 10, end: 25 };
        assert_eq!(span.start, 10);
        assert_eq!(span.end, 25);
    }

    #[test]
    fn test_text_span_equality() {
        let span1 = TextSpan { start: 5, end: 10 };
        let span2 = TextSpan { start: 5, end: 10 };
        let span3 = TextSpan { start: 5, end: 11 };

        assert_eq!(span1, span2);
        assert_ne!(span1, span3);
    }

    #[test]
    fn test_text_span_clone() {
        let span1 = TextSpan { start: 0, end: 100 };
        let span2 = span1;

        assert_eq!(span1, span2);
    }

    #[test]
    fn test_node_metadata_with_text_span() {
        let metadata = NodeMetadata {
            text_span: Some(TextSpan { start: 0, end: 10 }),
            modified: false,
        };

        assert!(metadata.text_span.is_some());
        assert_eq!(metadata.text_span.unwrap().start, 0);
        assert_eq!(metadata.text_span.unwrap().end, 10);
        assert!(!metadata.modified);
    }

    #[test]
    fn test_node_metadata_without_text_span() {
        let metadata = NodeMetadata {
            text_span: None,
            modified: true,
        };

        assert!(metadata.text_span.is_none());
        assert!(metadata.modified);
    }
}
