// SPDX-License-Identifier: Apache-2.0
//! The `colap` command.
//!
//! `colap check FILE...` validates files and prints every finding as
//! `path:line:col: severity: message` (or as a cola block with
//! `--format cola`); it writes nothing. Exit codes are lint-style: 0
//! clean, 1 errors, 2 usage. `colap generate` emits a Rust crate or
//! module and stays the default when no subcommand is given, so
//! `colap FILE` keeps working.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Arg, ArgAction, ArgMatches, Command};
use colap::diagnostic::{self, Severity};
use colap::generator::{CodeGenerator, GenerationMode};

/// The `generate` arguments, shared by the subcommand and the top level.
fn generate_args(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("input")
            .help("Input .cola file or markdown containing cola code blocks")
            .required(true)
            .value_parser(clap::value_parser!(PathBuf))
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
        Some(("generate", m)) => generate(m),
        _ => generate(&matches),
    }
}

fn check(matches: &ArgMatches) -> ExitCode {
    let files = matches.get_many::<PathBuf>("files").into_iter().flatten();
    let mut findings = Vec::new();
    for path in files {
        findings.extend(colap::check_file(path).diagnostics());
    }

    match matches.get_one::<String>("format").map(String::as_str) {
        Some("cola") => print!("{}", diagnostic::to_cola(&findings)),
        _ => {
            for finding in &findings {
                println!("{finding}");
            }
        }
    }

    if findings.iter().any(|d| d.severity == Severity::Error) {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn generate(matches: &ArgMatches) -> ExitCode {
    let Some(input_path) = matches.get_one::<PathBuf>("input").cloned() else {
        // Unreachable through clap (the argument is required), but a
        // usage problem is what it would be.
        eprintln!("error: an input file is required");
        return ExitCode::from(2);
    };

    // Crate name: from the option, or the input file stem + "-config",
    // underscores replaced with hyphens and lowercased.
    let crate_name = match matches.get_one::<String>("crate-name") {
        Some(name) => name.clone(),
        None => {
            let stem = input_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("generated");
            format!("{}-config", stem.replace('_', "-").to_lowercase())
        }
    };

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
        .and_then(|mut generator| generator.generate());
    if let Err(e) = result {
        eprintln!("{}: error: {e:#}", input_path.display());
        return ExitCode::from(1);
    }

    log::info!("Successfully generated code to {}", output_dir.display());
    log::info!("Generated crate name: {}", crate_name);
    ExitCode::SUCCESS
}
