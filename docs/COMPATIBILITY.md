# Format Compatibility & Known Limitations

> Honest assessment of what s1engine handles well, what's partial, and what's not supported.
> Last updated: 2026-03-19

## DOCX (OOXML — ECMA-376)

### What Works Well
- Paragraphs with full formatting (bold, italic, underline, strikethrough, super/subscript)
- Font size, family, color, highlight
- Headings (H1-H6) with style IDs and inheritance chains
- Lists (bullet, numbered, multi-level up to 8 deep)
- Tables (including nested tables, cell shading, borders)
- Images (inline, with width/height, alt text)
- Hyperlinks (URL + tooltip)
- Headers and footers (default, first page, even/odd) with fields
- Section properties (page size, margins, orientation, columns)
- Comments with author/date and range markers
- Footnotes and endnotes with body content
- Bookmarks (start/end markers)
- Track changes (insert/delete/format change detection)
- Document metadata (title, author, subject, dates)
- Style definitions with basedOn inheritance
- Page breaks and section breaks
- Tab stops

### What's Partial
- **Namespace extensions (w14, w15)**: Detected with debug warnings, but Office 2016+ features not semantically modeled. Raw XML preserved for round-trip.
- **Complex table formatting**: gridSpan (column merge) supported; vMerge (row merge) basic. Irregular merges may lose structure.
- **Text effects**: Shadow, glow, outline attributes stored but not rendered.
- **Equations**: OMML equations preserved as raw XML but not converted to LaTeX.
- **Form controls**: Structured document tags (w:sdt) preserved as raw XML.

### What's Not Supported
- SmartArt diagrams (dropped on import)
- Charts (dropped on import)
- VBA macros (preserved in ZIP but not accessible)
- Digital signatures (not validated)
- Embedded OLE objects
- Custom XML parts

## ODT (ODF 1.2)

### What Works Well
- Paragraphs, runs, text formatting (same coverage as DOCX)
- Tables with columns and row structure
- Images with frames
- Lists with automatic numbering
- Styles (automatic + named, paragraph + character)
- TOC elements with source attributes
- Bookmarks and hyperlinks
- Footnotes and endnotes
- Document metadata
- Section properties

### What's Partial
- **Column widths**: Style names stored for round-trip but actual widths not resolved from auto-styles.
- **Nested lists**: Flattened to paragraphs with ListInfo (by design — flat model is intentional).
- **Drawing objects**: Non-image draw elements (shapes, text boxes) skipped with debug warning.

### What's Not Supported
- SVG/drawing objects (beyond images)
- Change tracking in ODF format
- Database fields
- Chart objects

## PDF

### Export
- Full page layout with Knuth-Plass line breaking
- Font embedding with subsetting
- Images (JPEG passthrough, PNG decode/re-encode)
- Proper ToUnicode CMap for text extraction
- JPEG color space detection (grayscale, CMYK, RGB)
- Page numbers and field substitution

### Export Limitations
- No incremental PDF updates (always full rewrite)
- No PDF/A compliance
- No digital signatures in export
- Vector graphics render as placeholders

### PDF Viewer (Editor)
- PDF.js-based rendering with text selection
- Annotation tools: highlight, comment, draw, text box, redact
- Signature placement (draw, type, upload)
- Annotation undo (Ctrl+Z)

## TXT & Markdown

### What Works Well
- Full round-trip for plain text
- Markdown: headings, bold, italic, code, links, lists, tables (GFM)
- Markdown: images with alt text

### Limitations
- Markdown tables don't preserve column widths
- No LaTeX math in Markdown

## Round-Trip Fidelity

Tested scenarios (all pass):
- DOCX → open → export DOCX → reopen → compare: styles, headings, lists, tables, images preserved
- ODT → open → export ODT → reopen → compare: same
- DOCX ↔ ODT cross-format: formatting preserved, style IDs mapped
- Multi-section documents with different page sizes: preserved
- Documents with 10+ named styles and basedOn chains: preserved
- Comments with author/date: preserved
- Footnotes/endnotes: preserved
- Headers/footers with page number fields: preserved

## Performance Characteristics

- Document open: ~10ms for typical 10-page DOCX
- Export DOCX: ~5ms for 10-page document
- Layout/pagination: ~50ms for 10-page document (full re-layout)
- WASM bundle: ~2.5MB (debug), ~800KB (release + wasm-opt)
- Memory: ~5MB baseline for empty document, ~20MB for 100-page document

## Test Coverage

- 1,390 tests across 15 crates
- Round-trip tests for DOCX styles, nested tables, mixed lists, multi-section
- Property-based tests (proptest) for document model operations
- CRDT convergence tests (3-way sync)
- Format parser edge cases (truncated files, empty elements, missing media)
