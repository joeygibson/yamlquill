//! YAML file saving functionality.
//!
//! This module provides functions to save `YamlTree` structures to files with
//! atomic write operations and optional backup creation.

use crate::config::Config;
use crate::document::node::{YamlNode, YamlNumber, YamlValue};
use crate::document::tree::YamlTree;
use anyhow::{Context, Result};
use serde_yaml::Value;
use std::fs;
use std::path::Path;

/// Converts a YamlNode tree to a serde_yaml::Value.
///
/// This function recursively traverses the YamlNode structure and converts
/// it to serde_yaml::Value for serialization.
///
/// # Arguments
///
/// * `node` - The YamlNode to convert
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok(Value)` with the converted value
/// - `Err(anyhow::Error)` if the node type is not supported (Alias or MultiDoc)
///
/// # Type Conversions
///
/// - `YamlValue::Null` → `Value::Null`
/// - `YamlValue::Boolean` → `Value::Bool`
/// - `YamlValue::Number` → `Value::Number` (preserves Integer vs Float)
/// - `YamlValue::String` → `Value::String` (always plain in v1)
/// - `YamlValue::Array` → `Value::Sequence`
/// - `YamlValue::Object` → `Value::Mapping`
/// - `YamlValue::Alias` → Error (not supported in v1)
/// - `YamlValue::MultiDoc` → Error (not supported in v1)
///
/// # V1 Limitations
///
/// - Alias nodes return an error (Phase 3 will add support)
/// - MultiDoc nodes return an error (use save_yamll instead)
/// - All strings output as plain style (Phase 4 will preserve literal/folded)
fn convert_to_serde_value(node: &YamlNode) -> Result<Value> {
    let value = match node.value() {
        YamlValue::Null => Value::Null,

        YamlValue::Boolean(b) => Value::Bool(*b),

        YamlValue::Number(n) => {
            match n {
                YamlNumber::Integer(i) => Value::Number(serde_yaml::Number::from(*i)),
                YamlNumber::Float(f) => {
                    // serde_yaml::Number doesn't have a direct from_f64
                    // We need to serialize to f64 via serde_json compatibility
                    serde_yaml::to_value(f).context("Failed to convert float to YAML value")?
                }
            }
        }

        YamlValue::String(s) => {
            // V1: Always output as plain string
            // Phase 4 will preserve literal (|) and folded (>) styles
            Value::String(s.as_str().to_string())
        }

        YamlValue::Array(elements) => {
            let seq: Result<Vec<Value>> = elements.iter().map(convert_to_serde_value).collect();
            Value::Sequence(seq?)
        }

        YamlValue::Object(entries) => {
            let mut map = serde_yaml::Mapping::new();
            for (key, value) in entries {
                map.insert(Value::String(key.clone()), convert_to_serde_value(value)?);
            }
            Value::Mapping(map)
        }

        YamlValue::Alias(name) => {
            anyhow::bail!("Cannot serialize Alias nodes in v1 (alias: *{})", name)
        }

        YamlValue::MultiDoc(_) => {
            anyhow::bail!("Cannot serialize MultiDoc nodes - use save_yamll instead")
        }
    };

    Ok(value)
}

/// Creates a backup of a file by copying it with a .bak extension.
fn create_backup<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    let mut backup_path = path.to_path_buf();
    let original_name = backup_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?;
    backup_path.set_file_name(format!("{}.bak", original_name));
    fs::copy(path, backup_path).context("Failed to create backup")?;
    Ok(())
}

/// Saves a YAML tree to a file with optional backup creation.
///
/// This function serializes a `YamlTree` to YAML format and writes it to the
/// specified file path. The write operation is atomic (writes to a temp file
/// then renames) to prevent data loss on crashes. Optionally creates a backup
/// of the original file before writing.
///
/// For multi-document YAML documents (YamlValue::MultiDoc), automatically uses
/// line-by-line format via save_yamll.
///
/// # Arguments
///
/// * `path` - The path where the YAML file should be saved
/// * `tree` - The YAML tree to serialize and save
/// * `config` - Configuration including backup settings
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok(())` if the file was successfully saved
/// - `Err(anyhow::Error)` if:
///   - Creating a backup failed
///   - Serialization failed
///   - Writing the temp file failed
///   - Renaming the temp file to the target failed
///
/// # Examples
///
/// ```no_run
/// use yamlquill::file::saver::save_yaml_file;
/// use yamlquill::document::node::{YamlNode, YamlValue};
/// use yamlquill::document::tree::YamlTree;
/// use yamlquill::config::Config;
/// use indexmap::IndexMap;
///
/// let tree = YamlTree::new(YamlNode::new(YamlValue::Object(IndexMap::new())));
/// let config = Config::default();
/// save_yaml_file("output.yaml", &tree, &config).unwrap();
/// ```
///
/// # Atomic Write
///
/// This function uses an atomic write strategy:
/// 1. Serializes the YAML to a temporary file
/// 2. Renames the temporary file to the target path
///
/// This ensures that the target file is never left in a partially written state.
///
/// # Gzip Compression
///
/// If the path ends with `.gz`, the file will be automatically compressed with gzip.
pub fn save_yaml_file<P: AsRef<Path>>(path: P, tree: &YamlTree, config: &Config) -> Result<()> {
    let path = path.as_ref();

    // Determine if we should compress based on target filename
    let should_compress = path.to_string_lossy().ends_with(".gz");

    // Check if this is a multi-document YAML document
    if matches!(tree.root().value(), YamlValue::MultiDoc(_)) {
        return save_yamll(path, tree, config, should_compress);
    }

    // Create backup if requested and file exists
    if config.create_backup && path.exists() {
        create_backup(path)?;
    }

    // Convert YamlNode to serde_yaml::Value
    let value = convert_to_serde_value(tree.root())?;

    // Serialize to YAML string
    let yaml_str = serde_yaml::to_string(&value).context("Failed to serialize YAML")?;

    // Write atomically (compressed or uncompressed)
    write_file_atomic(path, yaml_str.as_bytes(), should_compress)?;

    Ok(())
}

/// Writes data to a file atomically, optionally compressing with gzip.
///
/// This function writes to a temporary file first, then atomically renames
/// it to the target path. This ensures the target file is never left in a
/// partially written state.
///
/// # Arguments
///
/// * `path` - Target file path
/// * `data` - Bytes to write
/// * `compress` - Whether to gzip-compress the data before writing
///
/// # Errors
///
/// Returns an error if:
/// - Creating the temp file fails
/// - Writing or compressing fails
/// - Renaming the temp file fails
fn write_file_atomic<P: AsRef<Path>>(path: P, data: &[u8], compress: bool) -> Result<()> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    let path = path.as_ref();
    let temp_path = path.with_extension("tmp");

    if compress {
        // Write compressed
        let file = fs::File::create(&temp_path).context("Failed to create temp file")?;
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder
            .write_all(data)
            .context("Failed to write compressed data")?;
        encoder.finish().context("Failed to finish compression")?;
    } else {
        // Write uncompressed
        fs::write(&temp_path, data).context("Failed to write temp file")?;
    }

    // Atomic rename
    fs::rename(&temp_path, path).context("Failed to rename temp file")?;

    Ok(())
}

/// Saves a multi-document YAML document to a file.
///
/// Documents are separated by `---` directives as per YAML 1.2 spec.
fn save_yamll<P: AsRef<Path>>(
    path: P,
    tree: &YamlTree,
    config: &Config,
    compress: bool,
) -> Result<()> {
    let path = path.as_ref();

    // Create backup if requested and file exists
    if config.create_backup && path.exists() {
        create_backup(path)?;
    }

    let mut output = String::new();

    if let YamlValue::MultiDoc(documents) = tree.root().value() {
        for (i, node) in documents.iter().enumerate() {
            // Add document separator before each document
            output.push_str("---\n");

            // Convert to serde_yaml::Value
            let value = convert_to_serde_value(node)
                .with_context(|| format!("Failed to convert document {} to YAML", i + 1))?;

            // Serialize to YAML with proper formatting
            let yaml = serde_yaml::to_string(&value)
                .with_context(|| format!("Failed to serialize document {}", i + 1))?;

            // Validate the serialized YAML
            serde_yaml::from_str::<serde_yaml::Value>(&yaml).with_context(|| {
                format!(
                    "Generated invalid YAML at document {} - this is a bug in yamlquill's serialization",
                    i + 1
                )
            })?;

            output.push_str(&yaml);

            // serde_yaml::to_string adds a trailing newline, so we don't need to add another
        }
    }

    // Write atomically with optional compression
    write_file_atomic(path, output.as_bytes(), compress)?;

    Ok(())
}

/// Serializes a node with format preservation for unmodified nodes.
///
/// If the node is unmodified and has a text span, extracts the original text.
/// Otherwise, serializes using the configured formatting.
#[allow(dead_code)]
fn serialize_preserving_format(
    node: &YamlNode,
    original: &str,
    config: &Config,
    depth: usize,
) -> String {
    // If format preservation is disabled, always use fresh serialization
    if !config.preserve_formatting {
        return serialize_node(node, config.indent_size, depth);
    }

    // If node is unmodified and has a valid text span, extract from original
    if !node.is_modified() {
        if let Some(span) = node.metadata.text_span.as_ref() {
            return original[span.start..span.end].to_string();
        }
    }

    // Node was modified or has no span - serialize fresh
    match node.value() {
        YamlValue::Object(entries) => serialize_object_preserving(entries, original, config, depth),
        YamlValue::Array(elements) | YamlValue::MultiDoc(elements) => {
            serialize_array_preserving(elements, original, config, depth)
        }
        _ => serialize_node(node, config.indent_size, depth),
    }
}

/// Serializes an object with format preservation for children.
#[allow(dead_code)]
fn serialize_object_preserving(
    entries: &indexmap::IndexMap<String, YamlNode>,
    original: &str,
    config: &Config,
    depth: usize,
) -> String {
    if entries.is_empty() {
        return "{}".to_string();
    }

    let indent = " ".repeat(config.indent_size * depth);
    let next_indent = " ".repeat(config.indent_size * (depth + 1));

    let mut result = "{\n".to_string();
    for (i, (key, value)) in entries.iter().enumerate() {
        result.push_str(&next_indent);
        result.push_str(&format!("\"{}\": ", escape_yaml_string(key)));
        result.push_str(&serialize_preserving_format(
            value,
            original,
            config,
            depth + 1,
        ));
        if i < entries.len() - 1 {
            result.push(',');
        }
        result.push('\n');
    }
    result.push_str(&indent);
    result.push('}');
    result
}

/// Serializes an array with format preservation for children.
#[allow(dead_code)]
fn serialize_array_preserving(
    elements: &[YamlNode],
    original: &str,
    config: &Config,
    depth: usize,
) -> String {
    if elements.is_empty() {
        return "[]".to_string();
    }

    let indent = " ".repeat(config.indent_size * depth);
    let next_indent = " ".repeat(config.indent_size * (depth + 1));

    let mut result = "[\n".to_string();
    for (i, element) in elements.iter().enumerate() {
        result.push_str(&next_indent);
        result.push_str(&serialize_preserving_format(
            element,
            original,
            config,
            depth + 1,
        ));
        if i < elements.len() - 1 {
            result.push(',');
        }
        result.push('\n');
    }
    result.push_str(&indent);
    result.push(']');
    result
}

/// Serializes a JSON node to a compact single-line string.
///
/// This is used for multi-document YAML format where each line must be a single-line JSON object.
/// Numbers are formatted as integers when they have no fractional part.
pub fn serialize_node_compact(node: &YamlNode) -> String {
    match node.value() {
        YamlValue::Object(entries) => {
            if entries.is_empty() {
                return "{}".to_string();
            }
            let parts: Vec<String> = entries
                .iter()
                .map(|(key, value)| {
                    format!(
                        "\"{}\":{}",
                        escape_yaml_string(key),
                        serialize_node_compact(value)
                    )
                })
                .collect();
            format!("{{{}}}", parts.join(","))
        }
        YamlValue::Array(elements) | YamlValue::MultiDoc(elements) => {
            if elements.is_empty() {
                return "[]".to_string();
            }
            let parts: Vec<String> = elements.iter().map(serialize_node_compact).collect();
            format!("[{}]", parts.join(","))
        }
        YamlValue::String(s) => format!("\"{}\"", escape_yaml_string(s.as_str())),
        YamlValue::Number(n) => {
            // Format numbers cleanly - remove unnecessary decimal points
            match n {
                YamlNumber::Integer(i) => i.to_string(),
                YamlNumber::Float(f) => {
                    if f.fract() == 0.0 && f.is_finite() {
                        format!("{:.0}", f)
                    } else {
                        f.to_string()
                    }
                }
            }
        }
        YamlValue::Boolean(b) => b.to_string(),
        YamlValue::Null => "null".to_string(),
        YamlValue::Alias(_) => panic!("Cannot serialize Alias in v1"),
    }
}

/// Serializes a JSON node in jq style (strict multi-line formatting).
///
/// This function matches jq's formatting behavior: all objects and arrays
/// are formatted with multi-line indentation, even if they're small.
/// No compact single-line formatting is used.
///
/// # Arguments
///
/// * `node` - The JSON node to serialize
/// * `indent_size` - Number of spaces per indentation level
/// * `current_depth` - Current nesting depth (used for recursion)
///
/// # Returns
///
/// A jq-style formatted JSON string
pub fn serialize_node_jq_style(
    node: &YamlNode,
    indent_size: usize,
    current_depth: usize,
) -> String {
    let indent = " ".repeat(indent_size * current_depth);
    let next_indent = " ".repeat(indent_size * (current_depth + 1));

    match node.value() {
        YamlValue::Object(entries) => {
            if entries.is_empty() {
                return "{}".to_string();
            }

            // jq always uses multi-line formatting for objects
            let mut result = "{\n".to_string();
            for (i, (key, value)) in entries.iter().enumerate() {
                result.push_str(&next_indent);
                result.push_str(&format!("\"{}\": ", escape_yaml_string(key)));
                result.push_str(&serialize_node_jq_style(
                    value,
                    indent_size,
                    current_depth + 1,
                ));
                if i < entries.len() - 1 {
                    result.push(',');
                }
                result.push('\n');
            }
            result.push_str(&indent);
            result.push('}');
            result
        }
        YamlValue::Array(elements) | YamlValue::MultiDoc(elements) => {
            if elements.is_empty() {
                return "[]".to_string();
            }

            // jq always uses multi-line formatting for arrays
            let mut result = "[\n".to_string();
            for (i, element) in elements.iter().enumerate() {
                result.push_str(&next_indent);
                result.push_str(&serialize_node_jq_style(
                    element,
                    indent_size,
                    current_depth + 1,
                ));
                if i < elements.len() - 1 {
                    result.push(',');
                }
                result.push('\n');
            }
            result.push_str(&indent);
            result.push(']');
            result
        }
        YamlValue::String(s) => format!("\"{}\"", escape_yaml_string(s.as_str())),
        YamlValue::Number(n) => match n {
            YamlNumber::Integer(i) => i.to_string(),
            YamlNumber::Float(f) => {
                if f.fract() == 0.0 && f.is_finite() {
                    format!("{:.0}", f)
                } else {
                    f.to_string()
                }
            }
        },
        YamlValue::Boolean(b) => b.to_string(),
        YamlValue::Null => "null".to_string(),
        YamlValue::Alias(_) => panic!("Cannot serialize Alias in v1"),
    }
}

/// Recursively serializes a JSON node to a formatted string.
///
/// This function converts a `YamlNode` and all its children into a JSON string
/// with proper indentation and formatting. It handles all JSON value types
/// including nested objects and arrays.
///
/// For arrays and objects containing only scalar values, uses compact single-line
/// formatting if the result would be reasonably short (< 80 characters).
///
/// # Arguments
///
/// * `node` - The JSON node to serialize
/// * `indent_size` - Number of spaces per indentation level
/// * `current_depth` - Current nesting depth (used for recursion)
///
/// # Returns
///
/// A formatted JSON string representing the node
pub fn serialize_node(node: &YamlNode, indent_size: usize, current_depth: usize) -> String {
    let indent = " ".repeat(indent_size * current_depth);
    let next_indent = " ".repeat(indent_size * (current_depth + 1));

    match node.value() {
        YamlValue::Object(entries) => {
            if entries.is_empty() {
                return "{}".to_string();
            }

            // Try compact formatting for objects with only scalar values
            if should_use_compact_format_object(entries) {
                let compact = serialize_object_compact(entries);
                if compact.len() <= 80 {
                    return compact;
                }
            }

            // Use multi-line formatting
            let mut result = "{\n".to_string();
            for (i, (key, value)) in entries.iter().enumerate() {
                result.push_str(&next_indent);
                result.push_str(&format!("\"{}\": ", escape_yaml_string(key)));
                result.push_str(&serialize_node(value, indent_size, current_depth + 1));
                if i < entries.len() - 1 {
                    result.push(',');
                }
                result.push('\n');
            }
            result.push_str(&indent);
            result.push('}');
            result
        }
        YamlValue::Array(elements) | YamlValue::MultiDoc(elements) => {
            if elements.is_empty() {
                return "[]".to_string();
            }

            // Try compact formatting for arrays with only scalar values
            if should_use_compact_format_array(elements) {
                let compact = serialize_array_compact(elements);
                if compact.len() <= 80 {
                    return compact;
                }
            }

            // Use multi-line formatting
            let mut result = "[\n".to_string();
            for (i, element) in elements.iter().enumerate() {
                result.push_str(&next_indent);
                result.push_str(&serialize_node(element, indent_size, current_depth + 1));
                if i < elements.len() - 1 {
                    result.push(',');
                }
                result.push('\n');
            }
            result.push_str(&indent);
            result.push(']');
            result
        }
        YamlValue::String(s) => format!("\"{}\"", escape_yaml_string(s.as_str())),
        YamlValue::Number(n) => {
            // Format numbers cleanly - remove unnecessary decimal points
            match n {
                YamlNumber::Integer(i) => i.to_string(),
                YamlNumber::Float(f) => {
                    if f.fract() == 0.0 && f.is_finite() {
                        format!("{:.0}", f)
                    } else {
                        f.to_string()
                    }
                }
            }
        }
        YamlValue::Boolean(b) => b.to_string(),
        YamlValue::Null => "null".to_string(),
        YamlValue::Alias(_) => panic!("Cannot serialize Alias in v1"),
    }
}

/// Checks if an object should use compact (single-line) formatting.
///
/// Returns true if all values in the object are scalar (not containers).
fn should_use_compact_format_object(entries: &indexmap::IndexMap<String, YamlNode>) -> bool {
    entries.values().all(|node| !node.value().is_container())
}

/// Checks if an array should use compact (single-line) formatting.
///
/// Returns true if all elements in the array are scalar (not containers).
fn should_use_compact_format_array(elements: &[YamlNode]) -> bool {
    elements.iter().all(|node| !node.value().is_container())
}

/// Serializes an object in compact (single-line) format.
///
/// Example: `{"a": 1, "b": "hello", "c": true}`
fn serialize_object_compact(entries: &indexmap::IndexMap<String, YamlNode>) -> String {
    let parts: Vec<String> = entries
        .iter()
        .map(|(key, value)| {
            format!(
                "\"{}\": {}",
                escape_yaml_string(key),
                serialize_scalar(value.value())
            )
        })
        .collect();
    format!("{{{}}}", parts.join(", "))
}

/// Serializes an array in compact (single-line) format.
///
/// Example: `[1, 2, 3, 4, 5]`
fn serialize_array_compact(elements: &[YamlNode]) -> String {
    let parts: Vec<String> = elements
        .iter()
        .map(|node| serialize_scalar(node.value()))
        .collect();
    format!("[{}]", parts.join(", "))
}

/// Serializes a scalar value (not a container) to a string.
///
/// This is a simplified version of serialize_node for scalar values only.
fn serialize_scalar(value: &YamlValue) -> String {
    match value {
        YamlValue::String(s) => format!("\"{}\"", escape_yaml_string(s.as_str())),
        YamlValue::Number(n) => match n {
            YamlNumber::Integer(i) => i.to_string(),
            YamlNumber::Float(f) => {
                if f.fract() == 0.0 && f.is_finite() {
                    format!("{:.0}", f)
                } else {
                    f.to_string()
                }
            }
        },
        YamlValue::Boolean(b) => b.to_string(),
        YamlValue::Null => "null".to_string(),
        _ => panic!("serialize_scalar called on non-scalar value"),
    }
}

/// Escapes special characters in a string for JSON serialization.
///
/// This function handles all special characters that need escaping in JSON strings:
/// - Backslash (\)
/// - Double quote (")
/// - Control characters (newline, tab, carriage return, etc.)
///
/// # Arguments
///
/// * `s` - The string to escape
///
/// # Returns
///
/// A new string with all special characters properly escaped
fn escape_yaml_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());

    for c in s.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\x08' => result.push_str("\\b"),
            '\x0C' => result.push_str("\\f"),
            c if c.is_control() => {
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::node::{YamlNumber, YamlString};
    use indexmap::IndexMap;

    #[test]
    fn test_serialize_null() {
        let node = YamlNode::new(YamlValue::Null);
        let result = serialize_node(&node, 2, 0);
        assert_eq!(result, "null");
    }

    #[test]
    fn test_serialize_boolean() {
        let node = YamlNode::new(YamlValue::Boolean(true));
        let result = serialize_node(&node, 2, 0);
        assert_eq!(result, "true");

        let node = YamlNode::new(YamlValue::Boolean(false));
        let result = serialize_node(&node, 2, 0);
        assert_eq!(result, "false");
    }

    #[test]
    fn test_serialize_number() {
        let node = YamlNode::new(YamlValue::Number(YamlNumber::Float(42.0)));
        let result = serialize_node(&node, 2, 0);
        assert_eq!(result, "42");

        let node = YamlNode::new(YamlValue::Number(YamlNumber::Float(2.5)));
        let result = serialize_node(&node, 2, 0);
        assert_eq!(result, "2.5");
    }

    #[test]
    fn test_serialize_string() {
        let node = YamlNode::new(YamlValue::String(YamlString::Plain("hello".to_string())));
        let result = serialize_node(&node, 2, 0);
        assert_eq!(result, "\"hello\"");
    }

    #[test]
    fn test_serialize_empty_object() {
        let node = YamlNode::new(YamlValue::Object(IndexMap::new()));
        let result = serialize_node(&node, 2, 0);
        assert_eq!(result, "{}");
    }

    #[test]
    fn test_serialize_empty_array() {
        let node = YamlNode::new(YamlValue::Array(vec![]));
        let result = serialize_node(&node, 2, 0);
        assert_eq!(result, "[]");
    }

    #[test]
    fn test_serialize_simple_object() {
        let mut obj = IndexMap::new();
        obj.insert(
            "name".to_string(),
            YamlNode::new(YamlValue::String(YamlString::Plain("Alice".to_string()))),
        );
        let node = YamlNode::new(YamlValue::Object(obj));
        let result = serialize_node(&node, 2, 0);
        // Small scalar objects use compact formatting
        assert_eq!(result, "{\"name\": \"Alice\"}");
    }

    #[test]
    fn test_serialize_simple_array() {
        let arr = vec![
            YamlNode::new(YamlValue::Number(YamlNumber::Float(1.0))),
            YamlNode::new(YamlValue::Number(YamlNumber::Float(2.0))),
            YamlNode::new(YamlValue::Number(YamlNumber::Float(3.0))),
        ];
        let node = YamlNode::new(YamlValue::Array(arr));
        let result = serialize_node(&node, 2, 0);
        // Small scalar arrays use compact formatting
        assert_eq!(result, "[1, 2, 3]");
    }

    #[test]
    fn test_serialize_nested_object() {
        let mut inner = IndexMap::new();
        inner.insert(
            "age".to_string(),
            YamlNode::new(YamlValue::Number(YamlNumber::Float(30.0))),
        );
        let mut outer = IndexMap::new();
        outer.insert("user".to_string(), YamlNode::new(YamlValue::Object(inner)));
        let node = YamlNode::new(YamlValue::Object(outer));
        let result = serialize_node(&node, 2, 0);
        // Inner object with single scalar value uses compact formatting
        assert_eq!(result, "{\n  \"user\": {\"age\": 30}\n}");
    }

    #[test]
    fn test_escape_yaml_string() {
        assert_eq!(escape_yaml_string("hello"), "hello");
        assert_eq!(escape_yaml_string("hello\"world"), "hello\\\"world");
        assert_eq!(escape_yaml_string("hello\\world"), "hello\\\\world");
        assert_eq!(escape_yaml_string("hello\nworld"), "hello\\nworld");
        assert_eq!(escape_yaml_string("hello\tworld"), "hello\\tworld");
        assert_eq!(escape_yaml_string("hello\rworld"), "hello\\rworld");
    }

    #[test]
    fn test_compact_array_with_scalars() {
        let arr = vec![
            YamlNode::new(YamlValue::Number(YamlNumber::Float(1.0))),
            YamlNode::new(YamlValue::String(YamlString::Plain("test".to_string()))),
            YamlNode::new(YamlValue::Boolean(true)),
            YamlNode::new(YamlValue::Null),
        ];
        let node = YamlNode::new(YamlValue::Array(arr));
        let result = serialize_node(&node, 2, 0);
        assert_eq!(result, "[1, \"test\", true, null]");
    }

    #[test]
    fn test_compact_object_with_scalars() {
        let mut obj = IndexMap::new();
        obj.insert(
            "a".to_string(),
            YamlNode::new(YamlValue::Number(YamlNumber::Float(1.0))),
        );
        obj.insert(
            "b".to_string(),
            YamlNode::new(YamlValue::String(YamlString::Plain("test".to_string()))),
        );
        obj.insert("c".to_string(), YamlNode::new(YamlValue::Boolean(false)));
        let node = YamlNode::new(YamlValue::Object(obj));
        let result = serialize_node(&node, 2, 0);
        assert_eq!(result, "{\"a\": 1, \"b\": \"test\", \"c\": false}");
    }

    #[test]
    fn test_nested_containers_use_multiline() {
        // Array containing an object should use multi-line formatting
        let mut inner = IndexMap::new();
        inner.insert(
            "key".to_string(),
            YamlNode::new(YamlValue::String(YamlString::Plain("value".to_string()))),
        );
        let arr = vec![YamlNode::new(YamlValue::Object(inner))];
        let node = YamlNode::new(YamlValue::Array(arr));
        let result = serialize_node(&node, 2, 0);
        assert!(
            result.contains('\n'),
            "Nested containers should use multi-line formatting"
        );
    }

    #[test]
    fn test_long_compact_array_uses_multiline() {
        // Create an array that would exceed 80 characters in compact format
        let arr: Vec<YamlNode> = (0..30)
            .map(|i| YamlNode::new(YamlValue::Number(YamlNumber::Float(i as f64))))
            .collect();
        let node = YamlNode::new(YamlValue::Array(arr));
        let result = serialize_node(&node, 2, 0);
        // Should fall back to multi-line because compact would be > 80 chars
        assert!(
            result.contains('\n'),
            "Long arrays should use multi-line formatting"
        );
    }

    #[test]
    fn test_roundtrip_preserves_formatting() {
        use crate::document::parser::parse_yaml;
        use std::fs;
        use tempfile::NamedTempFile;

        let original_yaml = "active: true\nage: 30\nname: Alice\n";

        // Parse
        let root_node = parse_yaml(original_yaml).unwrap();
        let tree = YamlTree::new(root_node);
        let config = Config {
            preserve_formatting: true,
            ..Default::default()
        };

        // Save
        let temp_file = NamedTempFile::new().unwrap();
        save_yaml_file(temp_file.path(), &tree, &config).unwrap();

        // Read back
        let saved_yaml = fs::read_to_string(temp_file.path()).unwrap();

        // Verify it's valid YAML with the same fields
        let parsed: serde_yaml::Value = serde_yaml::from_str(&saved_yaml).unwrap();
        assert_eq!(parsed["name"], "Alice");
        assert_eq!(parsed["age"], 30);
        assert_eq!(parsed["active"], true);
    }

    #[test]
    fn test_modified_node_uses_config_formatting() {
        use crate::document::node::YamlValue;
        use crate::document::parser::parse_yaml;
        use std::fs;
        use tempfile::NamedTempFile;

        let original_yaml = "name: Alice\n";

        // Parse
        let root_node = parse_yaml(original_yaml).unwrap();
        let mut tree = YamlTree::new(root_node);

        // Modify a value
        if let YamlValue::Object(ref mut entries) = tree.root_mut().value_mut() {
            *entries.get_index_mut(0).unwrap().1.value_mut() =
                YamlValue::String(YamlString::Plain("Bob".to_string()));
        }

        let config = Config::default();

        // Save
        let temp_file = NamedTempFile::new().unwrap();
        save_yaml_file(temp_file.path(), &tree, &config).unwrap();

        // Read back
        let saved_yaml = fs::read_to_string(temp_file.path()).unwrap();

        // Verify YAML format with modified value
        let parsed: serde_yaml::Value = serde_yaml::from_str(&saved_yaml).unwrap();
        assert_eq!(parsed["name"], "Bob");
    }

    #[test]
    fn test_preserve_formatting_can_be_disabled() {
        use crate::document::parser::parse_yaml;
        use std::fs;
        use tempfile::NamedTempFile;

        let original_yaml = "name: Alice\nage: 30\n";

        // Parse
        let root_node = parse_yaml(original_yaml).unwrap();
        let tree = YamlTree::new(root_node);

        // Disable format preservation
        let config = Config {
            preserve_formatting: false,
            ..Default::default()
        };

        // Save
        let temp_file = NamedTempFile::new().unwrap();
        save_yaml_file(temp_file.path(), &tree, &config).unwrap();

        // Read back
        let saved_yaml = fs::read_to_string(temp_file.path()).unwrap();

        // Verify YAML format with normalized structure
        let parsed: serde_yaml::Value = serde_yaml::from_str(&saved_yaml).unwrap();
        assert_eq!(parsed["name"], "Alice");
        assert_eq!(parsed["age"], 30);
    }

    #[test]
    fn test_edit_parent_invalidates_child_spans() {
        use crate::document::node::YamlValue;
        use crate::document::parser::parse_yaml;
        use std::fs;
        use tempfile::NamedTempFile;

        // Reproduce the exact scenario: company object with products array
        // When we rename a key in company and add a field, the products array
        // byte positions shift but the array itself isn't marked modified
        let original_yaml = r#"company:
  name: TechCorp
  products:
  - id: prod-1
    title: Product A
"#;

        let root_node = parse_yaml(original_yaml).unwrap();
        let mut tree = YamlTree::new(root_node);

        // Navigate to company object and modify it
        if let YamlValue::Object(ref mut root_entries) = tree.root_mut().value_mut() {
            if let YamlValue::Object(ref mut company_entries) =
                root_entries.get_index_mut(0).unwrap().1.value_mut()
            {
                // Rename "name" to "companyName" by removing old and inserting new
                if let Some((_, value)) = company_entries.shift_remove_entry("name") {
                    company_entries.insert("companyName".to_string(), value);
                }

                // Add a new field "employees": 23
                company_entries.insert(
                    "employees".to_string(),
                    crate::document::node::YamlNode::new(YamlValue::Number(YamlNumber::Integer(
                        23,
                    ))),
                );
            }
        }

        let config = crate::config::Config::default();
        let temp_file = NamedTempFile::new().unwrap();
        crate::file::saver::save_yaml_file(temp_file.path(), &tree, &config).unwrap();

        let saved_yaml = fs::read_to_string(temp_file.path()).unwrap();

        // Verify the saved YAML is valid
        let reparsed = serde_yaml::from_str::<serde_yaml::Value>(&saved_yaml);
        assert!(
            reparsed.is_ok(),
            "Saved YAML should be valid, but got: {}",
            saved_yaml
        );

        // Verify products array is intact
        let parsed = reparsed.unwrap();
        let company = &parsed["company"];
        assert_eq!(company["companyName"], "TechCorp");
        assert_eq!(company["employees"], 23);
        let products = company["products"].as_sequence().unwrap();
        assert_eq!(products.len(), 1);
        assert_eq!(products[0]["id"], "prod-1");
        assert_eq!(products[0]["title"], "Product A");
    }

    #[test]
    fn test_write_file_atomic_uncompressed() {
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().unwrap();
        let target_path = temp_file.path();
        let data = b"test content";

        write_file_atomic(target_path, data, false).unwrap();

        let written = fs::read_to_string(target_path).unwrap();
        assert_eq!(written, "test content");
    }

    #[test]
    fn test_write_file_atomic_compressed() {
        use flate2::read::GzDecoder;
        use std::io::Read;
        use tempfile::NamedTempFile;

        let temp_file = NamedTempFile::new().unwrap();
        let target_path = temp_file.path().with_extension("json.gz");
        let data = b"test content";

        write_file_atomic(&target_path, data, true).unwrap();

        // Decompress and verify
        let file = fs::File::open(&target_path).unwrap();
        let mut decoder = GzDecoder::new(file);
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed).unwrap();
        assert_eq!(decompressed, "test content");
    }

    // Task 10: Saver gzip tests

    #[test]
    fn test_save_yaml_as_gzipped() {
        use crate::document::parser::parse_yaml;
        use flate2::read::GzDecoder;
        use std::io::Read;
        use tempfile::NamedTempFile;

        // Create JSON tree
        let json = r#"{"name": "Alice", "age": 30}"#;
        let root_node = parse_yaml(json).unwrap();
        let tree = YamlTree::new(root_node);
        let config = Config::default();

        // Save as .json.gz
        let temp_file = NamedTempFile::new().unwrap();
        let gz_path = temp_file.path().with_extension("json.gz");
        save_yaml_file(&gz_path, &tree, &config).unwrap();

        // Decompress and verify
        let file = fs::File::open(&gz_path).unwrap();
        let mut decoder = GzDecoder::new(file);
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed).unwrap();

        // Verify it's valid YAML
        let parsed: serde_yaml::Value = serde_yaml::from_str(&decompressed).unwrap();
        assert_eq!(parsed["name"], "Alice");
        assert_eq!(parsed["age"], 30);
    }

    #[test]
    #[ignore = "TODO: Phase 3 - Multi-document support"]
    fn test_save_yamll_as_gzipped() {
        use crate::document::node::YamlValue;
        use flate2::read::GzDecoder;
        use std::io::Read;
        use tempfile::NamedTempFile;

        // Create multi-document YAML tree manually
        let lines = vec![
            {
                let mut obj = IndexMap::new();
                obj.insert(
                    "id".to_string(),
                    YamlNode::new(YamlValue::Number(YamlNumber::Float(1.0))),
                );
                YamlNode::new(YamlValue::Object(obj))
            },
            {
                let mut obj = IndexMap::new();
                obj.insert(
                    "id".to_string(),
                    YamlNode::new(YamlValue::Number(YamlNumber::Float(2.0))),
                );
                YamlNode::new(YamlValue::Object(obj))
            },
            {
                let mut obj = IndexMap::new();
                obj.insert(
                    "id".to_string(),
                    YamlNode::new(YamlValue::Number(YamlNumber::Float(3.0))),
                );
                YamlNode::new(YamlValue::Object(obj))
            },
        ];
        let root = YamlNode::new(YamlValue::MultiDoc(lines));
        let tree = YamlTree::new(root);
        let config = Config::default();

        // Save as .yaml.gz
        let temp_file = NamedTempFile::new().unwrap();
        let gz_path = temp_file.path().with_extension("jsonl.gz");
        save_yaml_file(&gz_path, &tree, &config).unwrap();

        // Decompress and verify
        let file = fs::File::open(&gz_path).unwrap();
        let mut decoder = GzDecoder::new(file);
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed).unwrap();

        // Verify multi-document YAML format (one JSON per line)
        let lines: Vec<&str> = decompressed.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(
            serde_yaml::from_str::<serde_yaml::Value>(lines[0])
                .unwrap()
                .get("id")
                .unwrap(),
            1
        );
        assert_eq!(
            serde_yaml::from_str::<serde_yaml::Value>(lines[1])
                .unwrap()
                .get("id")
                .unwrap(),
            2
        );
        assert_eq!(
            serde_yaml::from_str::<serde_yaml::Value>(lines[2])
                .unwrap()
                .get("id")
                .unwrap(),
            3
        );
    }

    // Task 11: Format switching tests

    #[test]
    fn test_format_switching_json_to_gz() {
        use crate::document::parser::parse_yaml;
        use flate2::read::GzDecoder;
        use std::io::Read;
        use tempfile::NamedTempFile;

        // Create and save as .json
        let json = r#"{"test": "value"}"#;
        let root_node = parse_yaml(json).unwrap();
        let tree = YamlTree::new(root_node);
        let config = Config::default();

        let temp_file = NamedTempFile::new().unwrap();
        let yaml_path = temp_file.path().with_extension("json");
        save_yaml_file(&yaml_path, &tree, &config).unwrap();

        // Verify uncompressed
        let content = fs::read_to_string(&yaml_path).unwrap();
        assert!(content.contains("test"));

        // Save same tree as .json.gz
        let gz_path = temp_file.path().with_extension("json.gz");
        save_yaml_file(&gz_path, &tree, &config).unwrap();

        // Verify compressed
        let file = fs::File::open(&gz_path).unwrap();
        let mut decoder = GzDecoder::new(file);
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed).unwrap();
        assert!(decompressed.contains("test"));
    }

    #[test]
    fn test_format_switching_gz_to_json() {
        use crate::document::parser::parse_yaml;
        use flate2::read::GzDecoder;
        use std::io::Read;
        use tempfile::NamedTempFile;

        // Create and save as .json.gz
        let json = r#"{"test": "value"}"#;
        let root_node = parse_yaml(json).unwrap();
        let tree = YamlTree::new(root_node);
        let config = Config::default();

        let temp_file = NamedTempFile::new().unwrap();
        let gz_path = temp_file.path().with_extension("json.gz");
        save_yaml_file(&gz_path, &tree, &config).unwrap();

        // Verify compressed
        let file = fs::File::open(&gz_path).unwrap();
        let mut decoder = GzDecoder::new(file);
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed).unwrap();
        assert!(decompressed.contains("test"));

        // Save same tree as .json (uncompressed)
        let yaml_path = temp_file.path().with_extension("json");
        save_yaml_file(&yaml_path, &tree, &config).unwrap();

        // Verify uncompressed
        let content = fs::read_to_string(&yaml_path).unwrap();
        assert!(content.contains("test"));

        // Verify it's NOT gzip (won't start with gzip magic bytes)
        let raw_bytes = fs::read(&yaml_path).unwrap();
        assert_ne!(&raw_bytes[0..2], &[0x1f, 0x8b]); // gzip magic bytes
    }

    // Task 12: Backup preservation test

    #[test]
    fn test_backup_preserves_compression() {
        use crate::document::parser::parse_yaml;
        use flate2::read::GzDecoder;
        use std::io::Read;
        use tempfile::NamedTempFile;

        // Create initial .json.gz file
        let json = r#"{"version": 1}"#;
        let root_node = parse_yaml(json).unwrap();
        let tree = YamlTree::new(root_node);
        let config = Config::default();

        let temp_file = NamedTempFile::new().unwrap();
        let gz_path = temp_file.path().with_extension("json.gz");
        save_yaml_file(&gz_path, &tree, &config).unwrap();

        // Modify and save with backup enabled
        let json2 = r#"{"version": 2}"#;
        let root_node2 = parse_yaml(json2).unwrap();
        let tree2 = YamlTree::new(root_node2);
        let config_with_backup = Config {
            create_backup: true,
            ..Default::default()
        };
        save_yaml_file(&gz_path, &tree2, &config_with_backup).unwrap();

        // Verify backup was created
        let backup_path = gz_path.with_file_name(format!(
            "{}.bak",
            gz_path.file_name().unwrap().to_str().unwrap()
        ));
        assert!(backup_path.exists());

        // Verify backup is compressed (can decompress)
        let file = fs::File::open(&backup_path).unwrap();
        let mut decoder = GzDecoder::new(file);
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed).unwrap();

        // Verify backup contains original version
        let parsed: serde_yaml::Value = serde_yaml::from_str(&decompressed).unwrap();
        assert_eq!(parsed["version"], 1);

        // Verify new file contains updated version
        let file2 = fs::File::open(&gz_path).unwrap();
        let mut decoder2 = GzDecoder::new(file2);
        let mut decompressed2 = String::new();
        decoder2.read_to_string(&mut decompressed2).unwrap();
        let parsed2: serde_yaml::Value = serde_yaml::from_str(&decompressed2).unwrap();
        assert_eq!(parsed2["version"], 2);
    }
}
