// SPDX-License-Identifier: Apache-2.0
use std::process::exit;
use std::fs;
use std::path::Path;
use std::io;

// Copy a file from src to dest, creating parent directories as needed
fn copy_file(src: &Path, dest: &Path) -> io::Result<u64> {
    // Create parent directories if they don't exist
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(src, dest)
}

fn main() {
    // Delete generated files before regenerating
    let actions_paths = [Path::new("src/parser/cola_actions.rs"), Path::new("src/cola_actions.rs")];
    let cola_paths = [Path::new("src/parser/cola.rs"), Path::new("src/cola.rs")];
    
    // Clean up any existing files
    for path in actions_paths.iter().chain(cola_paths.iter()) {
        if path.exists() {
            if let Err(e) = fs::remove_file(path) {
                eprintln!("Failed to delete {}: {e}", path.display());
                exit(1);
            }
        }
    }
    
    // Make sure src/cola.rustemo exists by copying from src/grammar if needed
    let grammar_src = Path::new("src/grammar/cola.rustemo");
    let grammar_dest = Path::new("src/cola.rustemo");
    if !grammar_dest.exists() && grammar_src.exists() {
        if let Err(e) = fs::copy(grammar_src, grammar_dest) {
            eprintln!("Failed to copy grammar file: {e}");
            exit(1);
        }
    }

    let mut settings = rustemo_compiler::Settings::new();
    settings = settings.in_source_tree();
    settings = settings.builder_loc_info(true);
    settings = settings.notrace(true);  // Disable debug trace output
    
    // Generate the parser code
    if let Err(e) = settings.process_dir() {
        eprintln!("{e}");
        exit(1);
    }
    
    // Move generated files to parser directory
    if let Err(e) = copy_file(Path::new("src/cola.rs"), Path::new("src/parser/cola.rs")) {
        eprintln!("Failed to move cola.rs: {e}");
        exit(1);
    }
    
    if let Err(e) = copy_file(Path::new("src/cola_actions.rs"), Path::new("src/parser/cola_actions.rs")) {
        eprintln!("Failed to move cola_actions.rs: {e}");
        exit(1);
    }
    
    // Clean up the original files
    if let Err(e) = fs::remove_file(Path::new("src/cola.rs")) {
        eprintln!("Failed to remove src/cola.rs: {e}");
        exit(1);
    }
    
    if let Err(e) = fs::remove_file(Path::new("src/cola_actions.rs")) {
        eprintln!("Failed to remove src/cola_actions.rs: {e}");
        exit(1);
    }
    
    println!("cargo:rerun-if-changed=src/grammar/cola.rustemo");
    println!("cargo:rerun-if-changed=src/cola.rustemo");
}
