<p align="center">
  <img src="static/img/colap-logo.svg" alt="Cola-p Logo" width="500">
</p>

# 🥤 colap – A Refreshingly Simple Configuration Language Parser & Code Generator

---

Tired of fighting with your config files? So were we.

**cola** is a lightweight, human-friendly configuration language designed to avoid YAML’s whitespace traps, JSON’s curly-brace fatigue, and TOML’s dotted-key spaghetti. It’s backed by [`colap`](https://crates.io/crates/colap), a Rust parser and code generator that makes working with configurations effortless.

Cola is:

✅ **Readable** – Clean nesting with minimal punctuation.\
✅ **Writable** – No indentation anxiety. No trailing comma drama.\
✅ **Composable** – Works standalone or embedded in Markdown as fenced `cola ... ` blocks.

---

## 🚀 Why cola?

### YAML: Tabs vs Spaces

Miss one indent, and your app silently loads nonsense.

### JSON: Curly Braces Forever

Nested configs turn into a forest of `{}` and `[]`.

### TOML: Dotted-Key Spaghetti

Deep nesting forces verbose dotted keys or confusing table headers.

### cola: Refreshingly Simple

- 🆓 **Not whitespace sensitive** – Tabs? Spaces? Mix them. Cola doesn’t care.
- ✅ **Minimal punctuation** – Just `:` for nesting and `;` to end blocks.
- 🏷️ **Infix plural syntax** – Explicitly name singular and plural forms for code generators.
- 📄 **Markdown-native** – Embed configs directly in documentation.

---

## 📖 Example Configuration

Here's a real-world configuration defining multiple LLM providers:

```cola
llm plural llms:
    openai:
        api:
            type: "REST",
            key: "some_api_key",
            base_url: "some_base_url"
        ;

        model plural models:
            gpt-4.1:
                name: "gpt-4.1",
                max_input_tokens: 1048576,
                output_price: 8.0,
                supports_vision: true
            ;
            gpt-4o:
                name: "gpt-4o",
                max_input_tokens: 1048576,
                output_price: 8.0,
                supports_vision: true
            ;
        ;
    ;
;
```

---

## 🆚 cola vs YAML / JSON / TOML

| Feature                      | YAML          | JSON         | TOML         | **cola**      |
|------------------------------|---------------|--------------|--------------|---------------|
| Whitespace sensitivity       | 😬 Yes         | 👍 No         | 👍 No         | ✅ No          |
| Curly brace clutter          | 👍 Minimal     | 😩 Lots       | 👍 Minimal    | ✅ Minimal     |
| Trailing commas allowed      | ❌ No          | ❌ No         | ✅ Yes        | ✅ Yes         |
| Multi-line strings           | 🤔 Hard        | 😕 Escapes    | ✅ Easy       | ✅ Easy        |
| Comments                     | ✅ `#`         | ❌ None       | ✅ `#`        | ✅ `//`, kept by `fmt` |
| Collections (arrays/maps)    | 😕 Indents     | 😩 `[]`, `{}` | 😕 Mixed      | ✅ `[]` lists, infix `plural` maps |
| Markdown embedding           | ❌ Awkward     | ❌ Awkward    | ❌ Awkward    | ✅ First-class |

---

The language is specified in [`docs/spec/cola.md`](docs/spec/cola.md), a
literate document that colap itself checks; a conformance corpus under
`tests/conformance/` holds the parser, the diagnostics and the highlighter to
it. Schemas are the one open decision listed there.

---

## 🎯 Design Philosophy

### 🔹 Why Semicolons?

Cola uses `;` as a block terminator:

- Prevents ambiguity at the end of nested blocks.
- Avoids silent parsing errors from missing indentation (YAML) or commas (JSON).
- Familiar and clear for both humans and parsers.

---

### 🔹 Why Infix `plural`?

```cola
model plural models:
    gpt-4:
        name: "gpt-4"
    ;
    gpt-4o:
        name: "gpt-4o"
    ;
;
```

- Makes singular and plural forms **explicit** in configuration.
- Eliminates brittle “add an `s`” heuristics in code generation.
- Supports irregular plurals (e.g. `person plural people`).
- Improves clarity: developers instantly know single vs multiple semantics.

---

## 🔧 Features

### Parser

- Parses standalone `.cola` files and Markdown files with embedded cola blocks
- Finds cola blocks with CommonMark's fence rules, so any other fenced block
  (`mermaid`, `toml`, a bare ```` ``` ````) is skipped and any prose is allowed
- Produces structured configuration models whose source locations point into
  the original file
- Reports errors as `path:line:col: message`
- Values: strings with escapes and `"""` block strings, numbers with digit
  separators and exponents kept exactly as written (no size limit, no
  rounding; machine types are the reader's choice), booleans, bare symbols (`basis: parsed`), lists
  (`tags: [a, "b", 3]`) and references (`@claims/cl-1`) that `colap check`
  resolves
- `//` comments inside cola blocks, kept in the model and written back in
  place by `colap fmt`
- Reads large files in linear time: a 10k-entry ledger (5 MB) in about
  0.4 s; `cargo bench --bench scale` guards it

### Code Generator

- Generates idiomatic Rust structs and collections (`HashMap`) from cola models
- Handles plural entities clearly
- Renames Rust keywords (e.g., `type` to `type_`)
- Provides constructors and accessor methods for easy integration

---

## 📦 Installation

Add `colap` to your Rust project:

```toml
[dependencies]
colap = "0.1"
```

---

## 📂 Codebase Structure

The colap codebase is organized into the following modules:

- `lib.rs`: The entry points — `parse_file`, `parse_str`, `parse_literate_str`
  and `parse_cola_str`
- `literate`: The markdown layer — a CommonMark-style fence scanner that hands
  only the bodies of ```` ```cola ```` blocks to the parser
- `error`: The error type, with file-accurate locations
- `parser`: Handles parsing of Cola syntax using rustemo
  - `cola.rs`: Generated parser code
  - `cola_actions.rs`: Parser actions
- `highlight`: A line lexer for editors, sharing the scanner's fence rules
- `model`: Contains the configuration model definitions
  - `config_model.rs`: Core configuration model and node types
  - `model_builder.rs`: Builds config model from parsed AST
  - `source_location.rs`: Source location tracking
- `generator`: Code generation for Rust structs
  - `generator_impl.rs`: Main code generator implementation
  - `templates/`: Handlebars templates for code generation
- `grammar`: Contains the rustemo grammar definition
  - `cola.rustemo`: Grammar rules for the Cola language

---

## 🚩 Usage

### Command Line Interface

```bash
USAGE:
    colap check [--format <text|cola>] <FILE>...
    colap fmt [--check] <FILE>...
    colap generate [OPTIONS] <input>
    colap [OPTIONS] <input>              # the same as `generate`

OPTIONS (generate):
    -m, --mode <MODE>              Generation mode: 'crate' or 'module' (default: crate)
    -n, --crate-name <NAME>        Name of the generated library crate (default: input-file-stem-config)
    -o, --output <DIR>             Base output directory (default: generated)

OPTIONS (all commands):
        --trace                    Print the parser's debug trace (debug builds only)
    -h, --help                     Print help information
    -V, --version                  Print version information
```

Exit codes are lint-style: **0** when every file is clean (warnings allowed),
**1** when any error was found, **2** for a usage error.

### Examples

#### Checking files

Validate literate files without generating anything:

```sh
colap check docs/*.md
```

Every finding is one line, `path:line:col: severity: message`, with the
location in the original file. All errors in a file are reported, not just
the first: a block that does not parse does not hide the next one, and every
duplicate is listed with both locations.

```text
ledger.md:12:8: error: Expected one of BooleanFalse, BooleanTrue, Semicolon, Identifier, Number, QuotedStringDouble, QuotedStringSingle.
ledger.md:21:5: error: duplicate entry `cl-1` in `claim`; first at 18:5
notes.md: warning: no cola blocks in this file
notes.md:3:1: warning: unclosed cola fence runs to the end of the file
config.md:5:5: warning: empty entity `llm/api`
```

`--format cola` prints the same findings as a cola block — a
`finding plural findings` collection with `severity`, `path`, `line`,
`column` and `message` fields — so a report can be fed back to colap or
embedded in a literate document.

The same checks run before every `generate`, which prints the findings to
stderr and exits 1 on errors.

#### Formatting files

Rewrite the cola blocks of literate files into the canonical layout, in
place; everything outside the fences stays byte for byte as it was, and a
collection split over several blocks stays split:

```sh
colap fmt docs/*.md
colap fmt --check docs/*.md    # writes nothing; exits 1 if a block is not canonical
```

The canonical layout is a pure function of the model, so formatting is
idempotent and files regenerated by tools come out byte-deterministic:

- four spaces of indentation per level;
- an entity with no children whose one-line form fits in 80 columns is
  written on one line (`database: url: "";`); otherwise one field per line,
  no trailing comma, and the closing `;` on its own line;
- fields before child entities, both in source order;
- floats always carry a decimal point (`8.0`), strings are double-quoted
  unless their text contains a double quote.

A file with errors is not formatted; `fmt` prints the findings and exits 1.
The same writer is available as `ConfigModel::to_cola()`.

**The order guarantee.** The model keeps fields and entries in source order,
across reopened blocks, and the generated plural wrappers iterate in that
order (`keys()`, `values()`, `iter()`), so appended entries show up at the
end of a diff.

#### Basic Crate Generation (Default)

Generate a complete Rust crate from a cola file:

```sh
colap path/to/myconfig.cola
```

This will:

1. Parse `myconfig.cola`
2. Generate code in the directory `generated/myconfig-config/`
3. Create a complete library crate with:
   - `Cargo.toml` with dependencies
   - `src/lib.rs` with generated structs
   - `tests/integration.rs` with integration tests
   - `README.md` with usage documentation

#### Module Generation

Generate a single Rust module file for embedding in your project:

```bash
colap path/to/myconfig.cola --mode module -o src -n config_parser
```

This will:

1. Parse `myconfig.cola`
2. Generate a single file `src/config_parser.rs`
3. Include inline unit tests within the module
4. Ready to be imported with `mod config_parser;` in your project

#### Custom crate name and output directory

```bash
colap path/to/myconfig.cola --crate-name my-custom-config --output custom/output/dir
```

This will:

1. Parse `myconfig.cola`
2. Generate code in the directory `custom/output/dir/my-custom-config/`
3. Create a library crate with generated structs in `custom/output/dir/my-custom-config/lib.rs`

## Code Generation Details

The code generator produces:

1. Rust structs for each entity type in the configuration
2. Collection structs (using `HashMap`) for plural entities
3. Accessor methods to retrieve fields and collections
4. Constructors for each entity type
5. Integration tests for validating parsing functionality

### Generated Code Structure

For a configuration file with entities like `Root`, `Llm`, and `Model`, the generator produces:

```rust
// Root entity struct
pub struct Root {
    llm: Llm,
    // other fields...
}

// Singular entity
pub struct Llm {
    provider: String,
    models: Models,  // plural entity reference
    // other fields...
}

// Plural entity (collection)
pub struct Models {
    entries: HashMap<String, Model>,
}

// Entity in a collection
pub struct Model {
    name: String,
    context_window: i64,
    // other fields...
}
```

### Type Handling

The generator maps Cola types to Rust types as follows:

| Cola Type | Rust Type |
|-----------|----------|
| String, Symbol, Reference | `String` |
| Number of integer kind (`8`) | `i64`, when the value fits |
| Number of float kind (`8.0`, `1e3`) | `f64`, the nearest double |
| Boolean | `bool` |
| List | `Vec<T>` of the elements' type, `Vec<ConfigValue>` when empty or mixed |
| Entity | Custom struct |
| Optional Entity | `Option<CustomStruct>` |

### Default Crate Name

If the `--crate-name` option is not provided, the crate name is derived from the input file:

1. Take the file stem (filename without extension)
2. Replace underscores with hyphens
3. Convert to lowercase
4. Append `-config` suffix

For example:

- `my_config.cola` → `my-config-config`
- `API_Model.md` → `api-model-config`

## Using the Generated Code

### Loading and Parsing a Configuration File

The generated code includes integration tests that demonstrate how to parse configuration files:

```rust
use my_config::Root;  // Generated crate name

// Load and parse a configuration file: markdown with ```cola fences,
// or a bare .cola file. Errors read `path:line:col: message`.
fn parse_config_file(path: &str) -> Root {
    let parsed = colap::parse_file(path).expect("Failed to parse configuration");
    for warning in &parsed.warnings {
        eprintln!("warning: {warning}");
    }
    Root::from_model(&parsed.model)
}

// Usage example
let config = parse_config_file("path/to/config.md");
```

The library exposes the same entry points for text already in memory:
`parse_literate_str` for markdown, `parse_cola_str` for bare cola, and
`parse_str`, which treats text as literate when it contains a cola fence and
as bare cola otherwise. Consumers do not need a `rustemo` dependency.

### Accessing Configuration Values

The generated code provides accessor methods for retrieving values:

```rust
// Get the LLM configuration
let llm = config.llm();

// Access a singular entity's properties
let provider = llm.provider();  // Returns a &String

// Access a collection (plural entity)
let models = llm.models();

// Get a specific model from the collection
if let Some(gpt4) = models.get("gpt-4.1") {
    let context_window = gpt4.context_window();  // Returns an i64
    println!("GPT-4.1 context window: {}", context_window);
}

// Iterate through all models
for (name, model) in models.iter() {
    println!("Model: {}, Context: {}", name, model.context_window());
}
```

### Optional Fields

Optional fields are represented as `Option<T>` types:

```rust
// Optional field access with safe pattern matching
if let Some(api) = llm.api() {
    if let Some(key) = api.key() {
        println!("API Key: {}", key);
    }
}

// Using map() for clean optional chaining
let api_type = llm.api().and_then(|api| api.type_()).unwrap_or("No API type specified");
```

### 📄 Embedded Markdown Support

Colap parses configurations directly from Markdown. Any fenced block whose
language is `cola` is parsed, supporting documentation-driven development.
The rest of the document is prose and is never looked at: headings, tables,
blockquotes, HTML, lines starting with inline code or a bare `#`, and fenced
blocks in any other language are all fine.

Fences follow [CommonMark](https://spec.commonmark.org/0.31.2/#fenced-code-blocks):

- an opening fence is three or more backticks or tildes, indented by at most
  three spaces, followed by an info string whose first word is the language
  (```` ```cola ````, `~~~cola`, ```` ```cola title="LLM providers" ````);
- the closing fence uses the same character and is at least as long, so a
  four-backtick block can quote a three-backtick cola example without it
  being parsed;
- a fence that is never closed runs to the end of the file and is reported
  as a warning naming its opening line.

Files named `.md` or `.markdown` are always read as literate documents; any
other file is read as literate when it contains a cola fence and as bare
cola otherwise.

The full rules are in the spec, section 2.

### 🔁 Reopening and duplicates

A name directly under a `plural` header is a **key**: it identifies an
entry. Every other entity name is a **slot**: it names a place in the
configuration. The rules follow that line.

- **Slots may be reopened.** The same declaration at the same path continues
  the same entity, so a collection can be written in batches with prose
  between them, and a section can be split by narrative:

  ```cola
  claim plural claims:
      cl-1: subject: "a";
  ;
  ```

  Some prose, then more entries of the same collection:

  ```cola
  claim plural claims:
      cl-2: subject: "b";
  ;
  ```

  Reopening a slot with a different declaration — another plural name, or
  singular where it was plural — is an error.
- **Keys are defined once.** A second `cl-1` under `claims`, in the same block
  or a later one, is an error. That also means a key cannot be reopened to
  batch a nested collection under it: entries of `llm/openai/models` are
  written in one block.
- **Fields are set once**, and an entity has one namespace: a name is a field
  or an entity, never both, and `model plural models` reserves both `model`
  and `models` in its parent.

Every duplicate is reported, not just the first, with both locations:

```text
ledger.md:21:5: duplicate entry `cl-1` in `claim`; first at 18:5
```

The design record `docs/design/2026-10-29-duplicate-key-semantics.md` has
the reasoning, and the spec, section 5, the rules in full.

---

## 📜 License

This project is licensed under the [Apache License 2.0](LICENSE).

Copyright 2025-2026 Hans Van Wesenbeeck (hvw@aivolution.ch)

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in this work by you shall be licensed under the
Apache License 2.0, without any additional terms or conditions (Section 5
of the license).

---
