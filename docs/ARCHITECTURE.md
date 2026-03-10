# Architecture

## System Overview

```
┌──────────────────────────────────────────────────────────────────┐
│                        Consumer Applications                      │
│              (Web Editor, Desktop App, CLI Tool, etc.)             │
└──────────────────────┬───────────────────────────────────────────┘
                       │ Public API (Rust / C FFI / WASM)
┌──────────────────────▼───────────────────────────────────────────┐
│                         s1engine (facade)                          │
│                  High-level API tying all modules                  │
├──────────────────────────────────────────────────────────────────┤
│                                                                    │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────────────────┐ │
│  │   s1-ops     │  │  s1-layout    │  │      s1-convert          │ │
│  │  Operations  │  │ Page Layout   │  │  Format Conversion       │ │
│  │  Undo/Redo   │  │ Pagination    │  │  DOC → DOCX pipeline     │ │
│  │  CRDT-ready  │  │ Text Flow     │  │                          │ │
│  └──────┬──────┘  └──────┬───────┘  └──────────────────────────┘ │
│         │                │                                         │
│  ┌──────▼──────────────────▼─────────────────────────────────────┐ │
│  │                      s1-model                                  │ │
│  │              Core Document Model (Tree/DOM)                    │ │
│  │          Nodes, Attributes, Styles, Metadata                   │ │
│  │              Unique IDs (CRDT-ready)                           │ │
│  └──────────────────────┬────────────────────────────────────────┘ │
│                         │                                          │
│  ┌──────────────────────▼────────────────────────────────────────┐ │
│  │                    Format I/O Layer                             │ │
│  │  ┌──────────────┐ ┌───────────┐ ┌─────────┐ ┌──────────────┐ │ │
│  │  │s1-format-docx│ │s1-fmt-odt │ │s1-f-pdf │ │s1-format-txt │ │ │
│  │  │  Read/Write  │ │Read/Write │ │Exp Only │ │  Read/Write  │ │ │
│  │  └──────────────┘ └───────────┘ └─────────┘ └──────────────┘ │ │
│  └───────────────────────────────────────────────────────────────┘ │
│                                                                    │
│  ┌───────────────────────────────────────────────────────────────┐ │
│  │                    s1-text (Text Processing)                   │ │
│  │         Font Loading · Text Shaping · Unicode/BiDi            │ │
│  └──────────────────────┬────────────────────────────────────────┘ │
│                         │ FFI                                      │
├─────────────────────────▼────────────────────────────────────────┤
│                  Native C/C++ Libraries                            │
│        HarfBuzz  ·  FreeType  ·  ICU  ·  Skia (optional)         │
└──────────────────────────────────────────────────────────────────┘
```

## Crate Structure

```
s1engine/
├── Cargo.toml                    # Workspace root
├── CLAUDE.md                     # AI development context (keep updated)
├── README.md                     # Project README
├── LICENSE-MIT                   # MIT license
├── LICENSE-APACHE                # Apache 2.0 license
├── crates/
│   ├── s1-model/                 # Core document model
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── node.rs           # Node types (paragraph, run, table, etc.)
│   │   │   ├── tree.rs           # Tree structure and traversal
│   │   │   ├── attributes.rs     # Node attributes and formatting
│   │   │   ├── styles.rs         # Style definitions and resolution
│   │   │   ├── metadata.rs       # Document metadata
│   │   │   └── id.rs             # Unique ID system (CRDT-ready)
│   │   └── Cargo.toml
│   │
│   ├── s1-ops/                   # Operations on the document model
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── operation.rs      # Operation types (insert, delete, format, etc.)
│   │   │   ├── cursor.rs         # Cursor and selection model
│   │   │   ├── history.rs        # Undo/redo stack
│   │   │   ├── transaction.rs    # Atomic operation batches
│   │   │   └── validate.rs       # Operation validation
│   │   └── Cargo.toml
│   │
│   ├── s1-format-docx/           # DOCX (OOXML) reader/writer
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── reader.rs         # DOCX → s1-model
│   │   │   ├── writer.rs         # s1-model → DOCX
│   │   │   ├── xml/              # XML parsing helpers
│   │   │   ├── relationships.rs  # OOXML relationship handling
│   │   │   ├── styles.rs         # OOXML style mapping
│   │   │   └── media.rs          # Embedded media (images, etc.)
│   │   └── Cargo.toml
│   │
│   ├── s1-format-odt/            # ODT (ODF) reader/writer
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── reader.rs         # ODT → s1-model
│   │   │   ├── writer.rs         # s1-model → ODT
│   │   │   └── styles.rs         # ODF style mapping
│   │   └── Cargo.toml
│   │
│   ├── s1-format-pdf/            # PDF export
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── exporter.rs       # s1-model → PDF (via layout tree)
│   │   │   ├── page.rs           # PDF page construction
│   │   │   ├── fonts.rs          # Font embedding
│   │   │   └── images.rs         # Image embedding
│   │   └── Cargo.toml
│   │
│   ├── s1-format-txt/            # Plain text reader/writer
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── reader.rs         # TXT → s1-model
│   │   │   └── writer.rs         # s1-model → TXT
│   │   └── Cargo.toml
│   │
│   ├── s1-convert/               # Format conversion pipelines
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── pipeline.rs       # Conversion pipeline orchestration
│   │   │   └── doc_to_docx.rs    # Legacy DOC → DOCX conversion
│   │   └── Cargo.toml
│   │
│   ├── s1-layout/                # Layout engine
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── page.rs           # Page geometry, margins, columns
│   │   │   ├── block.rs          # Block-level layout (paragraphs, tables)
│   │   │   ├── inline.rs         # Inline layout (text runs, images)
│   │   │   ├── line_break.rs     # Line breaking algorithm (Unicode UAX#14)
│   │   │   ├── pagination.rs     # Page break decisions
│   │   │   └── table.rs          # Table layout algorithm
│   │   └── Cargo.toml
│   │
│   ├── s1-text/                  # Text processing (C++ FFI layer)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── shaping.rs        # HarfBuzz text shaping
│   │   │   ├── fonts.rs          # FreeType font loading/metrics
│   │   │   ├── unicode.rs        # Unicode operations (BiDi, segmentation)
│   │   │   └── ffi/              # Raw FFI bindings
│   │   ├── build.rs              # C++ compilation and linking
│   │   └── Cargo.toml
│   │
│   └── s1engine/                 # High-level facade crate
│       ├── src/
│       │   ├── lib.rs            # Re-exports and high-level API
│       │   ├── document.rs       # Document handle (main entry point)
│       │   ├── builder.rs        # Document builder pattern
│       │   └── config.rs         # Engine configuration
│       └── Cargo.toml
│
├── ffi/
│   ├── c/                        # C header generation (cbindgen)
│   │   ├── src/lib.rs
│   │   ├── build.rs
│   │   └── Cargo.toml
│   └── wasm/                     # WASM bindings (wasm-bindgen)
│       ├── src/lib.rs
│       └── Cargo.toml
│
├── tests/
│   ├── integration/              # Cross-crate integration tests
│   └── fixtures/                 # Test documents (.docx, .odt, .pdf, .txt)
│
├── benches/                      # Performance benchmarks
└── docs/                         # Documentation
    ├── OVERVIEW.md
    ├── ARCHITECTURE.md
    ├── SPECIFICATION.md
    ├── ROADMAP.md
    ├── API_DESIGN.md
    └── DEPENDENCIES.md
```

## Core Design Decisions

### 1. Document Model: Tree with Unique IDs

The document model is a tree structure (similar to a DOM) where every node has a globally unique ID. This is critical for future CRDT support — every element must be independently addressable.

```
Document
├── Body
│   ├── Paragraph [id: (0, 1)]
│   │   ├── Run [id: (0, 2)] { bold: true }
│   │   │   └── Text "Hello "
│   │   └── Run [id: (0, 3)] { italic: true }
│   │       └── Text "world"
│   ├── Table [id: (0, 4)]
│   │   ├── Row [id: (0, 5)]
│   │   │   ├── Cell [id: (0, 6)]
│   │   │   │   └── Paragraph [id: (0, 7)] ...
│   │   │   └── Cell [id: (0, 8)]
│   │   │       └── Paragraph [id: (0, 9)] ...
│   │   └── Row [id: (0, 10)] ...
│   └── Paragraph [id: (0, 11)]
│       └── Run [id: (0, 12)]
│           └── Text "End of doc"
├── Styles
│   ├── ParagraphStyle "Heading1" { font_size: 24, bold: true }
│   └── CharacterStyle "Emphasis" { italic: true }
├── Headers/Footers
└── Metadata { title, author, created, modified }
```

**Node ID Strategy:**
- Each node gets a `NodeId` composed of `(replica_id, counter)`.
- For single-user mode, `replica_id` is always `0` — no overhead.
- When CRDT is enabled, `replica_id` differentiates users, enabling merge.
- This is the same approach used by Yjs, Automerge, and Diamond Types.

### 2. Operation-Based Editing

All mutations go through an **operation** layer — never direct tree manipulation. This is non-negotiable for CRDT/OT support.

```
// Every edit is an operation
Operation::InsertNode  { parent: (0,1), index: 1, node: Run { ... } }
Operation::DeleteNode  { target: (0,3) }
Operation::SetAttributes { target: (0,2), attrs: { bold: false } }
Operation::SplitNode   { target: (0,1), offset: 5 }
Operation::MergeNodes  { target: (0,1), with: (0,11) }
```

Benefits:
- **Undo/redo** is trivial — reverse operations
- **Collaboration** — operations can be broadcast and replayed
- **History** — full audit trail of changes
- **Validation** — operations are validated before application

### 3. Format I/O as Separate Crates

Each format (DOCX, ODT, PDF, TXT) is an independent crate that only depends on `s1-model`. This means:
- You can use DOCX support without pulling in PDF dependencies
- Each format can be developed/tested independently
- Adding new formats doesn't touch existing code

### 4. C++ Libraries via FFI (Not Rewriting)

We use battle-tested C/C++ libraries for text processing:

| Library | Purpose | Why Not Rewrite |
|---|---|---|
| **HarfBuzz** | Text shaping (ligatures, kerning, complex scripts) | 15+ years of development, handles Arabic/Devanagari/CJK correctly |
| **FreeType** | Font loading and glyph rasterization | Industry standard, handles every font format |
| **ICU** | Unicode BiDi, line breaking, normalization | Full Unicode compliance is enormous scope |

Rust wrappers: `harfbuzz-rs`, `freetype-rs`. For Unicode: prefer pure-Rust `unicode-bidi` + `unicode-linebreak` crates. Consider `icu4x` (pure Rust, by Unicode Consortium) if more ICU features are needed.

**Pure-Rust alternatives watch list**: `rustybuzz` (HarfBuzz port), `icu4x` (ICU replacement). If these mature sufficiently, we can drop all C/C++ dependencies.

### 5. Layout Engine: Incremental

The layout engine computes page geometry from the document model. It must be **incremental** — when a single paragraph changes, don't re-layout the entire document.

```
Document Model  →  Layout Tree  →  Render Output
(logical)          (physical)       (PDF/screen)

Paragraph (0,1) →  LayoutBlock {
                       x: 72, y: 100,
                       width: 468, height: 36,
                       lines: [
                         Line { glyphs: [...], y: 100 },
                         Line { glyphs: [...], y: 118 },
                       ]
                   }
```

### 6. CRDT-Ready, Not CRDT-First

The architecture supports CRDT from day 1 (unique IDs, operation-based editing), but the actual CRDT algorithm is Phase 4. We avoid the complexity of distributed systems until the core is solid.

When we implement CRDT, likely approach:
- **Diamond Types** (Rust-native CRDT for text, very performant)
- Or build on **Yrs** (Yjs Rust port) which handles tree CRDTs
- Extend to handle rich-text operations (formatting, structure)

### 7. Format Detection

When opening a document from bytes (not a file path), s1engine auto-detects the format:

| Magic Bytes | Format |
|---|---|
| `PK\x03\x04` (ZIP header) | DOCX or ODT (disambiguate by ZIP contents) |
| `%PDF` | PDF (reject — not supported for reading) |
| `\xD0\xCF\x11\xE0` (OLE2) | Legacy DOC → route to converter |
| UTF-8 BOM or printable ASCII | TXT |
| UTF-16 BOM | TXT (with encoding conversion) |

For ZIP files, check for `word/document.xml` (DOCX) or `content.xml` + `META-INF/manifest.xml` (ODT).

## Dependency Graph

```
s1engine (facade)
├── s1-model          (zero external deps, core types)
├── s1-ops            (depends on: s1-model)
├── s1-layout         (depends on: s1-model, s1-text)
├── s1-format-docx    (depends on: s1-model, quick-xml, zip)
├── s1-format-odt     (depends on: s1-model, quick-xml, zip)
├── s1-format-pdf     (depends on: s1-model, s1-layout, pdf-writer, subsetter)
├── s1-format-txt     (depends on: s1-model, encoding_rs)
├── s1-convert        (depends on: s1-format-docx, s1-format-odt, cfb)
└── s1-text           (depends on: harfbuzz-rs, freetype-rs, fontdb, unicode-bidi)
```

**Key principle**: `s1-model` has ZERO external dependencies. It is pure Rust data structures. Everything else is optional.

## Error Handling Strategy

- All public APIs return `Result<T, s1engine::Error>`
- `s1engine::Error` is an enum with variants per module (`FormatError`, `LayoutError`, `OperationError`)
- No panics in library code — ever
- Invalid documents produce warnings, not crashes (be lenient in parsing, strict in writing)

## Thread Safety

- `Document` is `Send + Sync` (can be shared across threads safely)
- Layout computation is parallelizable per-page
- Format I/O is single-threaded per document but multiple documents can be processed concurrently
