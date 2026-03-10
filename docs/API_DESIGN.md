# API Design

## Design Principles

1. **Simple things should be simple** — opening a document and exporting it is 3 lines of code
2. **Complex things should be possible** — full access to the document model for advanced use cases
3. **No panics** — all fallible operations return `Result`
4. **Zero-cost abstractions** — high-level API compiles down to efficient code
5. **Builder pattern** for construction, **iterator pattern** for queries
6. **Feature flags** for optional functionality — don't pay for what you don't use

---

## Cargo Feature Flags

```toml
[features]
default = ["docx", "odt", "txt"]

# Format support (each pulls in its crate)
docx = ["s1-format-docx"]
odt = ["s1-format-odt"]
pdf = ["s1-format-pdf", "s1-layout", "s1-text"]
txt = ["s1-format-txt"]

# Conversion
convert = ["s1-convert"]
doc-legacy = ["convert"]           # DOC → DOCX conversion

# Text processing (needed for layout and PDF)
text-shaping = ["s1-text"]

# FFI targets
ffi-c = []                         # C API
ffi-wasm = []                      # WASM bindings

# Collaboration
crdt = []                          # CRDT support (Phase 4)
```

Usage:
```toml
# Minimal: just DOCX parsing, no layout/PDF
s1engine = { version = "0.1", default-features = false, features = ["docx"] }

# Full: everything
s1engine = { version = "0.1", features = ["pdf", "convert"] }
```

---

## API Examples by Use Case

### Use Case 1: Open and Read a Document

```rust
use s1engine::{Engine, Format};

fn main() -> Result<(), s1engine::Error> {
    let engine = Engine::new();

    // Open from file (format auto-detected from extension)
    let doc = engine.open_file("report.docx")?;

    // Read metadata
    println!("Title: {:?}", doc.metadata().title);
    println!("Author: {:?}", doc.metadata().creator);

    // Get plain text
    println!("{}", doc.to_plain_text());

    // Iterate paragraphs
    for para in doc.paragraphs() {
        println!("Style: {:?}", para.style_name());
        for run in para.runs() {
            print!("{}", run.text());
            if run.is_bold() { print!(" [bold]"); }
        }
        println!();
    }

    Ok(())
}
```

### Use Case 2: Create a Document from Scratch

```rust
use s1engine::{Engine, Format};

fn main() -> Result<(), s1engine::Error> {
    let engine = Engine::new();
    let mut doc = engine.create_document();

    doc.builder()
        .metadata(|m| {
            m.title("Quarterly Report")
             .author("Engineering Team")
        })
        .heading(1, "Q4 2026 Report")
        .paragraph(|p| {
            p.text("Revenue grew by ")
             .bold("23%")
             .text(" compared to Q3.")
        })
        .heading(2, "Key Metrics")
        .table(3, 4, |t| {
            t.cell(0, 0, "Metric");
            t.cell(1, 0, "Q3");
            t.cell(2, 0, "Q4");
            t.cell(0, 1, "Users");
            t.cell(1, 1, "10,000");
            t.cell(2, 1, "15,000");
            t.cell(0, 2, "Revenue");
            t.cell(1, 2, "$1.2M");
            t.cell(2, 2, "$1.5M");
            t.cell(0, 3, "NPS");
            t.cell(1, 3, "72");
            t.cell(2, 3, "78");
        })
        .heading(2, "Conclusion")
        .paragraph(|p| p.text("Strong quarter with growth across all metrics."))
        .build();

    doc.save("report.docx", Format::Docx)?;
    doc.save("report.odt", Format::Odt)?;
    doc.save("report.pdf", Format::Pdf)?;
    doc.save("report.txt", Format::Txt)?;

    Ok(())
}
```

### Use Case 3: Edit an Existing Document

```rust
use s1engine::{Engine, Format};

fn main() -> Result<(), s1engine::Error> {
    let engine = Engine::new();
    let mut doc = engine.open_file("contract.docx")?;

    // Find and replace text
    let count = doc.find_replace("COMPANY_NAME", "Acme Corp")?;
    println!("Replaced {} occurrences", count);

    // Modify specific paragraph via transaction
    if let Some(para) = doc.paragraphs().nth(0) {
        let para_id = para.id();
        let mut txn = doc.begin_transaction("Update title");
        txn.set_text(para_id, "Updated Contract Title")?;
        txn.set_style(para_id, "Heading1")?;
        txn.commit()?;
    }

    // Append content
    {
        let mut txn = doc.begin_transaction("Add signature block");
        txn.append_paragraph(|p| {
            p.text("Signed: _________________________")
        })?;
        txn.append_paragraph(|p| {
            p.text("Date: _________________________")
        })?;
        txn.commit()?;
    }

    // Undo the last transaction
    doc.undo()?;

    doc.save("contract_updated.docx", Format::Docx)?;
    Ok(())
}
```

### Use Case 4: Format Conversion (CLI Tool)

```rust
use s1engine::{Engine, Format};
use std::path::Path;

fn main() -> Result<(), s1engine::Error> {
    let engine = Engine::new();
    let args: Vec<String> = std::env::args().collect();

    let input = &args[1];
    let output = &args[2];

    let doc = engine.open_file(input)?;
    let format = Format::from_extension(Path::new(output).extension())?;
    doc.save(output, format)?;

    println!("Converted {} -> {}", input, output);
    Ok(())
}
```

### Use Case 5: Batch Processing (Server)

```rust
use s1engine::Engine;

fn process_upload(engine: &Engine, data: &[u8]) -> Result<ProcessedDoc, s1engine::Error> {
    let doc = engine.open(data)?;

    Ok(ProcessedDoc {
        title: doc.metadata().title.clone(),
        author: doc.metadata().creator.clone(),
        text: doc.to_plain_text(),
        word_count: doc.to_plain_text().split_whitespace().count(),
        page_count: doc.layout()?.pages.len(),
        paragraphs: doc.paragraphs().count(),
    })
}
```

### Use Case 6: WASM (Browser)

```typescript
import { Engine, Format } from '@s1engine/wasm';

const engine = new Engine();

// Open a file from <input type="file">
const fileInput = document.getElementById('file') as HTMLInputElement;
fileInput.onchange = async () => {
    const file = fileInput.files![0];
    const buffer = new Uint8Array(await file.arrayBuffer());

    const doc = engine.open(buffer);
    console.log('Title:', doc.metadata().title);
    console.log('Text:', doc.toPlainText());

    // Export as PDF
    const pdfBytes = doc.export(Format.Pdf);
    downloadBlob(pdfBytes, 'output.pdf', 'application/pdf');

    doc.free(); // Explicit cleanup in WASM
};

// Create a document programmatically
const doc = engine.createDocument();
doc.builder()
    .heading(1, 'Hello from WASM')
    .paragraph('This document was created in the browser.')
    .build();

const docxBytes = doc.export(Format.Docx);
doc.free();
```

### Use Case 7: Collaboration (Phase 4+)

```rust
use s1engine::{Engine, CollabDocument};

// Initialize with unique replica ID
let engine = Engine::new();
let mut doc = CollabDocument::new(&engine, 42); // replica_id = 42

// Local edits produce operations
let ops = doc.apply_local(InsertText {
    node: paragraph_id,
    offset: 5,
    text: "inserted text".into(),
})?;

// Serialize operations for network transport
let encoded = ops.encode()?;
send_to_server(encoded);

// Receive and apply remote operations
let remote_ops = receive_from_server();
let decoded = Operations::decode(&remote_ops)?;
doc.apply_remote(decoded)?;

// Get cursor positions of other users (awareness)
let awareness = doc.awareness();
for (user_id, cursor) in awareness.cursors() {
    render_remote_cursor(user_id, cursor);
}
```

---

## Error Handling

```rust
/// Top-level error type (re-exported as s1engine::Error)
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Format error: {0}")]
    Format(#[from] FormatError),

    #[error("Layout error: {0}")]
    Layout(#[from] LayoutError),

    #[error("Operation error: {0}")]
    Operation(#[from] OperationError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}

#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    #[error("Invalid DOCX: {0}")]
    InvalidDocx(String),

    #[error("Invalid ODT: {0}")]
    InvalidOdt(String),

    #[error("Malformed XML at line {line}: {message}")]
    MalformedXml { line: u64, message: String },

    #[error("Missing required part: {0}")]
    MissingPart(String),

    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    #[error("Invalid ZIP archive: {0}")]
    InvalidZip(String),
}

#[derive(Debug, thiserror::Error)]
pub enum OperationError {
    #[error("Node not found: ({0}, {1})")]
    NodeNotFound(u64, u64),  // replica, counter

    #[error("Invalid position: node ({node_replica}, {node_counter}), offset {offset}")]
    InvalidPosition { node_replica: u64, node_counter: u64, offset: usize },

    #[error("Cannot apply operation: {0}")]
    InvalidOperation(String),

    #[error("Invalid parent-child relationship: {parent_type:?} cannot contain {child_type:?}")]
    InvalidHierarchy { parent_type: String, child_type: String },
}

#[derive(Debug, thiserror::Error)]
pub enum LayoutError {
    #[error("Font not found: {family} (bold={bold}, italic={italic})")]
    FontNotFound { family: String, bold: bool, italic: bool },

    #[error("Text shaping failed: {0}")]
    ShapingError(String),
}
```

---

## Versioning & Stability

- **Pre-1.0** (`0.x.y`): API may change between minor versions. Use exact version pins.
- **1.0+**: Semantic versioning. Public API stable within major versions.
- **Internal crates** (`s1-model`, `s1-ops`, etc.) have independent version numbers.
- **Facade crate** (`s1engine`) re-exports the stable API — consumers depend on this only.

---

## Naming Conventions

| Entity | Convention | Example |
|---|---|---|
| Crate names | `s1-{module}` | `s1-model`, `s1-format-docx` |
| Facade crate | `s1engine` | `s1engine` |
| Module names | `snake_case` | `line_break`, `table_layout` |
| Types | `PascalCase` | `NodeType`, `AttributeMap` |
| Functions | `snake_case` | `open_file`, `to_plain_text` |
| Constants | `SCREAMING_SNAKE` | `DEFAULT_FONT_SIZE` |
| Feature flags | `kebab-case` | `text-shaping`, `ffi-wasm` |
| Error variants | `PascalCase` | `NodeNotFound`, `InvalidDocx` |
| C API prefix | `s1_` | `s1_engine_new`, `s1_document_free` |
| WASM/NPM package | `@s1engine/` | `@s1engine/wasm` |
