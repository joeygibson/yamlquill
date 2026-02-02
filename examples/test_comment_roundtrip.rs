use std::fs;
use yamlquill::config::Config;
use yamlquill::document::node::YamlValue;
use yamlquill::document::parser::parse_yaml_auto;
use yamlquill::document::tree::YamlTree;
use yamlquill::file::saver::save_yaml_file;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load YAML with comments
    let yaml_content = fs::read_to_string("/tmp/test_manual.yaml")?;
    println!("Original YAML:");
    println!("{}\n", yaml_content);

    // Parse
    let root = parse_yaml_auto(&yaml_content)?;
    let tree = YamlTree::new(root);

    // Save without modifications
    let config = Config::default();
    save_yaml_file("/tmp/test_output1.yaml", &tree, &config)?;

    let saved1 = fs::read_to_string("/tmp/test_output1.yaml")?;
    println!("Saved YAML (no modifications):");
    println!("{}\n", saved1);

    // Verify it can be parsed again
    let reloaded = parse_yaml_auto(&saved1)?;
    println!("Successfully reloaded!");

    // Check comment count in reloaded
    match reloaded.value() {
        YamlValue::Object(map) => {
            let comment_count = map.keys().filter(|k| k.starts_with("__comment_")).count();
            println!("Comments in reloaded: {}", comment_count);
        }
        _ => {}
    }

    Ok(())
}
