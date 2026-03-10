# s1engine

A modern, modular document engine built in Rust. Read, write, edit, and convert documents — DOCX, ODT, PDF, and TXT.

Designed as an embeddable SDK for building document editors, converters, and collaborative editing applications.

## Status

**Pre-alpha** — Architecture and specification phase. Not yet usable.

## Features (Planned)

- **Multi-format**: DOCX (OOXML), ODT (ODF), PDF (export), TXT, DOC (via conversion)
- **Modular**: Use only what you need — just parsing? Just PDF export? Just the model?
- **CRDT-ready**: Document model designed for collaborative editing from day 1
- **Cross-platform**: Native (macOS/Linux/Windows), WASM (browser), C FFI
- **Safe**: Rust memory safety — no buffer overflows, no use-after-free
- **Fast**: Incremental layout, efficient format parsing, small WASM bundles

## Architecture

```
┌─────────────────────────────────────────────────┐
│              Consumer Applications                │
└─────────────────┬───────────────────────────────┘
                  │  Rust API / C FFI / WASM
┌─────────────────▼───────────────────────────────┐
│                s1engine (facade)                  │
├─────────────────────────────────────────────────┤
│  s1-ops       s1-layout       s1-convert         │
│  Operations   Page Layout     Format Conversion   │
├─────────────────────────────────────────────────┤
│                s1-model                           │
│          Core Document Model                      │
├──────────┬──────────┬──────────┬────────────────┤
│  format  │  format  │  format  │    format       │
│  -docx   │  -odt    │  -pdf    │    -txt         │
├──────────┴──────────┴──────────┴────────────────┤
│              s1-text (FFI Layer)                   │
│        HarfBuzz · FreeType · ICU                  │
└─────────────────────────────────────────────────┘
```

## Quick Start

> **Note**: s1engine is not yet published. These examples show the planned API.

### Open and Read

```rust
use s1engine::{Engine, Format};

let engine = Engine::new();
let doc = engine.open_file("report.docx")?;

println!("{}", doc.to_plain_text());

for para in doc.paragraphs() {
    println!("{:?}: {}", para.style_name(), para.text());
}
```

### Create a Document

```rust
use s1engine::{Engine, Format};

let engine = Engine::new();
let mut doc = engine.create_document();

doc.builder()
    .heading(1, "Hello World")
    .paragraph(|p| {
        p.text("This is ")
         .bold("s1engine")
         .text(" — a document engine in Rust.")
    })
    .build();

doc.save("output.docx", Format::Docx)?;
doc.save("output.pdf", Format::Pdf)?;
```

### Convert Between Formats

```rust
use s1engine::{Engine, Format};

let engine = Engine::new();
let doc = engine.open_file("input.docx")?;
doc.save("output.odt", Format::Odt)?;
doc.save("output.pdf", Format::Pdf)?;
doc.save("output.txt", Format::Txt)?;
```

### Cargo Feature Flags

```toml
[dependencies]
# Default: DOCX + ODT + TXT
s1engine = "0.1"

# Minimal: just DOCX parsing
s1engine = { version = "0.1", default-features = false, features = ["docx"] }

# Full: everything including PDF export
s1engine = { version = "0.1", features = ["pdf", "convert"] }
```

| Feature | Description | Default |
|---|---|---|
| `docx` | DOCX (OOXML) read/write | Yes |
| `odt` | ODT (ODF) read/write | Yes |
| `txt` | Plain text read/write | Yes |
| `pdf` | PDF export (requires layout + text shaping) | No |
| `convert` | Format conversion pipelines | No |
| `doc-legacy` | DOC → DOCX conversion | No |
| `ffi-c` | C API bindings | No |
| `ffi-wasm` | WASM bindings | No |
| `crdt` | CRDT collaboration primitives | No |

## Crate Structure

| Crate | Description |
|---|---|
| `s1engine` | Facade — high-level public API |
| `s1-model` | Core document model (tree, nodes, attributes, styles) |
| `s1-ops` | Operations, transactions, undo/redo |
| `s1-format-docx` | DOCX reader/writer |
| `s1-format-odt` | ODT reader/writer |
| `s1-format-pdf` | PDF exporter |
| `s1-format-txt` | Plain text reader/writer |
| `s1-convert` | Format conversion (incl. DOC → DOCX) |
| `s1-layout` | Page layout engine |
| `s1-text` | Text shaping, fonts, Unicode (C++ FFI) |

## Documentation

- [Overview](docs/OVERVIEW.md) — Vision, goals, non-goals
- [Architecture](docs/ARCHITECTURE.md) — System design and decisions
- [Specification](docs/SPECIFICATION.md) — Detailed technical spec
- [Roadmap](docs/ROADMAP.md) — Development phases and milestones
- [API Design](docs/API_DESIGN.md) — Public API surface and examples
- [Dependencies](docs/DEPENDENCIES.md) — External libraries and rationale

## Building

```bash
# Prerequisites
rustup install stable
# For PDF export (Phase 3+): HarfBuzz, FreeType, ICU system libraries

# Build
cargo build

# Test
cargo test

# Lint
cargo clippy -- -D warnings

# Format
cargo fmt
```

## Roadmap

| Phase | Timeline | Focus |
|---|---|---|
| 1. Foundation | Months 1-3 | Document model, operations, TXT, basic DOCX |
| 2. Rich Documents | Months 3-6 | Tables, images, lists, full DOCX, ODT |
| 3. Layout & Export | Months 6-9 | Text shaping, page layout, PDF export |
| 4. Collaboration | Months 9-14 | CRDT integration, conflict resolution |
| 5. Production | Months 14-18 | WASM, C FFI, hardening, release |

See [ROADMAP.md](docs/ROADMAP.md) for detailed milestones.

## License

Licensed under either of:

- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option.

## Contributing

s1engine is currently in early development. Contribution guidelines will be published with the first public release.
