# Document Converter CLI Example

Convert documents between formats using s1engine.

## Usage

```bash
# From the project root
cargo run --example document-converter -- input.docx output.pdf
cargo run --example document-converter -- report.odt report.docx
cargo run --example document-converter -- notes.md notes.pdf
```

## Supported Conversions

| From | To |
|------|----|
| DOCX | PDF, ODT, TXT, MD |
| ODT | PDF, DOCX, TXT, MD |
| TXT | DOCX, ODT, PDF |
| MD | DOCX, ODT, PDF |
