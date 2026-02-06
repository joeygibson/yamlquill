//! Tree-based navigation for YAML documents.
//!
//! This module provides the `YamlTree` type for navigating YAML structures using
//! path-based indexing. It enables traversal of nested objects and arrays by
//! specifying a sequence of indices that represent the path from the root to a
//! target node.
//!
//! # Example
//!
//! ```
//! use yamlquill::document::tree::YamlTree;
//! use yamlquill::document::node::{YamlNode, YamlValue, YamlString, YamlNumber};
//! use indexmap::IndexMap;
//!
//! // Create a simple tree
//! let tree = YamlTree::new(YamlNode::new(YamlValue::Object(IndexMap::from([
//!     ("name".to_string(), YamlNode::new(YamlValue::String(YamlString::Plain("Alice".to_string())))),
//!     ("age".to_string(), YamlNode::new(YamlValue::Number(YamlNumber::Integer(30)))),
//! ]))));
//!
//! // Access the root
//! assert!(tree.root().value().is_object());
//!
//! // Navigate to first field
//! let path = vec![0];
//! let child = tree.get_node(&path).unwrap();
//! if let YamlValue::String(s) = child.value() {
//!     assert_eq!(s, &YamlString::Plain("Alice".to_string()));
//! }
//! ```

use super::node::{YamlNode, YamlValue};
use std::collections::HashMap;

/// Tracks anchor definitions and alias references within a YAML tree.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AnchorRegistry {
    /// Maps anchor names to the path of the node with that anchor
    anchor_definitions: HashMap<String, Vec<usize>>,

    /// Maps alias node paths to the anchor name they reference
    alias_references: HashMap<Vec<usize>, String>,
}

impl AnchorRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an anchor definition at the given path.
    pub fn register_anchor(&mut self, name: String, path: Vec<usize>) {
        self.anchor_definitions.insert(name, path);
    }

    /// Registers an alias reference at the given path.
    pub fn register_alias(&mut self, path: Vec<usize>, target: String) {
        self.alias_references.insert(path, target);
    }

    /// Returns the path to the node with the given anchor name.
    pub fn get_anchor_path(&self, name: &str) -> Option<&Vec<usize>> {
        self.anchor_definitions.get(name)
    }

    /// Returns all alias paths that reference the given anchor.
    pub fn get_aliases_for(&self, anchor: &str) -> Vec<&Vec<usize>> {
        self.alias_references
            .iter()
            .filter(|(_, target)| target.as_str() == anchor)
            .map(|(path, _)| path)
            .collect()
    }

    /// Returns true if the anchor can be safely deleted (no aliases reference it).
    pub fn can_delete_anchor(&self, name: &str) -> bool {
        self.get_aliases_for(name).is_empty()
    }

    /// Removes all registrations for a node at the given path.
    pub fn remove_node(&mut self, path: &[usize]) {
        // Remove if it's an alias
        self.alias_references.remove(path);

        // Remove if it's an anchor (need to find by path)
        self.anchor_definitions.retain(|_, p| p != path);
    }
}

/// A complete YAML document tree.
///
/// `YamlTree` represents a parsed YAML document with a root node and optional
/// original source text for format preservation.
#[derive(Debug, Clone, PartialEq)]
pub struct YamlTree {
    root: YamlNode,
    /// The original YAML string (preserved for unmodified nodes)
    original_source: Option<String>,
    /// Tracks anchor definitions and alias references
    anchor_registry: AnchorRegistry,
}

impl YamlTree {
    /// Creates a new YAML tree with the given root node.
    ///
    /// The tree has no original source, so format preservation is not available.
    /// New nodes created via `YamlNode::new()` are marked as modified by default.
    ///
    /// # Example
    ///
    /// ```
    /// use yamlquill::document::tree::YamlTree;
    /// use yamlquill::document::node::{YamlNode, YamlValue, YamlString, YamlNumber};
    ///
    /// let root = YamlNode::new(YamlValue::Null);
    /// let tree = YamlTree::new(root);
    /// ```
    pub fn new(root: YamlNode) -> Self {
        let mut tree = Self {
            root,
            original_source: None,
            anchor_registry: AnchorRegistry::new(),
        };
        tree.build_anchor_registry();
        tree
    }

    /// Creates a new YAML tree with the given root node and original source.
    ///
    /// The original source enables format preservation for unmodified nodes.
    pub fn with_source(root: YamlNode, original_source: Option<String>) -> Self {
        let mut tree = Self {
            root,
            original_source,
            anchor_registry: AnchorRegistry::new(),
        };
        tree.build_anchor_registry();
        tree
    }

    /// Returns a reference to the original YAML source, if available.
    pub fn original_source(&self) -> Option<&str> {
        self.original_source.as_deref()
    }

    /// Returns a reference to the anchor registry.
    pub fn anchor_registry(&self) -> &AnchorRegistry {
        &self.anchor_registry
    }

    /// Returns a mutable reference to the anchor registry.
    pub fn anchor_registry_mut(&mut self) -> &mut AnchorRegistry {
        &mut self.anchor_registry
    }

    /// Returns a reference to the root node of the tree.
    ///
    /// # Example
    ///
    /// ```
    /// use yamlquill::document::tree::YamlTree;
    /// use yamlquill::document::node::{YamlNode, YamlValue, YamlString, YamlNumber};
    ///
    /// let root = YamlNode::new(YamlValue::Boolean(true));
    /// let tree = YamlTree::new(root);
    ///
    /// assert!(matches!(tree.root().value(), YamlValue::Boolean(true)));
    /// ```
    pub fn root(&self) -> &YamlNode {
        &self.root
    }

    /// Returns a mutable reference to the root node of the tree.
    ///
    /// # Example
    ///
    /// ```
    /// use yamlquill::document::tree::YamlTree;
    /// use yamlquill::document::node::{YamlNode, YamlValue, YamlString, YamlNumber};
    ///
    /// let root = YamlNode::new(YamlValue::Null);
    /// let mut tree = YamlTree::new(root);
    ///
    /// *tree.root_mut().value_mut() = YamlValue::Boolean(false);
    /// ```
    pub fn root_mut(&mut self) -> &mut YamlNode {
        &mut self.root
    }

    /// Gets an immutable reference to a node at the specified path.
    ///
    /// The path is a sequence of indices that navigate through the tree:
    /// - For objects: the index selects the nth key-value pair
    /// - For arrays: the index selects the nth element
    /// - For non-container values: any path beyond the current node returns None
    ///
    /// Returns `None` if:
    /// - The path is out of bounds at any level
    /// - The path attempts to traverse a non-container value
    ///
    /// # Example
    ///
    /// ```
    /// use yamlquill::document::tree::YamlTree;
    /// use yamlquill::document::node::{YamlNode, YamlValue, YamlString, YamlNumber};
    /// use indexmap::IndexMap;
    ///
    /// let tree = YamlTree::new(YamlNode::new(YamlValue::Object(IndexMap::from([
    ///     ("items".to_string(), YamlNode::new(YamlValue::Array(vec![
    ///         YamlNode::new(YamlValue::Number(YamlNumber::Integer(1))),
    ///         YamlNode::new(YamlValue::Number(YamlNumber::Integer(2))),
    ///     ]))),
    /// ]))));
    ///
    /// // Navigate to items[1]
    /// let path = vec![0, 1]; // First object field, second array element
    /// let node = tree.get_node(&path).unwrap();
    /// assert!(matches!(node.value(), YamlValue::Number(YamlNumber::Integer(2))));
    ///
    /// // Invalid path
    /// let invalid_path = vec![0, 99];
    /// assert!(tree.get_node(&invalid_path).is_none());
    /// ```
    pub fn get_node(&self, path: &[usize]) -> Option<&YamlNode> {
        let mut current = &self.root;

        for &index in path {
            match current.value() {
                YamlValue::Object(entries) => {
                    current = entries.get_index(index)?.1;
                }
                YamlValue::Array(elements) | YamlValue::MultiDoc(elements) => {
                    current = elements.get(index)?;
                }
                _ => return None,
            }
        }

        Some(current)
    }

    /// Gets a mutable reference to a node at the specified path.
    ///
    /// This method follows the same path resolution rules as `get_node`,
    /// but returns a mutable reference. Note that obtaining a mutable
    /// reference to a node marks it as modified.
    ///
    /// Returns `None` if:
    /// - The path is out of bounds at any level
    /// - The path attempts to traverse a non-container value
    ///
    /// # Example
    ///
    /// ```
    /// use yamlquill::document::tree::YamlTree;
    /// use yamlquill::document::node::{YamlNode, YamlValue, YamlString, YamlNumber};
    ///
    /// let mut tree = YamlTree::new(YamlNode::new(YamlValue::Array(vec![
    ///     YamlNode::new(YamlValue::String(YamlString::Plain("old".to_string()))),
    /// ])));
    ///
    /// // Modify first array element
    /// let path = vec![0];
    /// if let Some(node) = tree.get_node_mut(&path) {
    ///     *node.value_mut() = YamlValue::String(YamlString::Plain("new".to_string()));
    /// }
    ///
    /// // Verify the change
    /// let node = tree.get_node(&path).unwrap();
    /// if let YamlValue::String(s) = node.value() {
    ///     assert_eq!(s, &YamlString::Plain("new".to_string()));
    /// }
    /// ```
    pub fn get_node_mut(&mut self, path: &[usize]) -> Option<&mut YamlNode> {
        let mut current = &mut self.root;

        for &index in path {
            // Use value_mut_traverse to avoid marking parent containers as modified
            current = match current.value_mut_traverse() {
                YamlValue::Object(entries) => {
                    let (_key, value) = entries.get_index_mut(index)?;
                    value
                }
                YamlValue::Array(elements) | YamlValue::MultiDoc(elements) => {
                    elements.get_mut(index)?
                }
                _ => return None,
            };
        }

        Some(current)
    }

    /// Deletes the node at the given path.
    /// Returns an error if the path is empty (cannot delete root) or invalid.
    pub fn delete_node(&mut self, path: &[usize]) -> anyhow::Result<()> {
        use anyhow::anyhow;

        if path.is_empty() {
            return Err(anyhow!("Cannot delete root node"));
        }

        // Get parent path (all but last index)
        let parent_path = &path[..path.len() - 1];
        let index = path[path.len() - 1];

        // Get mutable reference to parent node
        let parent = self
            .get_node_mut(parent_path)
            .ok_or_else(|| anyhow!("Parent node not found"))?;

        // Delete from parent based on its type
        match parent.value_mut() {
            YamlValue::Object(entries) => {
                if index >= entries.len() {
                    return Err(anyhow!(
                        "Index {} out of bounds for object with {} entries",
                        index,
                        entries.len()
                    ));
                }
                entries.shift_remove_index(index);
            }
            YamlValue::Array(elements) | YamlValue::MultiDoc(elements) => {
                if index >= elements.len() {
                    return Err(anyhow!(
                        "Index {} out of bounds for array with {} elements",
                        index,
                        elements.len()
                    ));
                }
                elements.remove(index);
            }
            _ => {
                return Err(anyhow!("Parent is not a container type"));
            }
        }

        Ok(())
    }

    /// Inserts a node into an object at the specified path and index.
    /// The path must point to the object, and index specifies where to insert.
    pub fn insert_node_in_object(
        &mut self,
        path: &[usize],
        key: String,
        node: YamlNode,
    ) -> anyhow::Result<()> {
        use anyhow::anyhow;

        // Get parent path (all but last index)
        let parent_path = if path.is_empty() {
            &[]
        } else {
            &path[..path.len() - 1]
        };
        let index = if path.is_empty() {
            0
        } else {
            path[path.len() - 1]
        };

        // Get mutable reference to parent (or root if path is empty)
        let target = if parent_path.is_empty() {
            self.root_mut()
        } else {
            self.get_node_mut(parent_path)
                .ok_or_else(|| anyhow!("Parent node not found"))?
        };

        // Insert into object
        match target.value_mut() {
            YamlValue::Object(entries) => {
                if index > entries.len() {
                    return Err(anyhow!(
                        "Index {} out of bounds for object with {} entries",
                        index,
                        entries.len()
                    ));
                }
                entries.shift_insert(index, key, node);
            }
            _ => {
                return Err(anyhow!("Target is not an object"));
            }
        }

        Ok(())
    }

    /// Inserts a node into an array at the specified path and index.
    pub fn insert_node_in_array(&mut self, path: &[usize], node: YamlNode) -> anyhow::Result<()> {
        use anyhow::anyhow;

        // Get parent path (all but last index)
        let parent_path = if path.is_empty() {
            &[]
        } else {
            &path[..path.len() - 1]
        };
        let index = if path.is_empty() {
            0
        } else {
            path[path.len() - 1]
        };

        // Get mutable reference to parent (or root if path is empty)
        let target = if parent_path.is_empty() {
            self.root_mut()
        } else {
            self.get_node_mut(parent_path)
                .ok_or_else(|| anyhow!("Parent node not found"))?
        };

        // Insert into array
        match target.value_mut() {
            YamlValue::Array(elements) | YamlValue::MultiDoc(elements) => {
                if index > elements.len() {
                    return Err(anyhow!(
                        "Index {} out of bounds for array with {} elements",
                        index,
                        elements.len()
                    ));
                }
                elements.insert(index, node);
            }
            _ => {
                return Err(anyhow!("Target is not an array"));
            }
        }

        Ok(())
    }

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
                Some(path[..last_bracket].to_string())
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

    /// Builds the anchor registry by walking the tree and registering all anchors and aliases.
    pub fn build_anchor_registry(&mut self) {
        self.anchor_registry = AnchorRegistry::new();
        self.register_anchors_recursive(&self.root.clone(), &[]);
    }

    /// Recursively registers anchors and aliases in the tree.
    fn register_anchors_recursive(&mut self, node: &YamlNode, path: &[usize]) {
        // Register anchor if this node has one
        if let Some(anchor_name) = node.anchor() {
            self.anchor_registry
                .register_anchor(anchor_name.to_string(), path.to_vec());
        }

        // Register alias if this node is an alias
        if let Some(alias_target) = node.alias_target() {
            self.anchor_registry
                .register_alias(path.to_vec(), alias_target.to_string());
        }

        // Recurse into children
        match node.value() {
            YamlValue::Object(map) => {
                for (i, (_key, child)) in map.iter().enumerate() {
                    let mut child_path = path.to_vec();
                    child_path.push(i);
                    self.register_anchors_recursive(child, &child_path);
                }
            }
            YamlValue::Array(items) => {
                for (i, child) in items.iter().enumerate() {
                    let mut child_path = path.to_vec();
                    child_path.push(i);
                    self.register_anchors_recursive(child, &child_path);
                }
            }
            YamlValue::MultiDoc(docs) => {
                for (i, child) in docs.iter().enumerate() {
                    let mut child_path = path.to_vec();
                    child_path.push(i);
                    self.register_anchors_recursive(child, &child_path);
                }
            }
            _ => {} // Scalar types don't have children
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::node::YamlString;

    #[test]
    fn test_tree_with_original_source() {
        let root = YamlNode::new(YamlValue::String(YamlString::Plain("test".to_string())));
        let tree = YamlTree::with_source(root.clone(), Some("\"test\"".to_string()));

        assert_eq!(tree.original_source(), Some("\"test\""));
    }

    #[test]
    fn test_tree_without_original_source() {
        let root = YamlNode::new(YamlValue::Null);
        let tree = YamlTree::new(root);

        assert_eq!(tree.original_source(), None);
    }

    #[test]
    fn test_get_parent_path() {
        let tree = YamlTree::new(YamlNode::new(YamlValue::Null));

        assert_eq!(tree.get_parent_path("$"), None);
        assert_eq!(tree.get_parent_path("name"), None);
        assert_eq!(
            tree.get_parent_path("config.timeout"),
            Some("config".to_string())
        );
        assert_eq!(tree.get_parent_path("users[0]"), Some("users".to_string()));
        assert_eq!(
            tree.get_parent_path("users[0].name"),
            Some("users[0]".to_string())
        );
    }

    #[test]
    fn test_get_depth() {
        let tree = YamlTree::new(YamlNode::new(YamlValue::Null));

        assert_eq!(tree.get_depth("$"), 0);
        assert_eq!(tree.get_depth("name"), 0);
        assert_eq!(tree.get_depth("config.timeout"), 1);
        assert_eq!(tree.get_depth("users[0].name"), 2);
    }
}
