// SPDX-License-Identifier: Apache-2.0
use std::process::exit;

use std::fs;
use std::path::Path;

fn main() {
    // Delete cola_actions.rs before regenerating
    let actions_path = Path::new("src/cola_actions.rs");
    if actions_path.exists() {
        if let Err(e) = fs::remove_file(actions_path) {
            eprintln!("Failed to delete src/cola_actions.rs: {e}");
            exit(1);
        }
    }

    let mut settings = rustemo_compiler::Settings::new();
    settings = settings.in_source_tree();
    settings = settings.builder_loc_info(true);
    settings = settings.notrace(true);  // Disable debug trace output
    if let Err(e) = settings.process_dir() {
        eprintln!("{e}");
        exit(1);
    }
}
