# Test Documents

Test fixture documents for s1engine format readers, writers, and conversion pipelines. Organized by format with a mix of real-world samples and synthetic edge-case files.

## Directory Structure

```
testdocs/
├── doc/samples/           Legacy DOC (binary) format
│   ├── freetestdata_100kb.doc
│   └── freetestdata_500kb.doc
│
├── docx/samples/          OOXML DOCX format
│   ├── calibre_demo.docx
│   ├── freetestdata_100kb.docx
│   ├── freetestdata_500kb.docx
│   └── freetestdata_1mb.docx
│
├── odt/samples/           ODF ODT format
│   ├── freetestdata_100kb.odt
│   ├── freetestdata_500kb.odt
│   └── freetestdata_1mb.odt
│
├── pdf/samples/           PDF format (export-only reference)
│   ├── dummy.pdf
│   ├── freetestdata_100kb.pdf
│   ├── freetestdata_500kb.pdf
│   └── freetestdata_1mb.pdf
│
├── rtf/samples/           RTF format (reserved)
│
├── txt/samples/           Plain text format
│   ├── empty.txt              Synthetic: 0-byte file
│   ├── single_line.txt        Synthetic: single line, no trailing newline
│   ├── unicode_mixed.txt      Synthetic: multi-script Unicode
│   ├── crlf_endings.txt       Synthetic: Windows \r\n line endings
│   ├── utf16_bom.txt          Synthetic: UTF-16 LE with BOM
│   ├── large_single_para.txt  Synthetic: 10,000 words, one paragraph
│   ├── many_paragraphs.txt    Synthetic: 1,000 short paragraphs
│   └── moby_dick.txt          Real-world: full novel text
│
└── md/samples/            Markdown format
    ├── gfm_table.md           Synthetic: GFM tables with alignment and special chars
    ├── nested_lists.md        Synthetic: 4-5 level nested lists (mixed types)
    ├── code_blocks.md         Synthetic: fenced blocks (Rust/Python/JS/Bash), inline code
    ├── emphasis_edge.md       Synthetic: nested/combined emphasis, escapes
    ├── links_images.md        Synthetic: inline/reference/autolinks, images
    ├── headings_all.md        Synthetic: h1-h6, ATX and setext styles
    ├── markdown_here_readme.md  Real-world: complex Markdown document
    └── markdown_test.md         Real-world: Markdown feature showcase
```

## Synthetic TXT Files

| File | What It Tests | Key Properties |
|------|---------------|----------------|
| `empty.txt` | Empty file handling | 0 bytes, no content at all |
| `single_line.txt` | Single-line input, no trailing newline | Contains "Hello World" (11 bytes, no `\n`) |
| `unicode_mixed.txt` | Multi-script Unicode support | English, Chinese, Arabic, Japanese, Korean, Emoji, accented Latin, math symbols, currency |
| `crlf_endings.txt` | Windows line ending detection | All lines use `\r\n` (CRLF), 5 lines |
| `utf16_bom.txt` | UTF-16 LE encoding with BOM | Starts with `FF FE` BOM, includes non-ASCII characters |
| `large_single_para.txt` | Performance with long paragraphs | 10,000 words in a single paragraph (no internal line breaks) |
| `many_paragraphs.txt` | Scalability with many nodes | 1,000 one-sentence paragraphs |

## Synthetic MD Files

| File | What It Tests | Key Features |
|------|---------------|--------------|
| `gfm_table.md` | GFM table parsing | Left/center/right alignment, special chars in cells (pipes, bold, code, links, emoji), empty cells, single-column table, wide table |
| `nested_lists.md` | Deep list nesting | 4-5 level unordered, 4 level ordered, mixed ordered/unordered nesting, list items with continuation paragraphs, code blocks and blockquotes in lists, empty list items |
| `code_blocks.md` | Code block handling | Fenced blocks with language annotation (Rust, Python, JavaScript, Bash), no-language block, nested backtick fences, inline code with special chars, indented code blocks |
| `emphasis_edge.md` | Emphasis parsing edge cases | Nested bold/italic, bold-italic combinations, strikethrough, combined emphasis+strike, code inside emphasis, emphasis at line boundaries, intra-word emphasis, escaped markers |
| `links_images.md` | Link and image parsing | Inline links with titles, reference links (explicit and implicit), autolinks, email autolinks, formatted text in links, images with alt text/title, images as links, parentheses in URLs, escaped brackets |
| `headings_all.md` | Heading parsing | All 6 ATX levels, setext h1 and h2, closing hashes, headings with emphasis/code/links, 7-hash non-heading, no-space after hash, empty heading, heading immediately before paragraph |

## Real-World Samples

| File | Source | What It Tests |
|------|--------|---------------|
| `moby_dick.txt` | Project Gutenberg | Large real-world text, natural paragraph structure |
| `freetestdata_*.{docx,odt,doc,pdf}` | FreeTestData.com | Standard format compliance, various file sizes |
| `calibre_demo.docx` | Calibre project | Rich formatting from a real application |
| `markdown_here_readme.md` | Markdown Here project | Complex real-world Markdown with many features |
| `markdown_test.md` | Community test doc | Comprehensive Markdown feature exercise |

## Usage

### In Rust Tests

```rust
// Load a test fixture
let bytes = include_bytes!("../../testdocs/txt/samples/unicode_mixed.txt");
let doc = s1_format_txt::TxtReader::read(bytes)?;

// Or read from path
let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("../../testdocs/docx/samples/freetestdata_100kb.docx");
let bytes = std::fs::read(&path)?;
```

### Round-Trip Testing

```rust
// Read -> Write -> Read -> Compare
let original = std::fs::read("testdocs/txt/samples/crlf_endings.txt")?;
let doc = TxtReader::read(&original)?;
let written = TxtWriter::write(&doc)?;
let doc2 = TxtReader::read(&written)?;
assert_eq!(doc.text_content(), doc2.text_content());
```

### Edge-Case Coverage Checklist

When adding a new format reader/writer, verify against:

- [ ] Empty input (`empty.txt`)
- [ ] Single element input (`single_line.txt`)
- [ ] Unicode/multi-script content (`unicode_mixed.txt`)
- [ ] Alternate line endings (`crlf_endings.txt`)
- [ ] Alternate encodings (`utf16_bom.txt`)
- [ ] Large single node (`large_single_para.txt`)
- [ ] Many nodes (`many_paragraphs.txt`)
- [ ] Real-world content (`moby_dick.txt`, `freetestdata_*`)
