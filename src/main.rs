// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Arg, ArgAction, Command};
use colap::parser::cola::ColaParser;
use colap::model::model_builder::ModelBuilder;
use rustemo::Parser;

use colap::generator::{CodeGenerator, GenerationMode};

fn main() -> Result<()> {
    env_logger::init();

    let matches = Command::new("colap")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Generate a typed Rust API for a Cola configuration model")
        .arg(
            Arg::new("input")
                .help("Input .cola file or markdown containing Cola code blocks")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Base output directory; <crate-name> directory will be created in this directory (default: generated)")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("crate-name")
                .short('n')
                .long("crate-name")
                .help("Name of the generated library crate (default: input-file-stem-config)")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("mode")
                .short('m')
                .long("mode")
                .help("Generation mode: 'crate' (default) or 'module'")
                .value_parser(["crate", "module"])
                .default_value("crate")
                .action(ArgAction::Set),
        )
        .get_matches();

    let input_path: PathBuf = matches.get_one::<String>("input").unwrap().into();

    // Determine crate name - either from CLI arg or based on input file
    let crate_name = match matches.get_one::<String>("crate-name") {
        Some(name) => name.clone(),
        None => {
            // Default to input file stem + "-config"
            let stem = input_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("generated");

            // Replace underscores with hyphens and convert to lowercase
            format!("{}-config", stem.replace('_', "-").to_lowercase())
        }
    };

    // Default base output directory if not supplied
    let default_base_output = PathBuf::from("generated");
    let base_output_dir: PathBuf = matches
        .get_one::<String>("output")
        .map(PathBuf::from)
        .unwrap_or(default_base_output);

    // Get the generation mode
    let mode = matches.get_one::<String>("mode").unwrap();
    
    // Create final output directory path by appending /<crate-name> to the base output
    let output_dir = base_output_dir.join(&crate_name);

    generate(input_path, output_dir, crate_name, mode.clone())
}

fn generate(input_path: PathBuf, output_dir: PathBuf, crate_name: String, mode: String) -> Result<()> {
    let source = std::fs::read_to_string(&input_path)
        .with_context(|| format!("Unable to read {}", input_path.display()))?;

    let _is_markdown = {
        let ext = input_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();
        ext == "md" || ext == "markdown"
    };

    // Parse the input using colap
    let parser = ColaParser::new();

    // For both markdown and cola files, we use the ColaParser
    // The parser is designed to handle both cola code blocks in markdown
    // and direct cola content
    let cola_ast = match parser.parse(&source) {
        Ok(ast) => ast,
        Err(e) => return Err(anyhow::anyhow!("Failed to parse input: {}", e)),
    };

    // Convert the AST to a ConfigModel using ModelBuilder
    let model = ModelBuilder::build_config_model(&cola_ast)
        .map_err(|e| anyhow::anyhow!("Failed to build model: {}", e))?;

    log::info!(
        "Successfully built ConfigModel from {}",
        input_path.display()
    );

    // Display the configuration using pretty_display
    println!("\nConfig Structure:\n{}", model.pretty_display());


    // GenerationMode is already imported at the top
    
    // Create the appropriate GenerationMode based on the mode parameter
    let generation_mode = match mode.as_str() {
        "module" => {
            // For module mode, create a single .rs file
            let module_file = output_dir.with_extension("rs");
            GenerationMode::Module {
                output_file: module_file,
            }
        }
        "crate" | _ => {
            // Default to crate mode
            GenerationMode::Crate {
                output_dir: output_dir.clone(),
                crate_name: crate_name.clone(),
            }
        }
    };
    
    let mut generator = CodeGenerator::new(
        model,
        generation_mode,
        input_path.clone(),
    )?;
    generator.generate()?;

    log::info!("Successfully generated code to {}", output_dir.display());
    log::info!("Generated crate name: {}", crate_name);

    Ok(())
}
