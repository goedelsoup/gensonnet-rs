//! Traits tests

use super::*;
use std::collections::HashMap;

#[test]
fn test_processing_options() {
    let options = ProcessingOptions {
        include_docs: true,
        include_validation: true,
        generate_helpers: false,
        output_format: OutputFormat::Jsonnet,
        mode: ProcessingMode::Full,
    };

    assert!(options.include_docs);
    assert!(options.include_validation);
    assert!(!options.generate_helpers);
}

#[test]
fn test_generation_options() {
    let mut options = GenerationOptions {
        format: OutputFormat::Jsonnet,
        include_validation: true,
        include_docs: true,
        generate_helpers: true,
        template: None,
        custom: HashMap::new(),
    };

    options.custom.insert(
        "test_key".to_string(),
        serde_yaml::Value::String("test_value".to_string()),
    );

    assert_eq!(options.format, OutputFormat::Jsonnet);
    assert!(options.include_validation);
    assert!(options.custom.contains_key("test_key"));
}

#[test]
fn test_validation_options() {
    let options = ValidationOptions {
        strict: true,
        allow_unknown_fields: false,
        validate_references: true,
        custom_rules: HashMap::new(),
    };

    assert!(options.strict);
    assert!(!options.allow_unknown_fields);
    assert!(options.validate_references);
}
