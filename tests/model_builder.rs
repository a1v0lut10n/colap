// SPDX-License-Identifier: Apache-2.0
use rustemo::Parser;
use colap::cola::ColaParser;
use colap::config_model::{ConfigNode, ConfigValue};
use colap::model_builder::ModelBuilder;
use std::fs;
use std::path::Path;

#[test]
fn test_model_builder_with_test_genite() {
    // Parse the test file
    let test_file = Path::new("tests/data/test_genite.md");
    let content = fs::read_to_string(test_file).expect("Failed to read test file");

    // Create a new parser and parse the content
    let parser = ColaParser::new();
    let parse_result = parser.parse(&content);
    assert!(parse_result.is_ok(), "Failed to parse test file");

    let ast = parse_result.unwrap();

    // Use ModelBuilder to convert AST to ConfigModel
    let model_result = ModelBuilder::build_config_model(&ast);
    assert!(model_result.is_ok(), "Failed to build model from AST");

    let model = model_result.unwrap();

    // Verify the model structure by checking key entities and fields
    // First verify the model contains the llm entity
    let llm_path = "llm";
    let llm_entity = model.find_entity_by_path(llm_path);
    assert!(llm_entity.is_some(), "Failed to find llm entity");

    // Check the openai entity under llm
    let openai_path = "llm/openai";
    let openai_entity = model.find_entity_by_path(openai_path);
    assert!(
        openai_entity.is_some(),
        "Failed to find openai entity under llm"
    );

    if let Some(openai_id) = openai_entity {
        // Check that api entity exists under openai
        let api_path = "llm/openai/api";
        let api_entity = model.find_entity_by_path(api_path);
        assert!(
            api_entity.is_some(),
            "Failed to find api entity under openai"
        );

        if let Some(api_id) = api_entity {
            // Check that key field exists and has the correct value
            let key = model.get_field_value(api_id, "key");
            assert!(key.is_some(), "Failed to find key field in api entity");

            if let Some(ConfigValue::String(value)) = key {
                assert_eq!(value, "some_api_key", "key field has incorrect value");
            } else {
                panic!("key field is not a String or has incorrect value");
            }

            // Check that type field exists and has the correct value
            let type_field = model.get_field_value(api_id, "type");
            assert!(
                type_field.is_some(),
                "Failed to find type field in api entity"
            );

            if let Some(ConfigValue::String(value)) = type_field {
                assert_eq!(value, "REST", "type field has incorrect value");
            } else {
                panic!("type field is not a String or has incorrect value");
            }
        }
    }

    // Check for nested models and plural entities
    let models_path = "llm/openai/model";
    let models_entity = model.find_entity_by_path(models_path);
    assert!(
        models_entity.is_some(),
        "Failed to find models (plural) entity"
    );

    if let Some(models_id) = models_entity {
        // Check for one of the specific models
        let gpt4_path = "llm/openai/model/gpt-4.1";
        let gpt4_entity = model.find_entity_by_path(gpt4_path);
        assert!(gpt4_entity.is_some(), "Failed to find gpt-4.1 model entity");

        if let Some(gpt4_id) = gpt4_entity {
            // Check that name field exists and has the correct value
            let name = model.get_field_value(gpt4_id, "name");
            assert!(
                name.is_some(),
                "Failed to find name field in gpt-4.1 model entity"
            );

            if let Some(ConfigValue::String(value)) = name {
                assert_eq!(value, "gpt-4.1", "name field has incorrect value");
            } else {
                panic!("name field is not a String or has incorrect value");
            }

            // Check that max_input_tokens field exists and has the correct value
            let tokens = model.get_field_value(gpt4_id, "max_input_tokens");
            assert!(
                tokens.is_some(),
                "Failed to find max_input_tokens field in gpt-4.1 model entity"
            );

            if let Some(ConfigValue::Integer(value)) = tokens {
                assert_eq!(value, 1047576, "max_input_tokens field has incorrect value");
            } else {
                panic!("max_input_tokens field is not an Integer or has incorrect value");
            }
        }
    }

    // Print the model structure for debugging
    println!("ConfigModel structure:\n{}", model);
}
