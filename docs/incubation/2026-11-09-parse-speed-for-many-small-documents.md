---
date: 2026-11-09
type: incubation
---

# Parse speed, now that a consumer reads thousands of small documents

Captured 2026-11-09, from schedulai's Git store (SAI-0001-2). That store
keeps one record per file: a tessera is a Markdown file that opens with a
`cola` block of about a dozen fields. To show its grid it reads every
file. On the day it moved from YAML front matter to cola, the scan of
1,000 tesserae went from 13 ms to 41 ms. Hans asked for an item here to
do something about performance.

## A second shape of the same finding

The scale benchmark (COLA-0006,
`docs/journal/2026-10/2026-10-29-scale-benchmark.md`) already said it for
one large file: rustemo parsing is 94% of a reading, about 37 µs per
entry of fifteen fields, and "that is where any further speed-up lives".
schedulai is the other shape, many small documents, and lands on the
same number.

One tessera block, 411 bytes and 12 fields, on schedulai's development
machine (AMD Ryzen 7 5800X, Linux, release profile, best of five rounds
of 20,000):

| Step | Time |
|---|---|
| `ColaParser::new()` | 0.02 µs |
| `ColaParser::new().parse(block)`, to the AST | 39.3 µs |
| the same with one parser reused | 38.6 µs |
| `check_cola_str(block)`, to the model | 43.3 µs |
| `literate::scan(file)` | 0.4 µs |
| `check_literate_str(file)`, to the model | 44.2 µs |
| `model.to_cola()`, the canonical writer | 3.8 µs |

So for a small record:

- parsing is about 90% of a reading, the model 10%, the fence scan 1%;
- building the parser costs nothing, so reusing one is not the lever;
- it works out at roughly 3 µs a field, or 10 MB/s;
- writing is ten times faster than reading.

## Why it may matter

Not for schedulai today: 41 ms for 1,000 records is fine, and its
commits cost 2 ms each whatever the format. It starts to show at 10,000
records (0.4 s for a full scan), and schedulai rescans after every
change for now.

It matters more as a pattern. "One record per file, fields in cola" is
what makes cola attractive to a store, and the knowledge repositories
with their ledgers are the large-file version of the same thing. Both
pay per field, and at 3 µs a field cola reads at about a third of the
speed YAML front matter did in the same test.

## Where the time might go

Not profiled. What the generated code shows (`src/parser/cola.rs`):

- the lexer is rustemo's `StringLexer` over a table of recognizers, and
  seven of them are regular expressions: identifier, number,
  reference, the two quoted strings, the block string, and layout
  (whitespace and comments);
- the expressions are compiled once (`Lazy<Regex>`), so compilation is
  not the cost;
- each token is found by trying recognizers at the current position, and
  layout is matched again between tokens. A twelve-field record is about
  sixty tokens, which would put a token at around 0.6 µs.

That points at per-token overhead in lexing, but it is a guess until a
profile says so. The parser tables and the AST building could equally
be it.

## Ways this could go

Roughly in order of effort:

- **Profile first.** A flame graph of `benches/scale.rs` and of a
  small-document loop. Everything below depends on what it shows.
- **A small-document case in the benchmark.** `benches/scale.rs` guards
  linearity on one large file. A second table for 10,000 one-entity
  documents would guard the per-document cost, which no test sees today.
- **Cheaper recognizers.** If lexing is the cost: hand-written matchers
  for identifier, number, whitespace and the plain string in place of
  the regular expressions, keeping the grammar and the parser as they
  are. rustemo-compiler has a setting for a lexer supplied by the user
  (`LexerType::Custom`).
- **A hand-written lexer or parser.** cola's grammar is nine rules
  (spec section 4). A recursive-descent reader would be small and would
  also end the dependency on rustemo that
  `2026-09-21-standalone-generated-parsers.md` describes from another
  side. The conformance corpus is what would make that safe: it holds
  any reader to the parser's located terminals and diagnostics.
- **Skipping the model.** A consumer that wants twelve fields of one
  entity does not need the comment-keeping, location-carrying model.
  A lighter read path (fields of the first entity, values only) would
  save the 10% the model costs and some allocation. The smallest gain,
  and it splits the API.

## What would settle it

- The profile.
- A target. "As fast as YAML front matter for a twelve-field record"
  would be about 10 µs, a factor of four. Whether that is worth a
  hand-written lexer depends on whether anyone reaches 10,000 records.
- Whether diagnostics survive. Line and column on every error, every
  error in a file and not only the first: a faster reader that loses
  those is not a trade worth making.

## Links

- `docs/journal/2026-10/2026-10-29-scale-benchmark.md` — the large-file
  numbers and the 94%.
- `docs/incubation/2026-09-21-standalone-generated-parsers.md` — the
  rustemo dependency, from the consumer's side.
- schedulai: `crates/schedulai-store/src/format.rs` (how it reads and
  writes records) and
  `docs/catalyst/2026-11-09-source-of-truth-and-revision-model.md`
  (the scan measurements).
