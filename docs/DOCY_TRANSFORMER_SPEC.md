# s1-format-docy: DOCY Binary Transformer Specification

## Purpose

A Rust crate that transforms `s1engine::DocumentModel` → OnlyOffice DOCY binary format.
This enables `sdkjs BinaryFileReader` to render documents with full fidelity — headers,
footers, images, tables, TOC, comments, footnotes — all handled natively by sdkjs.

## Architecture

```
DOCX bytes
  → s1-format-docx::read()         [existing, 296 tests]
  → DocumentModel                   [s1-model, 76 tests]
  → s1-format-docy::write()         [NEW CRATE]
  → DOCY binary (base64 string)
  → sdkjs OpenDocumentFromBin()     [native rendering]
  → Full fidelity display
```

## DOCY File Format Summary

```
Header:  DOCY;v5;{byte_count};{base64_binary}
Binary:  [table_count:u8][mt_items: type:u8 + offset:u32 × N][table_data...]
```

### Required Tables (write order)

| # | Type ID | Name | Required | s1-model Source |
|---|---------|------|----------|-----------------|
| 1 | 0 | Signature | Yes | Version constant (5) |
| 2 | 9 | Settings | Yes | DocumentDefaults, SectionProperties |
| 3 | 3 | Numbering | If lists | NumberingDefinitions |
| 4 | 5 | Style | Yes | Vec\<Style\> |
| 5 | 6 | Document | Yes | Body children (Paragraphs, Tables) |
| 6 | 4 | HdrFtr | If present | Header/Footer nodes |
| 7 | 8 | Comments | If present | CommentStart/End/Body nodes |
| 8 | 10 | Footnotes | If present | FootnoteRef/Body nodes |
| 9 | 11 | Endnotes | If present | EndnoteRef/Body nodes |
| 10 | 2 | Media | If images | MediaStore |
| 11 | 7 | Other | Optional | Theme (can be empty) |

## Node Type → DOCY Mapping

### Block-Level Elements (Document Table, type 6)

| s1-model NodeType | DOCY Type ID | Writer |
|-------------------|--------------|--------|
| Paragraph | c_oSerParType.Par (0) | write_paragraph() |
| Table | c_oSerParType.Table (3) | write_table() |
| TableOfContents | c_oSerParType.Sdt (15) | write_toc() |
| Section (last para) | c_oSerParType.sectPr (4) | write_section_props() |

### Inline Elements (inside Paragraph Content)

| s1-model NodeType | DOCY Run Type ID | Notes |
|-------------------|------------------|-------|
| Run | c_oSerRunType.run (0) | Contains rPr + Content |
| Text | (inside Content) | Written as string |
| LineBreak | c_oSerRunType.linebreak (5) | |
| PageBreak | c_oSerRunType.pagebreak (4) | |
| ColumnBreak | c_oSerRunType.columnbreak (18) | |
| Tab | c_oSerRunType.tab (2) | |
| Image | c_oSerRunType.image (6) or pptxDrawing (12) | Inline drawing |
| Field | c_oSerRunType.fldChar (29) + instrText (30) | |
| BookmarkStart | c_oSerParType.BookmarkStart (23) | At paragraph level |
| BookmarkEnd | c_oSerParType.BookmarkEnd (24) | At paragraph level |
| CommentStart | c_oSerParType.CommentStart (6) | At paragraph level |
| CommentEnd | c_oSerParType.CommentEnd (7) | At paragraph level |
| FootnoteRef | c_oSerRunType.footnoteReference (26) | |
| EndnoteRef | c_oSerRunType.endnoteReference (27) | |

## Attribute → DOCY Property Mapping

### Run Properties (rPr) — c_oSerProp_rPrType

| s1-model AttributeKey | DOCY Property | ID | Encoding |
|-----------------------|---------------|-----|----------|
| Bold | Bold | 0 | Bool |
| Italic | Italic | 1 | Bool |
| Underline | Underline | 2 | Byte (0=none,1=single,2=double,...) |
| Strikethrough | Strikeout | 3 | Bool |
| FontFamily | FontAscii | 4 | String |
| FontFamily | FontHAnsi | 5 | String (same value) |
| FontFamilyEastAsia | FontAE | 6 | String |
| FontFamilyCS | FontCS | 7 | String |
| FontSize | FontSize | 8 | Long (half-points = pts × 2) |
| FontSizeCS | FontSizeCS | 44 | Long (half-points) |
| Color | Color | 9 | WriteItem(color object) |
| HighlightColor | HighLight | 11 | Byte (highlight enum) or RGB |
| Superscript | VertAlign | 10 | Byte (1=superscript) |
| Subscript | VertAlign | 10 | Byte (2=subscript) |
| FontSpacing | Spacing | 14 | Long (twips = pts × 20) |
| BoldCS | BoldCs | 43 | Bool |
| ItalicCS | ItalicCs | 45 | Bool |
| Caps | Caps | 49 | Bool |
| SmallCaps | SmallCaps | 50 | Bool |
| Hidden | Vanish | 19 | Bool |
| Language | Lang | 40 | String |
| StyleId | RStyle | 13 | String |
| DoubleStrikethrough | DStrikeout | 42 | Bool |
| ThemeColor | (inside Color) | - | Theme color reference |

### Paragraph Properties (pPr) — c_oSerProp_pPrType

| s1-model AttributeKey | DOCY Property | ID | Encoding |
|-----------------------|---------------|-----|----------|
| Alignment | Jc | 5 | Byte (0=right,1=left,2=center,3=justify) |
| IndentLeft | Ind_Left | 3 | Long (twips = pts × 20) |
| IndentRight | Ind_Right | 4 | Long (twips) |
| IndentFirstLine | Ind_FirstLine | 39 | Long (twips, negative=hanging) |
| SpacingBefore | Spacing_Before | 9+sub | Long (twips) |
| SpacingAfter | Spacing_After | 9+sub | Long (twips) |
| LineSpacing | Spacing_Line | 9+sub | Depends on LineRule |
| KeepWithNext | KeepNext | 7 | Bool |
| KeepLinesTogether | KeepLines | 6 | Bool |
| PageBreakBefore | PageBreakBefore | 8 | Bool |
| WidowControl | WidowControl | 14 | Bool |
| StyleId | ParaStyle | 21 | String (style name) |
| ListInfo | NumPr | 37 | NumId(Long) + Ilvl(Long) |
| Background | Shd | 14 | WriteItem(shd object) |
| ParagraphBorders | PBdr | 32 | WriteItem(border object) |
| TabStops | Tabs | 38 | WriteItem(tab array) |
| OutlineLevel | OutlineLvl | 40 | Byte (0-8) |
| Bidi | Bidi | 46 | Bool |
| ContextualSpacing | ContextualSpacing | 1 | Bool |

### Table Properties (tblPr) — c_oSerProp_tblPrType

| s1-model AttributeKey | DOCY Property | ID | Encoding |
|-----------------------|---------------|-----|----------|
| TableWidth | TableW | 7 | WriteItem(width) |
| TableAlignment | Jc | 0 | Byte |
| TableBorders | TableBorders | 5 | WriteItem(borders) |
| TableLayout | TableLayout | 8 | Byte (0=auto,1=fixed) |
| TableIndent | tblInd | 15 | Long (twips) |
| TableDefaultCellMargins | TableCellMar | 6 | WriteItem(margins) |

### Cell Properties (tcPr) — c_oSerProp_cellPrType

| s1-model AttributeKey | DOCY Property | ID | Encoding |
|-----------------------|---------------|-----|----------|
| CellWidth | TableCellW | 3 | WriteItem(width) |
| VerticalAlign | VAlign | 5 | Byte (0=top,1=center,2=bottom) |
| CellBorders | TableCellBorders | 1 | WriteItem(borders) |
| CellBackground | Shd | 2 | WriteItem(shd) |
| ColSpan | GridSpan | 0 | Long |
| RowSpan | VMerge | 4 | Byte (1=restart,2=continue) |

### Section Properties (sectPr) — c_oSerProp_secPrType

| s1-model Field | DOCY Property | ID | Encoding |
|----------------|---------------|-----|----------|
| page_width | pgSz_W | 0 | Long (twips) |
| page_height | pgSz_H | 1 | Long (twips) |
| orientation | pgSz_Orient | 12 | Byte (0=portrait,1=landscape) |
| margin_top | pgMar_Top | 3 | Long (twips) |
| margin_bottom | pgMar_Bottom | 6 | Long (twips) |
| margin_left | pgMar_Left | 4 | Long (twips) |
| margin_right | pgMar_Right | 5 | Long (twips) |
| header_distance | pgMar_Header | 7 | Long (twips) |
| footer_distance | pgMar_Footer | 8 | Long (twips) |
| columns | cols (sub) | 10 | Byte + spacing |

## Unit Conversions (s1-model → DOCY)

| s1-model Unit | DOCY Unit | Conversion |
|---------------|-----------|------------|
| Points (f64) | Twips (i32) | × 20 |
| Points (f64) | Half-points (i32) | × 2 |
| Points (f64) | EMU (i32) | × 12700 |
| Color (r,g,b,a) | RGB bytes | Direct (3 bytes) |
| Alignment enum | Byte | Left→1, Center→2, Right→0, Justify→3 |

**CRITICAL**: sdkjs alignment values differ from standard:
- `0 = Right` (NOT left)
- `1 = Left`
- `2 = Center`
- `3 = Justify`

## Implementation Plan

### Phase 1: Core Binary Writer
- `DokyWriter` struct with `Vec<u8>` buffer
- Methods: `write_byte`, `write_long`, `write_bool`, `write_string2`, `write_string3`
- `write_item(type, closure)` — TLV pattern with auto length
- `write_item_with_length(closure)` — length-prefixed block
- Base64 encoding for final output

### Phase 2: Table Writers
1. `write_signature()` — version 5
2. `write_settings(model)` — doc defaults + compat
3. `write_styles(model)` — all styles with pPr + rPr
4. `write_numbering(model)` — abstract nums + instances
5. `write_document(model)` — body children (paragraphs, tables)
6. `write_headers_footers(model)` — section-linked headers/footers
7. `write_comments(model)` — comment data
8. `write_footnotes(model)` — footnote content
9. `write_endnotes(model)` — endnote content
10. `write_media(model)` — embedded images

### Phase 3: Property Writers
- `write_run_props(attrs)` — all rPr properties
- `write_para_props(attrs)` — all pPr properties
- `write_table_props(attrs)` — all tblPr properties
- `write_cell_props(attrs)` — all tcPr properties
- `write_row_props(attrs)` — all trPr properties
- `write_section_props(section)` — all sectPr properties

### Phase 4: Content Writers
- `write_paragraph(node)` — pPr + content (runs, breaks, images)
- `write_run(node)` — rPr + text content
- `write_table(node)` — tblPr + rows + cells
- `write_image(node, media)` — drawing with embedded image
- `write_hyperlink(node)` — hyperlink wrapper
- `write_bookmark(node)` — bookmark start/end
- `write_comment_range(node)` — comment start/end markers
- `write_footnote_ref(node)` — footnote reference
- `write_field(node)` — field code + result

### Phase 5: WASM Integration
- New method: `WasmDocument::to_docy() -> Result<String, JsError>`
- Returns base64 DOCY string ready for `OpenDocumentFromBin()`
- Adapter updated: `openDocx()` calls `doc.to_docy()` then `api.OpenDocumentFromBin('', docy)`

### Phase 6: Testing
- Unit tests for each writer function
- Round-trip test: DOCX → s1engine → DOCY → sdkjs → verify
- Fixture-based tests for each element type
- Comparison tests: original DOCX rendering vs DOCY rendering

## Crate Structure

```
crates/s1-format-docy/
├── Cargo.toml          (depends on s1-model only)
├── src/
│   ├── lib.rs          (pub fn write(model) -> Vec<u8>)
│   ├── writer.rs       (DocyWriter binary buffer)
│   ├── tables/
│   │   ├── mod.rs
│   │   ├── signature.rs
│   │   ├── settings.rs
│   │   ├── styles.rs
│   │   ├── numbering.rs
│   │   ├── document.rs
│   │   ├── headers_footers.rs
│   │   ├── comments.rs
│   │   ├── footnotes.rs
│   │   ├── media.rs
│   │   └── other.rs
│   ├── props/
│   │   ├── mod.rs
│   │   ├── run_props.rs
│   │   ├── para_props.rs
│   │   ├── table_props.rs
│   │   ├── cell_props.rs
│   │   ├── row_props.rs
│   │   └── section_props.rs
│   └── content/
│       ├── mod.rs
│       ├── paragraph.rs
│       ├── run.rs
│       ├── table.rs
│       ├── image.rs
│       ├── hyperlink.rs
│       ├── bookmark.rs
│       ├── comment.rs
│       ├── footnote.rs
│       └── field.rs
└── tests/
    ├── round_trip.rs
    ├── fixtures/
    └── each_element.rs
```

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| DOCY format undocumented | High | Studied from BinaryFileWriter source |
| Property ID mismatch | High | Test each property with sdkjs reader |
| Unit conversion errors | Critical | Convert all to twips/half-points at boundary |
| Missing optional tables | Medium | sdkjs handles missing tables gracefully |
| Image embedding | Medium | Use same format as BinaryFileWriter |
| Style resolution | Medium | Write complete style chain including defaults |
| Large binary size | Low | DOCY is compact by design |

## Dependencies

```toml
[dependencies]
s1-model = { workspace = true }
base64 = "0.22"   # for final encoding
```

No other dependencies. Format isolation maintained.
