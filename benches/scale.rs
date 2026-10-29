// SPDX-License-Identifier: Apache-2.0
//! Reading a large literate file stays fast and linear.
//!
//! The file is the shape of a downstream ledger batch: one plural with
//! thousands of keyed entries of fifteen fields and one nested entity
//! each, split over cola blocks of a hundred entries with prose between
//! them. The nested entity matters: it is created under its entry, and
//! a build that finds the entry by scanning its siblings is quadratic. The bench reads
//! it at several sizes, prints the time and the cost per entry, and
//! fails when the cost per entry grows with the size: a quadratic build
//! costs ten times more per entry at 10k than at 1k, a linear one the
//! same.
//!
//! Run with `cargo bench --bench scale`; `COLAP_BENCH_ROUNDS` sets the
//! number of timed rounds per size (default 5, the best one counts).

use colap::rustemo::Parser as _;
use std::fmt::Write as _;
use std::process::ExitCode;
use std::time::{Duration, Instant};

const FIELDS_PER_ENTRY: usize = 15;
const ENTRIES_PER_BLOCK: usize = 100;
const SIZES: [usize; 4] = [1_000, 2_000, 5_000, 10_000];
/// Per-entry cost at the largest size over the smallest, at most.
const MAX_GROWTH: f64 = 3.0;

/// A ledger batch with `entries` claims.
fn ledger(entries: usize) -> String {
    let mut out = format!("# Ledger batch\n\nA generated ledger with {entries} claims.\n");
    for block in (0..entries).step_by(ENTRIES_PER_BLOCK) {
        let _ = write!(
            out,
            "\nBatch {}.\n\n```cola\nclaim plural claims:\n",
            block / ENTRIES_PER_BLOCK
        );
        for i in block..(block + ENTRIES_PER_BLOCK).min(entries) {
            let _ = write!(
                out,
                "    cl-{i}:
        title: \"Claim number {i} about a thing\",
        source: \"src-{}\",
        kind: \"assertion\",
        confidence: 0.{:02},
        weight: {},
        verified: {},
        author: \"author-{}\",
        created: \"2026-10-{:02}\",
        status: \"open\",
        domain: \"knowledge\",
        score: {},
        ratio: 0.{:03},
        note: 'single quoted \"note\" {i}',
        retracted: false,
        revision: {}
        provenance:
            batch: {},
            checked: true
        ;
    ;
",
                i % 97,
                i % 100,
                i * 7 % 1000,
                i % 3 == 0,
                i % 13,
                i % 28 + 1,
                i % 50,
                i % 1000,
                i % 5,
                i / ENTRIES_PER_BLOCK,
            );
        }
        out.push_str(";\n```\n");
    }
    out
}

/// Read the text once and check the model is complete.
fn read(text: &str, entries: usize) -> Result<Duration, String> {
    let start = Instant::now();
    let parsed = colap::parse_literate_str(text, None).map_err(|e| e.to_string())?;
    let elapsed = start.elapsed();
    let model = parsed.model;
    let claims = model
        .find_entity_by_path("claims")
        .and_then(|id| model.get_node(id))
        .ok_or("no `claims` entity")?;
    let claims = claims.borrow();
    if claims.children.len() != entries {
        return Err(format!(
            "{} entries read, {entries} written",
            claims.children.len()
        ));
    }
    let fields: usize = claims
        .children
        .iter()
        .filter_map(|&id| model.get_node(id))
        .map(|node| node.borrow().fields.len())
        .sum();
    if fields != entries * FIELDS_PER_ENTRY {
        return Err(format!(
            "{fields} fields read, {} written",
            entries * FIELDS_PER_ENTRY
        ));
    }
    Ok(elapsed)
}

/// Where the time goes: finding the fences, parsing each block, and
/// building the model from the ASTs. The three are timed separately,
/// so their sum is one reading of the text.
fn breakdown(text: &str) {
    let start = Instant::now();
    let scan = colap::literate::scan(text);
    let scanned = start.elapsed();
    let parser = colap::cola::ColaParser::new();
    let mut parsing = Duration::ZERO;
    let mut building = Duration::ZERO;
    let mut builder = colap::ModelBuilder::new(None);
    for block in scan.cola_blocks() {
        let start = Instant::now();
        let ast = parser.parse(&block.body);
        parsing += start.elapsed();
        if let Ok(ast) = ast {
            let start = Instant::now();
            builder.add_block(&ast, block.body_line() - 1);
            building += start.elapsed();
        }
    }
    let start = Instant::now();
    let finished = builder.finish().is_ok();
    building += start.elapsed();
    let total = (scanned + parsing + building).as_secs_f64();
    let share = |d: Duration| 100.0 * d.as_secs_f64() / total;
    println!(
        "breakdown at {} bytes: scan {:.1} ms ({:.0}%), parse {:.1} ms ({:.0}%), build {:.1} ms ({:.0}%){}",
        text.len(),
        scanned.as_secs_f64() * 1e3,
        share(scanned),
        parsing.as_secs_f64() * 1e3,
        share(parsing),
        building.as_secs_f64() * 1e3,
        share(building),
        if finished { "" } else { " (model has errors)" },
    );
}

fn main() -> ExitCode {
    let rounds: usize = std::env::var("COLAP_BENCH_ROUNDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);
    println!(
        "{:>8} {:>8} {:>9} {:>10} {:>12}",
        "entries", "fields", "bytes", "best ms", "us/entry"
    );
    let mut per_entry = Vec::new();
    for entries in SIZES {
        let text = ledger(entries);
        let mut best = Duration::MAX;
        for _ in 0..rounds {
            match read(&text, entries) {
                Ok(elapsed) => best = best.min(elapsed),
                Err(message) => {
                    eprintln!("{entries} entries: {message}");
                    return ExitCode::FAILURE;
                }
            }
        }
        let us_per_entry = best.as_secs_f64() * 1e6 / entries as f64;
        println!(
            "{entries:>8} {:>8} {:>9} {:>10.1} {us_per_entry:>12.2}",
            entries * FIELDS_PER_ENTRY,
            text.len(),
            best.as_secs_f64() * 1e3,
        );
        per_entry.push(us_per_entry);
    }
    breakdown(&ledger(SIZES[SIZES.len() - 1]));
    let growth = per_entry[per_entry.len() - 1] / per_entry[0];
    println!(
        "per-entry cost grows {growth:.2}x from {} to {} entries (limit {MAX_GROWTH}x)",
        SIZES[0],
        SIZES[SIZES.len() - 1]
    );
    if growth > MAX_GROWTH {
        eprintln!("reading is not linear in the number of entries");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
