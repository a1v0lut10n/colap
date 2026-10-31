// SPDX-License-Identifier: Apache-2.0
//! The `colap` command.
//!
//! `colap check FILE...` validates files and prints every finding as
//! `path:line:col: severity: message` (or as a cola block with
//! `--format cola`); it writes nothing. `colap fmt FILE...` rewrites
//! the cola blocks of files into the canonical layout, in place
//! (`--check` only reports). Exit codes are lint-style: 0 clean, 1
//! errors, 2 usage. `colap generate` emits a Rust crate or module and
//! stays the default when no subcommand is given, so `colap FILE`
//! keeps working.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Arg, ArgAction, ArgMatches, Command};
use colap::diagnostic::{self, Diagnostic, Severity};
use colap::generator::{CodeGenerator, GenerationMode};

/// The `generate` arguments, shared by the subcommand and the top level.
fn generate_args(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("input")
            .help("Input .cola file or markdown containing cola code blocks; optional with --schema, then validated and copied in for the tests")
            .required_unless_present("schema")
            .value_parser(clap::value_parser!(PathBuf))
            .index(1),
    )
    .arg(
        Arg::new("schema")
            .long("schema")
            .value_name("SCHEMA")
            .value_parser(clap::value_parser!(PathBuf))
            .help("Generate from this schema's declarations instead of the input's values (docs/spec/schema.md)"),
    )
    .arg(
        Arg::new("colap-path")
            .long("colap-path")
            .value_name("DIR")
            .value_parser(clap::value_parser!(PathBuf))
            .help("Make the generated crate depend on this colap checkout instead of the published version"),
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
}

fn cli() -> Command {
    let cmd = Command::new("colap")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Validate cola files and generate typed Rust APIs from them")
        .arg(
            Arg::new("trace")
                .long("trace")
                .global(true)
                .action(ArgAction::SetTrue)
                .help("Print the parser's debug trace (debug builds only)"),
        )
        .subcommand(
            Command::new("check")
                .about("Validate files and report every finding; writes nothing")
                .arg(
                    Arg::new("files")
                        .value_name("FILE")
                        .required(true)
                        .num_args(1..)
                        .value_parser(clap::value_parser!(PathBuf)),
                )
                .arg(
                    Arg::new("format")
                        .long("format")
                        .value_parser(["text", "cola"])
                        .default_value("text")
                        .help("text: one `path:line:col: severity: message` per finding; cola: a cola block of findings"),
                )
                .arg(
                    Arg::new("schema")
                        .long("schema")
                        .value_name("SCHEMA")
                        .value_parser(clap::value_parser!(PathBuf))
                        .help("Validate each file against this schema, a cola document (docs/spec/schema.md)"),
                ),
        )
        .subcommand(
            Command::new("fmt")
                .about("Rewrite the cola blocks of files into the canonical layout, in place")
                .arg(
                    Arg::new("files")
                        .value_name("FILE")
                        .required(true)
                        .num_args(1..)
                        .value_parser(clap::value_parser!(PathBuf)),
                )
                .arg(
                    Arg::new("check")
                        .long("check")
                        .action(ArgAction::SetTrue)
                        .help("Write nothing; exit 1 if any block is not canonical"),
                ),
        )
        .subcommand(generate_args(
            Command::new("generate").about("Generate a Rust crate or module (the default command)"),
        ))
        .subcommand_negates_reqs(true)
        .args_conflicts_with_subcommands(true);
    generate_args(cmd)
}

/// rustemo's trace (debug builds only) prints unless `RUSTEMO_NOTRACE`
/// is set, and it reads the variable on every call.
fn set_trace(on: bool) {
    // SAFETY: called from `main` before any other thread exists, which
    // is the condition under which changing the process environment is
    // sound.
    unsafe {
        if on {
            std::env::remove_var("RUSTEMO_NOTRACE");
        } else {
            std::env::set_var("RUSTEMO_NOTRACE", "1");
        }
    }
}

fn main() -> ExitCode {
    env_logger::init();
    // Usage errors exit with 2 here, clap's own convention.
    let matches = cli().get_matches();
    set_trace(matches.get_flag("trace"));

    match matches.subcommand() {
        Some(("check", m)) => check(m),
        Some(("fmt", m)) => fmt(m),
        Some(("generate", m)) => generate(m),
        _ => generate(&matches),
    }
}

fn check(matches: &ArgMatches) -> ExitCode {
    let files = matches.get_many::<PathBuf>("files").into_iter().flatten();
    let mut findings = Vec::new();
    // A schema that fails to load is reported alone: nothing can be
    // validated against it.
    let schema = match matches.get_one::<PathBuf>("schema") {
        Some(path) => {
            let loaded = colap::load_schema_file(path);
            findings.extend(loaded.findings);
            match loaded.schema {
                Some(schema) if !findings.iter().any(|d| d.severity == Severity::Error) => {
                    Some(schema)
                }
                _ => {
                    print_findings(matches, &findings);
                    return ExitCode::from(1);
                }
            }
        }
        None => None,
    };
    for path in files {
        let mut report = colap::check_file(path);
        if let Some(schema) = &schema {
            report.validate(schema);
        }
        findings.extend(report.diagnostics());
    }
    print_findings(matches, &findings);

    if findings.iter().any(|d| d.severity == Severity::Error) {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn print_findings(matches: &ArgMatches, findings: &[Diagnostic]) {
    match matches.get_one::<String>("format").map(String::as_str) {
        Some("cola") => print!("{}", diagnostic::to_cola(findings)),
        _ => {
            for finding in findings {
                println!("{finding}");
            }
        }
    }
}

fn fmt(matches: &ArgMatches) -> ExitCode {
    let check_only = matches.get_flag("check");
    let files = matches.get_many::<PathBuf>("files").into_iter().flatten();
    let mut failed = false;
    for path in files {
        let formatted = match colap::format::format_file(path) {
            Ok(formatted) => formatted,
            Err(error) => {
                for finding in Diagnostic::from_error(&error) {
                    println!("{finding}");
                }
                failed = true;
                continue;
            }
        };
        if formatted.is_canonical() {
            continue;
        }
        if check_only {
            for line in &formatted.changed {
                println!(
                    "{}:{line}:1: error: not canonical; run `colap fmt`",
                    path.display()
                );
            }
            failed = true;
        } else if let Err(e) = std::fs::write(path, &formatted.text) {
            println!("{}: error: {e}", path.display());
            failed = true;
        }
    }
    if failed {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn generate(matches: &ArgMatches) -> ExitCode {
    if matches.contains_id("schema") {
        return generate_from_schema(matches);
    }
    let Some(input_path) = matches.get_one::<PathBuf>("input").cloned() else {
        // Unreachable through clap (the argument is required without a
        // schema), but a usage problem is what it would be.
        eprintln!("error: an input file is required");
        return ExitCode::from(2);
    };

    let crate_name = crate_name_for(matches, &input_path, None);

    let base_output_dir: PathBuf = matches
        .get_one::<String>("output")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("generated"));
    let mode = matches
        .get_one::<String>("mode")
        .map(String::as_str)
        .unwrap_or("crate");
    let output_dir = base_output_dir.join(&crate_name);

    // Markdown (`.md`) is literate; anything else is detected: bare
    // cola, or literate when it carries a cola fence.
    let report = colap::check_file(&input_path);
    for finding in report.diagnostics() {
        eprintln!("{finding}");
    }
    let model = match report.outcome {
        Ok(model) => model,
        Err(_) => return ExitCode::from(1),
    };

    log::info!(
        "Successfully built ConfigModel from {}",
        input_path.display()
    );
    println!("\nConfig Structure:\n{}", model.pretty_display());

    let generation_mode = match mode {
        "module" => GenerationMode::Module {
            output_file: output_dir.with_extension("rs"),
        },
        _ => GenerationMode::Crate {
            output_dir: output_dir.clone(),
            crate_name: crate_name.clone(),
        },
    };

    let result = CodeGenerator::new(model, generation_mode, input_path.clone())
        .map(|generator| match matches.get_one::<PathBuf>("colap-path") {
            Some(path) => generator.with_colap_path(path),
            None => generator,
        })
        .and_then(|mut generator| generator.generate());
    if let Err(e) = result {
        eprintln!("{}: error: {e:#}", input_path.display());
        return ExitCode::from(1);
    }

    log::info!("Successfully generated code to {}", output_dir.display());
    log::info!("Generated crate name: {}", crate_name);
    ExitCode::SUCCESS
}

/// The crate name: from the option, else the schema's `name`, else
/// the file stem, with `-config`, underscores replaced with hyphens
/// and lowercased.
fn crate_name_for(matches: &ArgMatches, file: &Path, schema_name: Option<&str>) -> String {
    match matches.get_one::<String>("crate-name") {
        Some(name) => name.clone(),
        None => {
            let stem = schema_name.unwrap_or_else(|| {
                file.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("generated")
            });
            let stem = stem.strip_suffix(".schema").unwrap_or(stem);
            format!("{}-config", stem.replace('_', "-").to_lowercase())
        }
    }
}

/// `generate --schema`: types from declarations. The input, when
/// given, is validated first and goes into the crate's tests.
fn generate_from_schema(matches: &ArgMatches) -> ExitCode {
    let Some(schema_path) = matches.get_one::<PathBuf>("schema").cloned() else {
        return ExitCode::from(2);
    };
    let loaded = colap::load_schema_file(&schema_path);
    for finding in &loaded.findings {
        eprintln!("{finding}");
    }
    let clean = loaded.is_clean();
    let Some(schema) = loaded.schema.filter(|_| clean) else {
        return ExitCode::from(1);
    };
    let schema_text = match std::fs::read_to_string(&schema_path) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("{}: error: {e}", schema_path.display());
            return ExitCode::from(1);
        }
    };

    let input_path = matches.get_one::<PathBuf>("input").cloned();
    if let Some(input) = &input_path {
        let mut report = colap::check_file(input);
        report.validate(&schema);
        for finding in report.diagnostics() {
            eprintln!("{finding}");
        }
        if !report.is_clean() {
            return ExitCode::from(1);
        }
    }

    let crate_name = crate_name_for(matches, &schema_path, schema.name.as_deref());
    let base_output_dir: PathBuf = matches
        .get_one::<String>("output")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("generated"));
    let output_dir = base_output_dir.join(&crate_name);
    let generation_mode = match matches.get_one::<String>("mode").map(String::as_str) {
        Some("module") => GenerationMode::Module {
            output_file: output_dir.with_extension("rs"),
        },
        _ => GenerationMode::Crate {
            output_dir: output_dir.clone(),
            crate_name: crate_name.clone(),
        },
    };

    let result = colap::generator::SchemaGenerator::new(
        schema,
        schema_text,
        schema_path.clone(),
        input_path,
        generation_mode,
    )
    .map(|generator| match matches.get_one::<PathBuf>("colap-path") {
        Some(path) => generator.with_colap_path(path),
        None => generator,
    })
    .and_then(|mut generator| generator.generate());
    if let Err(e) = result {
        eprintln!("{}: error: {e:#}", schema_path.display());
        return ExitCode::from(1);
    }
    log::info!("Successfully generated code to {}", output_dir.display());
    ExitCode::SUCCESS
}
