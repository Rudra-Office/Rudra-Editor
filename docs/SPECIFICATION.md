# Technical Specification

## 1. Document Model (`s1-model`)

### 1.1 Node Types

The document is a tree of typed nodes. Every node type maps to constructs found in both OOXML (DOCX) and ODF (ODT).

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum NodeType {
    // Root
    Document,

    // Structural
    Body,
    Section,          // Page layout properties (margins, orientation, columns)

    // Block-level
    Paragraph,
    Table,
    TableRow,
    TableCell,

    // Inline-level
    Run,              // Contiguous text with uniform formatting
    Text,             // Raw text content (leaf node)
    LineBreak,
    PageBreak,
    ColumnBreak,
    Tab,

    // Objects
    Image,            // Inline or floating image
    Drawing,          // Vector drawing / shape

    // Lists
    ListItem,         // Paragraph with list context

    // Headers/Footers
    Header,
    Footer,

    // Fields
    Field,            // Dynamic fields (page number, date, TOC, etc.)

    // Annotations
    BookmarkStart,
    BookmarkEnd,
    CommentStart,
    CommentEnd,
    CommentBody,      // Comment content container
}
```

**Node type hierarchy constraints** (enforced by validation):

| Parent | Allowed Children |
|---|---|
| `Document` | `Body`, `Header`, `Footer` |
| `Body` | `Section`, `Paragraph`, `Table` |
| `Section` | `Paragraph`, `Table` |
| `Paragraph` | `Run`, `LineBreak`, `PageBreak`, `Tab`, `Image`, `BookmarkStart`, `BookmarkEnd`, `CommentStart`, `CommentEnd`, `Field` |
| `Run` | `Text` (exactly one) |
| `Table` | `TableRow` |
| `TableRow` | `TableCell` |
| `TableCell` | `Paragraph`, `Table` (recursive) |
| `Header` | `Paragraph`, `Table` |
| `Footer` | `Paragraph`, `Table` |

### 1.2 Node Structure

```rust
/// A single node in the document tree.
#[derive(Debug, Clone)]
pub struct Node {
    /// Globally unique identifier (CRDT-ready)
    pub id: NodeId,

    /// Type of this node
    pub node_type: NodeType,

    /// Formatting and properties
    pub attributes: AttributeMap,

    /// Children (ordered). Empty for leaf nodes.
    pub children: Vec<NodeId>,

    /// Parent reference. None only for Document root.
    pub parent: Option<NodeId>,

    /// For Text nodes: the text content. None for non-text nodes.
    pub text_content: Option<String>,
}

/// CRDT-compatible unique identifier.
/// Composed of (replica_id, counter) — globally unique across all replicas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct NodeId {
    /// Replica/site identifier. 0 for single-user mode.
    pub replica: u64,

    /// Monotonically increasing counter per replica.
    pub counter: u64,
}

impl NodeId {
    pub const ROOT: NodeId = NodeId { replica: 0, counter: 0 };
}

/// Flexible attribute storage using typed keys.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AttributeMap {
    inner: HashMap<AttributeKey, AttributeValue>,
}

/// Typed attribute keys (prevents stringly-typed errors).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AttributeKey {
    // Run attributes
    FontFamily,
    FontSize,
    Bold,
    Italic,
    Underline,
    Strikethrough,
    Color,
    HighlightColor,
    Superscript,
    Subscript,
    FontSpacing,
    Language,

    // Paragraph attributes
    Alignment,
    IndentLeft,
    IndentRight,
    IndentFirstLine,
    SpacingBefore,
    SpacingAfter,
    LineSpacing,
    KeepWithNext,
    KeepLinesTogether,
    PageBreakBefore,
    Borders,
    Background,
    TabStops,
    StyleId,
    ListInfo,

    // Section attributes
    PageWidth,
    PageHeight,
    MarginTop,
    MarginBottom,
    MarginLeft,
    MarginRight,
    Columns,
    ColumnSpacing,
    Orientation,
    HeaderDistance,
    FooterDistance,

    // Table attributes
    TableWidth,
    TableAlignment,
    TableBorders,
    CellMargins,

    // Cell attributes
    CellWidth,
    VerticalAlign,
    CellBorders,
    CellBackground,
    ColSpan,
    RowSpan,

    // Image attributes
    ImageMediaId,
    ImageWidth,
    ImageHeight,
    ImageAltText,

    // Field attributes
    FieldType,
    FieldCode,

    // Link attributes
    HyperlinkUrl,
    HyperlinkTooltip,
    BookmarkName,

    // Comment attributes
    CommentId,
    CommentAuthor,
    CommentDate,
}

/// Typed attribute values.
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Color(Color),
    Alignment(Alignment),
    UnderlineStyle(UnderlineStyle),
    LineSpacing(LineSpacing),
    Borders(Borders),
    TabStops(Vec<TabStop>),
    ListInfo(ListInfo),
    PageOrientation(PageOrientation),
    TableWidth(TableWidth),
    VerticalAlignment(VerticalAlignment),
    MediaId(MediaId),
    FieldType(FieldType),
}
```

### 1.3 Supporting Types

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,  // 255 = fully opaque
}

impl Color {
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0, a: 255 };
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255, a: 255 };

    pub fn from_hex(hex: &str) -> Result<Color, ParseError>;
    pub fn to_hex(&self) -> String;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnderlineStyle {
    Single,
    Double,
    Thick,
    Dotted,
    Dashed,
    Wave,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineSpacing {
    Single,
    OnePointFive,
    Double,
    Exact(f64),    // Exact spacing in points
    AtLeast(f64),  // Minimum spacing in points
    Multiple(f64), // Multiple of line height
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageOrientation {
    Portrait,
    Landscape,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TableWidth {
    Auto,
    Fixed(f64),       // in points
    Percent(f64),     // 0.0 to 100.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalAlignment {
    Top,
    Center,
    Bottom,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TabStop {
    pub position: f64,         // in points from left margin
    pub alignment: TabAlignment,
    pub leader: TabLeader,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabAlignment { Left, Center, Right, Decimal }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabLeader { None, Dot, Dash, Underscore }

#[derive(Debug, Clone, PartialEq)]
pub struct ListInfo {
    pub level: u8,             // 0-8, nesting depth
    pub num_format: ListFormat,
    pub num_id: u32,           // References numbering definition
    pub start: Option<u32>,    // Override start number
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListFormat {
    Bullet,
    Decimal,           // 1, 2, 3
    LowerAlpha,        // a, b, c
    UpperAlpha,        // A, B, C
    LowerRoman,        // i, ii, iii
    UpperRoman,        // I, II, III
}

#[derive(Debug, Clone, PartialEq)]
pub struct Borders {
    pub top: Option<BorderSide>,
    pub bottom: Option<BorderSide>,
    pub left: Option<BorderSide>,
    pub right: Option<BorderSide>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BorderSide {
    pub style: BorderStyle,
    pub width: f64,       // in points
    pub color: Color,
    pub spacing: f64,     // space between border and content
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderStyle {
    None, Single, Double, Dashed, Dotted, Thick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MediaId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldType {
    PageNumber,
    PageCount,
    Date,
    Time,
    FileName,
    Author,
    TableOfContents,
    Custom,
}
```

### 1.4 Style System

Styles form an inheritance chain. A Run's effective formatting is resolved by:

```
Direct formatting → Character style → Paragraph style → Default style
(highest priority)                                      (lowest priority)
```

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    pub id: String,
    pub name: String,
    pub style_type: StyleType,
    pub parent_id: Option<String>,       // Inherits from parent style
    pub next_style_id: Option<String>,   // Style for next paragraph after Enter
    pub attributes: AttributeMap,        // All attributes this style defines
    pub is_default: bool,                // Is this the default for its type?
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleType {
    Paragraph,
    Character,
    Table,
    List,
}
```

**Style resolution algorithm:**
1. Start with the node's direct `attributes`
2. If the node references a character style (via `StyleId` attribute), overlay its attributes (direct wins on conflict)
3. Walk up to the node's paragraph ancestor. If paragraph references a paragraph style, overlay its attributes
4. If any style has a `parent_id`, recursively resolve the parent chain
5. Finally, overlay the document's default style for that type
6. Result: fully resolved `AttributeMap` with no `None` gaps for standard properties

### 1.5 Document Metadata

```rust
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub subject: Option<String>,
    pub creator: Option<String>,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub created: Option<String>,       // ISO 8601 datetime string
    pub modified: Option<String>,      // ISO 8601 datetime string
    pub revision: Option<u32>,
    pub language: Option<String>,      // BCP 47 language tag
    pub custom_properties: HashMap<String, String>,
}
```

### 1.6 Media Storage

```rust
#[derive(Debug, Clone, Default)]
pub struct MediaStore {
    items: HashMap<MediaId, MediaItem>,
    next_id: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MediaItem {
    pub id: MediaId,
    pub content_type: String,            // MIME type (e.g., "image/png")
    pub data: Vec<u8>,                   // Raw bytes
    pub filename: Option<String>,        // Original filename
}

impl MediaStore {
    /// Insert media. Returns existing MediaId if content already stored (dedup by hash).
    pub fn insert(&mut self, content_type: String, data: Vec<u8>) -> MediaId;

    /// Get media by ID.
    pub fn get(&self, id: MediaId) -> Option<&MediaItem>;
}
```

### 1.7 Document Tree Container

```rust
/// The complete document model.
#[derive(Debug, Clone)]
pub struct DocumentModel {
    /// All nodes, indexed by NodeId.
    nodes: HashMap<NodeId, Node>,

    /// The root node ID (always NodeId::ROOT, type Document).
    root: NodeId,

    /// ID generator for this replica.
    id_gen: IdGenerator,

    /// Named styles.
    styles: Vec<Style>,

    /// Document metadata.
    metadata: DocumentMetadata,

    /// Embedded media (images, etc.).
    media: MediaStore,

    /// Numbering/list definitions (for DOCX compatibility).
    numbering_definitions: Vec<NumberingDefinition>,
}

impl DocumentModel {
    pub fn new() -> Self;
    pub fn new_with_replica(replica_id: u64) -> Self;

    // Tree operations (used internally by s1-ops, not public)
    pub(crate) fn insert_node(&mut self, parent: NodeId, index: usize, node: Node) -> Result<(), ModelError>;
    pub(crate) fn remove_node(&mut self, id: NodeId) -> Result<Node, ModelError>;
    pub(crate) fn move_node(&mut self, id: NodeId, new_parent: NodeId, index: usize) -> Result<(), ModelError>;

    // Tree queries (public)
    pub fn node(&self, id: NodeId) -> Option<&Node>;
    pub fn root(&self) -> &Node;
    pub fn children(&self, id: NodeId) -> impl Iterator<Item = &Node>;
    pub fn parent(&self, id: NodeId) -> Option<&Node>;
    pub fn ancestors(&self, id: NodeId) -> impl Iterator<Item = &Node>;
    pub fn descendants(&self, id: NodeId) -> impl Iterator<Item = &Node>;  // DFS
    pub fn node_count(&self) -> usize;

    // Style queries
    pub fn styles(&self) -> &[Style];
    pub fn style_by_id(&self, id: &str) -> Option<&Style>;
    pub fn resolve_attributes(&self, node_id: NodeId) -> AttributeMap; // Fully resolved

    // Metadata and media
    pub fn metadata(&self) -> &DocumentMetadata;
    pub fn media(&self) -> &MediaStore;
}

/// Generates unique NodeIds for a given replica.
#[derive(Debug, Clone)]
struct IdGenerator {
    replica: u64,
    counter: u64,
}

impl IdGenerator {
    pub fn next(&mut self) -> NodeId;
}
```

---

## 2. Operations (`s1-ops`)

### 2.1 Operation Types

```rust
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Operation {
    /// Insert a new node as child of parent at given index.
    InsertNode {
        parent_id: NodeId,
        index: usize,
        node: Node,
    },

    /// Delete a node and all its descendants.
    DeleteNode {
        target_id: NodeId,
    },

    /// Move a node to a new parent/position.
    MoveNode {
        target_id: NodeId,
        new_parent_id: NodeId,
        new_index: usize,
    },

    /// Insert text into a Text node at given character offset.
    InsertText {
        target_id: NodeId,
        offset: usize,
        text: String,
    },

    /// Delete text from a Text node.
    DeleteText {
        target_id: NodeId,
        offset: usize,
        length: usize,
    },

    /// Set attributes on a node (merge with existing).
    SetAttributes {
        target_id: NodeId,
        attributes: AttributeMap,
    },

    /// Remove specific attributes from a node.
    RemoveAttributes {
        target_id: NodeId,
        keys: Vec<AttributeKey>,
    },

    /// Split a node at a given offset.
    /// For Paragraph: split into two paragraphs at the given child index.
    /// For Text: split the text content, creating a new Run + Text.
    SplitNode {
        target_id: NodeId,
        offset: usize,
    },

    /// Merge two adjacent sibling nodes into one.
    MergeNodes {
        target_id: NodeId,
        merge_with_id: NodeId,
    },

    /// Set document-level metadata.
    SetMetadata {
        key: String,
        value: Option<String>,
    },

    /// Add/replace embedded media.
    InsertMedia {
        media_item: MediaItem,
    },

    /// Set or update a style definition.
    SetStyle {
        style: Style,
    },

    /// Remove a style definition.
    RemoveStyle {
        style_id: String,
    },
}
```

### 2.2 Operation Execution

```rust
/// Apply an operation to a document model. Returns the inverse operation (for undo).
pub fn apply(model: &mut DocumentModel, op: &Operation) -> Result<Operation, OperationError>;

/// Validate an operation without applying it.
pub fn validate(model: &DocumentModel, op: &Operation) -> Result<(), OperationError>;
```

### 2.3 Transaction Model

```rust
/// A group of operations that form an atomic undo/redo unit.
#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: u64,
    pub operations: Vec<Operation>,
    pub inverse_operations: Vec<Operation>,  // For undo (in reverse order)
    pub timestamp: String,                    // ISO 8601
    pub description: String,                  // e.g., "Bold selection"
}
```

### 2.4 Cursor and Selection

```rust
/// A position in the document (between two characters or at node boundary).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// The node containing this position.
    pub node_id: NodeId,
    /// Character offset within text node, or child index within container.
    pub offset: usize,
}

/// A selection is an anchor + focus. When collapsed (anchor == focus), it's a cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    pub anchor: Position,
    pub focus: Position,
}

impl Selection {
    pub fn collapsed(pos: Position) -> Self;
    pub fn is_collapsed(&self) -> bool;
    pub fn is_forward(&self) -> bool;
    pub fn start(&self) -> Position;  // min(anchor, focus)
    pub fn end(&self) -> Position;    // max(anchor, focus)
}
```

### 2.5 Undo/Redo History

```rust
pub struct History {
    undo_stack: Vec<Transaction>,
    redo_stack: Vec<Transaction>,
    max_depth: usize,                // Configurable limit (default: 100)
}

impl History {
    pub fn push(&mut self, txn: Transaction);
    pub fn undo(&mut self, model: &mut DocumentModel) -> Result<(), OperationError>;
    pub fn redo(&mut self, model: &mut DocumentModel) -> Result<(), OperationError>;
    pub fn can_undo(&self) -> bool;
    pub fn can_redo(&self) -> bool;
    pub fn clear(&mut self);
}
```

---

## 3. Format: DOCX (`s1-format-docx`)

### 3.1 DOCX Structure

A .docx file is a ZIP archive containing XML files (OOXML / ECMA-376):

```
[Content_Types].xml
_rels/.rels
word/
  document.xml          <- Main document body
  styles.xml            <- Style definitions
  numbering.xml         <- List/numbering definitions
  settings.xml          <- Document settings
  fontTable.xml         <- Font declarations
  header1.xml           <- Header content
  footer1.xml           <- Footer content
  _rels/document.xml.rels  <- Relationships
  media/                <- Embedded images
    image1.png
docProps/
  core.xml              <- Dublin Core metadata (title, author, dates)
  app.xml               <- Application metadata (word count, etc.)
```

### 3.2 Reader Specification

**Input**: `&[u8]` or `impl Read + Seek`
**Output**: `Result<DocumentModel, DocxError>`

Steps:
1. Open ZIP archive (`zip` crate)
2. Parse `[Content_Types].xml` to discover parts
3. Parse `_rels/.rels` and `word/_rels/document.xml.rels` for relationships
4. Parse `docProps/core.xml` → `DocumentMetadata`
5. Parse `word/styles.xml` → `Vec<Style>`
6. Parse `word/numbering.xml` → numbering definitions
7. Parse `word/document.xml` → document tree (paragraphs, runs, tables, etc.)
8. Extract `word/media/*` → `MediaStore`
9. Parse `word/header*.xml` and `word/footer*.xml` if present
10. Assemble `DocumentModel`

**OOXML Elements Supported:**

| OOXML Element | Support Level | Maps To | Phase |
|---|---|---|---|
| `w:p` (paragraph) | Full | `Paragraph` node | 1 |
| `w:r` (run) | Full | `Run` node | 1 |
| `w:t` (text) | Full | `Text` node | 1 |
| `w:rPr` (run properties) | Most | `RunAttributes` | 1 |
| `w:pPr` (para properties) | Most | `ParagraphAttributes` | 1 |
| `w:style` | Most | `Style` | 1 |
| `w:br` (breaks) | Full | Break nodes | 1 |
| `w:tab` | Full | `Tab` node | 1 |
| `w:tbl` (table) | Full | `Table` node | 2 |
| `w:tr` (table row) | Full | `TableRow` node | 2 |
| `w:tc` (table cell) | Full | `TableCell` node | 2 |
| `w:tcPr` (cell properties) | Full | `CellAttributes` | 2 |
| `w:drawing` (images) | Basic | `Image` node | 2 |
| `w:hyperlink` | Full | Attribute on Run | 2 |
| `w:bookmarkStart/End` | Full | Bookmark nodes | 2 |
| `w:numPr` (numbering) | Full | `ListInfo` | 2 |
| `w:sectPr` (section props) | Full | `SectionAttributes` | 2 |
| `w:hdr/w:ftr` | Basic | Header/Footer | 2 |
| `w:commentRangeStart/End` | Basic | Comment nodes | 2 |
| `w:fldSimple` / `w:fldChar` | Basic | `Field` nodes | 2 |

**Elements Deferred:**
- `w:smartTag`, `w:sdt` (structured doc tags) — complex, low priority
- `mc:AlternateContent` — read preferred choice only
- VML drawings — legacy, skip or convert to basic shape
- Embedded OLE objects — skip with placeholder warning
- Math equations (OMML) — Phase 4+

### 3.3 Writer Specification

**Input**: `&DocumentModel`
**Output**: `Result<Vec<u8>, DocxError>` (ZIP bytes)

The writer produces a valid .docx that opens in Microsoft Word 2016+, LibreOffice 7+, and Google Docs.

Priorities:
1. **Round-trip fidelity**: Read a DOCX, write it back → preserve as much as possible
2. **Valid output**: Always produce a valid OOXML document (pass Open XML SDK validation)
3. **Compatibility**: Target Word 2016+ and LibreOffice 7+

### 3.4 Rust Dependencies

- `zip` — ZIP archive read/write
- `quick-xml` — Fast XML parsing and writing
- `base64` — For embedded content

---

## 4. Format: ODT (`s1-format-odt`)

### 4.1 ODT Structure

ZIP archive with XML (ODF / ISO 26300):

```
META-INF/manifest.xml
content.xml            <- Main content
styles.xml             <- Styles
meta.xml               <- Metadata
settings.xml           <- Settings
Pictures/              <- Embedded images
```

### 4.2 Mapping

| ODF Element | Maps To |
|---|---|
| `text:p` | `Paragraph` |
| `text:span` | `Run` |
| `text:h` | `Paragraph` with heading style |
| Text content | `Text` |
| `table:table` | `Table` |
| `table:table-row` | `TableRow` |
| `table:table-cell` | `TableCell` |
| `text:list` / `text:list-item` | List structure |
| `style:style` | `Style` |
| `draw:frame` + `draw:image` | `Image` |
| `text:a` | Hyperlink attribute |

### 4.3 Rust Dependencies

Same as DOCX: `zip`, `quick-xml`

---

## 5. Format: PDF Export (`s1-format-pdf`)

### 5.1 Approach

PDF export works from the **layout tree**, not directly from the document model:

```
DocumentModel → s1-layout (Layout Engine) → LayoutDocument → s1-format-pdf → PDF bytes
```

### 5.2 PDF Features by Phase

| Feature | Phase | Priority |
|---|---|---|
| Text rendering with correct glyph positioning | 3 | Must |
| Font embedding with subsetting | 3 | Must |
| Images (JPEG, PNG) | 3 | Must |
| Tables (borders, backgrounds) | 3 | Must |
| Page numbers | 3 | Should |
| Headers/Footers | 3 | Should |
| Hyperlinks (PDF annotations) | 3 | Should |
| Bookmarks / document outline | 3 | Nice |
| PDF metadata (title, author) | 3 | Nice |
| PDF/A compliance | 5 | Nice |

### 5.3 Rust Dependencies

- `pdf-writer` — Low-level PDF generation (pure Rust, proven by Typst)
- `subsetter` — Font subsetting (embed only used glyphs)
- `image` — Image decoding

---

## 6. Format: TXT (`s1-format-txt`)

### 6.1 Reader

- Detect encoding via BOM: UTF-8 BOM, UTF-16 LE/BE BOM
- If no BOM: attempt UTF-8, fall back to Latin-1
- Each line → `Paragraph` with single `Run` containing single `Text`
- Empty lines → empty `Paragraph` (no children)
- No formatting applied (all defaults)

### 6.2 Writer

- Serialize all `Text` nodes in document order
- Paragraphs separated by newline (configurable: `\n` or `\r\n`)
- Tables: columns separated by tab, rows by newline
- Strip all formatting
- Output encoding: UTF-8 (always)

---

## 7. Format Conversion (`s1-convert`)

### 7.1 DOC → DOCX Conversion

Legacy `.doc` files use Microsoft's OLE2 binary format.

**Option A: External Tool (Recommended for Phase 1-2)**
- Shell out to LibreOffice headless: `soffice --convert-to docx input.doc`
- Pros: Excellent coverage, well-tested
- Cons: Requires LibreOffice installed

**Option B: Native OLE2 Reader (Phase 3+)**
- Parse OLE2 container using `cfb` crate
- Parse Word binary format records
- Convert to `DocumentModel`
- Pros: No external dependency
- Cons: Very complex, months of work

**Decision**: Start with Option A, implement Option B later if needed.

### 7.2 General Conversion Pipeline

```rust
pub fn convert(
    input: &[u8],
    from: Format,
    to: Format,
    options: ConvertOptions,
) -> Result<Vec<u8>, ConvertError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Docx,
    Odt,
    Pdf,
    Txt,
    Doc,  // Input only — converted via pipeline
}

#[derive(Debug, Clone, Default)]
pub struct ConvertOptions {
    /// For PDF: page size override
    pub page_size: Option<(f64, f64)>,
    /// For TXT: line ending style
    pub line_ending: Option<LineEnding>,
}
```

Pipeline: `Source Format → DocumentModel → Target Format`

---

## 8. Layout Engine (`s1-layout`)

### 8.1 Layout Process

```
1. Resolve styles (compute effective attributes for every node)
2. Shape text (HarfBuzz: characters → positioned glyphs)
3. Break lines (Knuth-Plass algorithm preferred, greedy fallback)
4. Layout blocks (stack paragraphs with spacing, handle tables)
5. Paginate (break into pages, respect widows/orphans/keep-together)
6. Position headers/footers (substitute page numbers)
7. Output: LayoutDocument (pages → blocks → lines → glyph runs)
```

### 8.2 Layout Tree

```rust
pub struct LayoutDocument {
    pub pages: Vec<LayoutPage>,
}

pub struct LayoutPage {
    pub index: usize,          // 0-based page number
    pub width: f64,            // in points
    pub height: f64,
    pub content_area: Rect,    // Margins applied
    pub blocks: Vec<LayoutBlock>,
    pub header: Option<LayoutBlock>,
    pub footer: Option<LayoutBlock>,
}

pub struct LayoutBlock {
    pub source_id: NodeId,     // Link back to document model
    pub bounds: Rect,
    pub kind: LayoutBlockKind,
}

pub enum LayoutBlockKind {
    Paragraph { lines: Vec<LayoutLine> },
    Table { rows: Vec<LayoutTableRow> },
    Image { media_id: MediaId, bounds: Rect },
}

pub struct LayoutLine {
    pub baseline_y: f64,
    pub height: f64,
    pub runs: Vec<GlyphRun>,
}

pub struct GlyphRun {
    pub source_id: NodeId,     // Link to Run node
    pub font_id: FontId,
    pub font_size: f64,
    pub color: Color,
    pub glyphs: Vec<ShapedGlyph>,
}

pub struct ShapedGlyph {
    pub glyph_id: u16,        // Font glyph index
    pub x_advance: f64,
    pub y_advance: f64,
    pub x_offset: f64,
    pub y_offset: f64,
    pub cluster: u32,          // Maps back to source character index
}

pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
```

### 8.3 Incremental Layout

When a document is edited, only re-layout what changed:
1. Mark the modified paragraph as dirty
2. Re-layout that paragraph (re-shape, re-break lines)
3. If paragraph height changed → re-layout the page (re-stack blocks)
4. If page break changed → re-paginate from that page forward
5. Subsequent pages only re-paginate (blocks shift), no re-shaping

This is critical for editor performance — target < 5ms for single-edit re-layout.

---

## 9. Text Processing (`s1-text`)

### 9.1 HarfBuzz Integration

```rust
pub fn shape_text(
    text: &str,
    font: &Font,
    font_size: f64,
    features: &[FontFeature],       // OpenType features (e.g., liga, kern)
    language: Option<&str>,          // BCP 47
    direction: Direction,            // LTR or RTL
) -> Vec<ShapedGlyph>;

pub enum Direction { Ltr, Rtl }

pub struct FontFeature {
    pub tag: [u8; 4],    // e.g., b"liga", b"kern"
    pub value: u32,       // 0 = off, 1 = on
}
```

### 9.2 FreeType Integration

```rust
pub fn load_font(path: &Path) -> Result<Font, FontError>;
pub fn load_font_from_memory(data: &[u8]) -> Result<Font, FontError>;

pub struct Font { /* opaque */ }

impl Font {
    pub fn family_name(&self) -> &str;
    pub fn style_name(&self) -> &str;
    pub fn is_bold(&self) -> bool;
    pub fn is_italic(&self) -> bool;
    pub fn metrics(&self, size: f64) -> FontMetrics;
    pub fn glyph_index(&self, ch: char) -> Option<u16>;
    pub fn has_glyph(&self, ch: char) -> bool;
}

pub struct FontMetrics {
    pub ascent: f64,       // Distance from baseline to top
    pub descent: f64,      // Distance from baseline to bottom (negative)
    pub line_gap: f64,     // Extra spacing between lines
    pub units_per_em: u16,
}
```

### 9.3 Font Discovery

```rust
pub struct FontDatabase { /* wraps fontdb */ }

impl FontDatabase {
    pub fn new() -> Self;                   // Loads system fonts
    pub fn with_fonts_dir(path: &Path) -> Self;
    pub fn find(&self, family: &str, bold: bool, italic: bool) -> Option<Font>;
    pub fn fallback(&self, ch: char) -> Option<Font>;  // Find font that has this glyph
}
```

### 9.4 Unicode Processing

```rust
/// Determine text direction for BiDi text (Unicode UAX #9).
pub fn bidi_resolve(text: &str) -> Vec<BidiRun>;

pub struct BidiRun {
    pub start: usize,      // Byte offset in text
    pub end: usize,
    pub direction: Direction,
    pub level: u8,          // BiDi embedding level
}

/// Find valid line break opportunities (Unicode UAX #14).
pub fn line_break_opportunities(text: &str) -> Vec<BreakOpportunity>;

pub struct BreakOpportunity {
    pub offset: usize,     // Byte offset where break is allowed
    pub mandatory: bool,   // true for hard breaks (e.g., \n)
}
```

---

## 10. Engine Facade (`s1engine`)

### 10.1 Primary API

```rust
/// The main entry point for s1engine.
pub struct Engine {
    config: EngineConfig,
}

impl Engine {
    pub fn new() -> Self;
    pub fn with_config(config: EngineConfig) -> Self;

    /// Create a new empty document.
    pub fn create_document(&self) -> Document;

    /// Open a document from bytes (format auto-detected).
    pub fn open(&self, data: &[u8]) -> Result<Document, Error>;

    /// Open a document from file path (format detected from extension + magic bytes).
    pub fn open_file(&self, path: impl AsRef<Path>) -> Result<Document, Error>;

    /// Convert between formats without loading into Document.
    pub fn convert(&self, data: &[u8], from: Format, to: Format) -> Result<Vec<u8>, Error>;
}

#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Maximum undo history depth (default: 100).
    pub max_undo_depth: usize,
    /// Font search directories (in addition to system fonts).
    pub font_dirs: Vec<PathBuf>,
    /// Default page size in points (default: US Letter 612x792).
    pub default_page_size: (f64, f64),
}

/// A loaded document with full editing capabilities.
pub struct Document { /* internal: model, history, media */ }

impl Document {
    // --- Export ---
    pub fn export(&self, format: Format) -> Result<Vec<u8>, Error>;
    pub fn save(&self, path: impl AsRef<Path>, format: Format) -> Result<(), Error>;

    // --- Metadata ---
    pub fn metadata(&self) -> &DocumentMetadata;

    // --- Editing ---
    pub fn begin_transaction(&mut self, description: &str) -> TransactionBuilder;
    pub fn apply(&mut self, op: Operation) -> Result<(), Error>;
    pub fn undo(&mut self) -> Result<(), Error>;
    pub fn redo(&mut self) -> Result<(), Error>;
    pub fn can_undo(&self) -> bool;
    pub fn can_redo(&self) -> bool;

    // --- Queries ---
    pub fn to_plain_text(&self) -> String;
    pub fn paragraphs(&self) -> impl Iterator<Item = ParagraphRef>;
    pub fn node(&self, id: NodeId) -> Option<NodeRef>;
    pub fn find_text(&self, query: &str) -> Vec<TextMatch>;
    pub fn find_replace(&mut self, find: &str, replace: &str) -> Result<usize, Error>;

    // --- Layout ---
    pub fn layout(&self) -> Result<LayoutDocument, Error>;

    // --- Model access ---
    pub fn model(&self) -> &DocumentModel;

    // --- Builder ---
    pub fn builder(&mut self) -> DocumentBuilder;
}
```

### 10.2 Builder API

```rust
let engine = Engine::new();
let mut doc = engine.create_document();

doc.builder()
    .metadata(|m| {
        m.title("My Document")
         .author("Engineering Team")
    })
    .heading(1, "Introduction")
    .paragraph(|p| {
        p.text("This is ")
         .bold("bold")
         .text(" and this is ")
         .italic("italic")
         .text(".")
    })
    .table(3, 2, |table| {
        table.cell(0, 0, "Name");
        table.cell(1, 0, "Age");
        table.cell(2, 0, "City");
        table.cell(0, 1, "Alice");
        table.cell(1, 1, "30");
        table.cell(2, 1, "Berlin");
    })
    .page_break()
    .paragraph(|p| p.text("Page two content"))
    .build();

doc.save("output.docx", Format::Docx)?;
doc.save("output.pdf", Format::Pdf)?;
```

---

## 11. FFI Bindings

### 11.1 C API (cbindgen)

```c
// C API — auto-generated from Rust, prefixed with s1_
S1Engine* s1_engine_new(void);
void s1_engine_free(S1Engine* engine);

S1Document* s1_engine_open(S1Engine* engine, const uint8_t* data, size_t len, S1Error* err);
S1Document* s1_engine_open_file(S1Engine* engine, const char* path, S1Error* err);
S1Document* s1_engine_create_document(S1Engine* engine);

int s1_document_export(S1Document* doc, S1Format format, uint8_t** out_data, size_t* out_len, S1Error* err);
int s1_document_save(S1Document* doc, const char* path, S1Format format, S1Error* err);
const char* s1_document_to_plain_text(S1Document* doc);

void s1_document_free(S1Document* doc);
void s1_bytes_free(uint8_t* data, size_t len);
void s1_string_free(char* str);

const char* s1_error_message(S1Error* err);
void s1_error_free(S1Error* err);
```

### 11.2 WASM API (wasm-bindgen)

```typescript
// TypeScript declarations (generated from Rust WASM)
export class Engine {
  constructor();
  open(data: Uint8Array): Document;
  createDocument(): Document;
}

export class Document {
  export(format: Format): Uint8Array;
  save(path: string, format: Format): void;
  toPlainText(): string;
  applyOperation(op: Operation): void;
  undo(): void;
  redo(): void;
  free(): void;
}
```

---

## 12. Testing Strategy

### 12.1 Unit Tests
- Every crate has `#[cfg(test)] mod tests` in each source file
- Every public function has at least one test
- `s1-model` and `s1-ops` use `proptest` for property-based testing
- Format crates use round-trip tests

### 12.2 Integration Tests
- `tests/integration/` — cross-crate workflows
- Open real-world documents, verify structure, re-export
- Format conversion pipelines

### 12.3 Fuzz Testing
- `cargo-fuzz` targets for every format reader
- Fuzz operation application with random sequences
- Ensures no panics on any input

### 12.4 Fixture Tests
- Real documents in `tests/fixtures/`
- See CLAUDE.md for full fixture list

---

## 13. Performance Targets

| Operation | Target | How Measured |
|---|---|---|
| Open 10-page DOCX | < 50ms | `criterion` benchmark |
| Open 100-page DOCX | < 500ms | `criterion` benchmark |
| Export 10-page PDF | < 200ms | `criterion` benchmark |
| Export 100-page PDF | < 2s | `criterion` benchmark |
| Single paragraph edit + re-layout | < 5ms | `criterion` benchmark |
| Full document layout (10 pages) | < 100ms | `criterion` benchmark |
| WASM bundle size | < 2MB gzipped | CI check |
| Memory: 10-page doc loaded | < 10MB | Integration test |
| Memory: 100-page doc loaded | < 50MB | Integration test |
