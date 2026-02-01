//! JSON file loading functionality.
//!
//! This module provides functions to load JSON documents from files or stdin,
//! parsing them into `YamlTree` structures that can be edited by yamlquill.

use crate::document::parser::{parse_yaml, parse_value};
use crate::document::tree::YamlTree;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Loads and parses a JSON file from the filesystem.
///
/// This function reads a file from disk and parses its contents as JSON,
/// returning a `YamlTree` structure ready for editing.
///
/// # Arguments
///
/// * `path` - The path to the JSON file to load
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok(YamlTree)` if the file was successfully loaded and parsed
/// - `Err(anyhow::Error)` if:
///   - The file could not be read (doesn't exist, permission denied, etc.)
///   - The file contents are not valid YAML
///
/// # Examples
///
/// ```no_run
/// use yamlquill::file::loader::load_yaml_file;
///
/// let tree = load_yaml_file("config.json").unwrap();
/// // tree is now ready for editing
/// ```
///
/// # Errors
///
/// This function will return an error if:
/// - The file path does not exist
/// - The file cannot be read (permissions, etc.)
/// - The file contents are not valid YAML
pub fn load_yaml_file<P: AsRef<Path>>(path: P) -> Result<YamlTree> {
    let path_ref = path.as_ref();

    // Check if file is gzipped
    let is_gzipped = path_ref
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "gz")
        .unwrap_or(false);

    // Read content (decompress if needed)
    let content = if is_gzipped {
        read_gzipped_file(path_ref)?
    } else {
        fs::read_to_string(path_ref).context("Failed to read file")?
    };

    // Determine format from filename (before .gz)
    let is_jsonl = determine_jsonl_format(path_ref);

    // Parse accordingly
    if is_jsonl {
        parse_yamll_content(&content)
    } else {
        parse_yaml(&content).context("Failed to parse YAML")
    }
}

/// Helper function to parse multi-document YAML content (newline-delimited JSON).
///
/// Each line must be a valid YAML value. Blank lines are skipped.
pub fn parse_yamll_content(content: &str) -> Result<YamlTree> {
    use crate::document::node::{YamlNode, YamlValue};

    let mut lines = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue; // Skip blank lines
        }

        let value: serde_yaml::Value = serde_yaml::from_str(line)
            .with_context(|| format!("Invalid YAML on line {}", line_num + 1))?;

        let node = parse_value(&value);
        lines.push(node);
    }

    if lines.is_empty() {
        anyhow::bail!("No valid YAML found in multi-document YAML content");
    }

    let root = YamlNode::new(YamlValue::MultiDoc(lines));
    Ok(YamlTree::new(root))
}

/// Loads and parses JSON from standard input.
///
/// This function reads from stdin until EOF and parses the contents as JSON,
/// returning a `YamlTree` structure ready for editing. This is useful for
/// piping JSON data into the editor.
///
/// The function automatically detects whether the input is regular YAML or
/// multi-document YAML format (newline-delimited JSON). It tries regular YAML first, and
/// if that fails, it attempts to parse as multi-document YAML.
///
/// # Returns
///
/// Returns a `Result` containing:
/// - `Ok(YamlTree)` if stdin was successfully read and parsed
/// - `Err(anyhow::Error)` if:
///   - Reading from stdin failed
///   - The input contents are not valid YAML or multi-document YAML
///
/// # Examples
///
/// ```no_run
/// use yamlquill::file::loader::load_yaml_from_stdin;
///
/// // Usage: echo '{"key": "value"}' | cargo run -- -
/// let tree = load_yaml_from_stdin().unwrap();
/// ```
///
/// # Errors
///
/// This function will return an error if:
/// - Reading from stdin fails
/// - The input contents are not valid YAML or multi-document YAML
pub fn load_yaml_from_stdin() -> Result<YamlTree> {
    use std::io::{self, Read};

    let mut buffer = Vec::new();
    io::stdin()
        .read_to_end(&mut buffer)
        .context("Failed to read from stdin")?;

    // Check for gzip magic bytes (0x1f 0x8b)
    let content = if buffer.starts_with(&[0x1f, 0x8b]) {
        decompress_gzip_bytes(&buffer)?
    } else {
        String::from_utf8(buffer).context("Invalid UTF-8 in stdin")?
    };

    // Try to parse as regular YAML first
    if let Ok(tree) = parse_yaml(&content) {
        return Ok(tree);
    }

    // If regular YAML parsing fails, try multi-document YAML format
    parse_yamll_content(&content)
        .context("Failed to parse YAML from stdin: input is neither valid YAML nor valid multi-document YAML")
}

/// Loads and parses a multi-document YAML (JSON Lines) file from the filesystem.
///
/// Each line in the file must be a valid YAML value. Blank lines are skipped.
/// The result is a YamlTree with a MultiDoc containing all lines.
pub fn load_jsonl_file<P: AsRef<Path>>(path: P) -> Result<YamlTree> {
    let content = fs::read_to_string(path.as_ref()).context("Failed to read multi-document YAML file")?;
    parse_yamll_content(&content)
}

/// Determines if file is multi-document YAML format based on filename.
///
/// Checks for .yaml or .yaml extension, handling .gz suffix correctly.
/// Examples:
/// - `data.yaml` → true
/// - `data.yaml.gz` → true
/// - `data.json.gz` → false
fn determine_jsonl_format<P: AsRef<Path>>(path: P) -> bool {
    let path_str = path.as_ref().to_string_lossy();

    // Remove .gz suffix if present
    let base = if let Some(stripped) = path_str.strip_suffix(".gz") {
        stripped
    } else {
        &path_str
    };

    base.ends_with(".yaml") || base.ends_with(".yaml")
}

/// Reads and decompresses a gzipped file.
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be opened
/// - The file is not valid gzip format (corrupted)
/// - The decompressed content is not valid UTF-8
fn read_gzipped_file<P: AsRef<Path>>(path: P) -> Result<String> {
    use flate2::read::GzDecoder;
    use std::io::Read;

    let file = fs::File::open(path).context("Failed to open gzipped file")?;
    let mut decoder = GzDecoder::new(file);
    let mut content = String::new();
    decoder
        .read_to_string(&mut content)
        .context("Failed to decompress gzipped file - file may be corrupted")?;
    Ok(content)
}

/// Decompresses gzip-encoded bytes to a UTF-8 string.
///
/// # Errors
///
/// Returns an error if:
/// - The bytes are not valid gzip format
/// - The decompressed content is not valid UTF-8
fn decompress_gzip_bytes(bytes: &[u8]) -> Result<String> {
    use flate2::read::GzDecoder;
    use std::io::Read;

    let mut decoder = GzDecoder::new(bytes);
    let mut content = String::new();
    decoder
        .read_to_string(&mut content)
        .context("Failed to decompress gzipped stdin")?;
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::node::YamlValue;

    #[test]
    fn test_parse_yamll_content_simple() {
        let content = r#"{"id":1,"name":"Alice"}
{"id":2,"name":"Bob"}
{"id":3,"name":"Charlie"}"#;

        let tree = parse_yamll_content(content).unwrap();

        match tree.root().value() {
            YamlValue::MultiDoc(lines) => {
                assert_eq!(lines.len(), 3);

                // Check first line has correct structure
                if let YamlValue::Object(fields) = lines[0].value() {
                    assert_eq!(fields.len(), 2);
                } else {
                    panic!("Expected object on line 1");
                }
            }
            _ => panic!("Expected MultiDoc"),
        }
    }

    #[test]
    fn test_parse_yamll_content_skips_blank_lines() {
        let content = r#"{"id":1}

{"id":2}

{"id":3}"#;

        let tree = parse_yamll_content(content).unwrap();

        match tree.root().value() {
            YamlValue::MultiDoc(lines) => {
                assert_eq!(lines.len(), 3);
            }
            _ => panic!("Expected MultiDoc"),
        }
    }

    #[test]
    fn test_parse_yamll_content_mixed_types() {
        let content = r#"{"type":"object"}
["array","values"]
42
"string value"
true
null"#;

        let tree = parse_yamll_content(content).unwrap();

        match tree.root().value() {
            YamlValue::MultiDoc(lines) => {
                assert_eq!(lines.len(), 6);

                // Verify each type
                assert!(matches!(lines[0].value(), YamlValue::Object(_)));
                assert!(matches!(lines[1].value(), YamlValue::Array(_)));
                assert!(matches!(lines[2].value(), YamlValue::Number(_)));
                assert!(matches!(lines[3].value(), YamlValue::String(_)));
                assert!(matches!(lines[4].value(), YamlValue::Boolean(_)));
                assert!(matches!(lines[5].value(), YamlValue::Null));
            }
            _ => panic!("Expected MultiDoc"),
        }
    }

    #[test]
    fn test_parse_yamll_content_empty() {
        let content = "";
        let result = parse_yamll_content(content);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No valid YAML found"));
    }

    #[test]
    fn test_parse_yamll_content_invalid_json_line() {
        let content = r#"{"valid":true}
{invalid json}
{"valid":false}"#;

        let result = parse_yamll_content(content);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid YAML on line 2"));
    }

    #[test]
    fn test_load_yaml_from_stdin_requires_actual_stdin() {
        // This test documents that load_yaml_from_stdin requires actual stdin
        // It cannot be easily tested in unit tests without mocking
        // The core multi-document YAML parsing logic is tested via parse_yamll_content tests
    }

    #[test]
    fn test_load_yaml_file_integration() {
        // Integration tests for file loading are in tests/file_tests.rs
        // This is just a placeholder to document the test structure
    }

    #[test]
    fn test_determine_jsonl_format() {
        assert!(determine_jsonl_format("data.yaml"));
        assert!(determine_jsonl_format("data.yaml"));
        assert!(determine_jsonl_format("path/to/data.yaml.gz"));
        assert!(determine_jsonl_format("path/to/data.yaml.gz"));
        assert!(!determine_jsonl_format("data.json"));
        assert!(!determine_jsonl_format("data.json.gz"));
    }

    #[test]
    fn test_read_gzipped_file() {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create temp file with gzipped YAML
        let yaml_content = r#"{"test": "value"}"#;
        let temp_file = NamedTempFile::new().unwrap();
        let gz_path = temp_file.path().with_extension("json.gz");

        // Write compressed content
        let file = fs::File::create(&gz_path).unwrap();
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(yaml_content.as_bytes()).unwrap();
        encoder.finish().unwrap();

        // Test decompression
        let decompressed = read_gzipped_file(&gz_path).unwrap();
        assert_eq!(decompressed, yaml_content);
    }

    #[test]
    fn test_read_gzipped_file_corrupted() {
        use tempfile::NamedTempFile;

        // Create file with .gz extension but invalid gzip data
        let temp_file = NamedTempFile::new().unwrap();
        let gz_path = temp_file.path().with_extension("json.gz");
        fs::write(&gz_path, b"not gzip data").unwrap();

        // Should return error with helpful message
        let result = read_gzipped_file(&gz_path);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("decompress") || err_msg.contains("corrupted"));
    }

    #[test]
    fn test_load_gzipped_json_file() {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create temp file with gzipped YAML
        let yaml_content = r#"{"name": "Alice", "age": 30}"#;
        let temp_file = NamedTempFile::new().unwrap();
        let gz_path = temp_file.path().with_extension("json.gz");

        // Write compressed content
        let file = fs::File::create(&gz_path).unwrap();
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(yaml_content.as_bytes()).unwrap();
        encoder.finish().unwrap();

        // Load and verify
        let tree = load_yaml_file(&gz_path).unwrap();

        // Verify structure
        if let YamlValue::Object(entries) = tree.root().value() {
            assert_eq!(entries.len(), 2);
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_load_gzipped_jsonl_file() {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create temp file with gzipped multi-document YAML
        let jsonl_content = r#"{"id":1,"name":"Alice"}
{"id":2,"name":"Bob"}
{"id":3,"name":"Charlie"}"#;
        let temp_file = NamedTempFile::new().unwrap();
        let gz_path = temp_file.path().with_extension("jsonl.gz");

        // Write compressed content
        let file = fs::File::create(&gz_path).unwrap();
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(jsonl_content.as_bytes()).unwrap();
        encoder.finish().unwrap();

        // Load and verify
        let tree = load_yaml_file(&gz_path).unwrap();

        // Verify it's multi-document YAML format
        if let YamlValue::MultiDoc(lines) = tree.root().value() {
            assert_eq!(lines.len(), 3);
        } else {
            panic!("Expected MultiDoc");
        }
    }
}
