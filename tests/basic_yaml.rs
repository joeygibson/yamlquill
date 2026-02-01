use yamlquill::document::parser::parse_yaml;
use yamlquill::document::node::YamlValue;

#[test]
fn test_parse_simple_yaml() {
    let yaml = r#"
name: Test
count: 42
enabled: true
"#;

    let node = parse_yaml(yaml).expect("Failed to parse YAML");

    match node.value() {
        YamlValue::Object(obj) => {
            assert_eq!(obj.len(), 3);

            // Check name
            let name = obj.get("name").expect("name field missing");
            match name.value() {
                YamlValue::String(s) => assert_eq!(s.as_str(), "Test"),
                _ => panic!("name should be string"),
            }

            // Check count
            let count = obj.get("count").expect("count field missing");
            match count.value() {
                YamlValue::Number(n) => assert_eq!(n.as_f64(), 42.0),
                _ => panic!("count should be number"),
            }

            // Check enabled
            let enabled = obj.get("enabled").expect("enabled field missing");
            match enabled.value() {
                YamlValue::Boolean(b) => assert!(*b),
                _ => panic!("enabled should be bool"),
            }
        }
        _ => panic!("Root should be object"),
    }
}

#[test]
fn test_parse_array() {
    let yaml = r#"
- Alice
- Bob
- Carol
"#;

    let node = parse_yaml(yaml).expect("Failed to parse YAML");

    match node.value() {
        YamlValue::Array(arr) => {
            assert_eq!(arr.len(), 3);
            match arr[0].value() {
                YamlValue::String(s) => assert_eq!(s.as_str(), "Alice"),
                _ => panic!("Array element should be string"),
            }
        }
        _ => panic!("Root should be array"),
    }
}
