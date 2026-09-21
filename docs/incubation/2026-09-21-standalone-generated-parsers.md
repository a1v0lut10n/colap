# Generated parsers should not make consumers depend on rustemo

A crate that adopts a colap-generated configuration parser today has to add
two dependencies, not one. chaiss's `chaiss-core/Cargo.toml` says it plainly:

```toml
colap = "0.2"
rustemo = "0.7.1"   # supplies the Parser trait colap's generated parser is invoked through
```

The generated code (and the library path via `colap::cola::ColaParser`) is
*invoked through* rustemo's `Parser` trait, so the trait must be in scope in
the consumer, so rustemo must be a direct dependency of the consumer — with
its version pinned in lockstep with whatever colap was built against. That
is an implementation detail leaking into every downstream Cargo.toml. It
also just cost colap an adoption: genite-brain's service work (GEN-026 in
genite) wanted the literate `brain.cola` config and chose to stay on TOML
until this is fixed.

Ways this could go, roughly in order of reach:

- **Re-export**: `pub use rustemo;` from colap, and generated code plus docs
  reference `colap::rustemo::Parser`. One-line fix, consumers drop the
  direct dep, version lockstep becomes colap's problem (where it belongs).
- **Facade**: colap (and each generated module) exposes a self-contained
  entry point — `parse_str(&str) -> Result<ConfigModel, Error>` /
  `MyConfig::from_str(&str)` — that uses the trait internally and never
  asks the consumer to import it. The trait disappears from the public
  surface entirely, which also frees colap to swap parser runtimes later
  without breaking anyone.
- **Both**: facade as the documented path, re-export as the escape hatch
  for consumers that want the raw AST.

The facade direction pairs naturally with the other adoption rough edges
chaiss already recorded (fence-less input silently yielding an empty model,
unknown keys silently ignored, rustemo's debug trace needing
`RUSTEMO_NOTRACE=1`): a single owned entry point is the obvious place to
turn those into real errors and to set the trace default, instead of every
consumer rediscovering them.
