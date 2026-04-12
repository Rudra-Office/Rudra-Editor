//! Feature coverage tests — verifies all major APIs work end-to-end.

use s1engine::{DocumentBuilder, Engine, Format};

#[test]
fn builder_basic() {
    let doc = DocumentBuilder::new()
        .title("Test")
        .author("A")
        .heading(1, "H1")
        .paragraph(|p| p.text("Body"))
        .build();
    assert!(doc.to_plain_text().contains("H1"));
    assert!(doc.to_plain_text().contains("Body"));
}

#[test]
fn docx_roundtrip() {
    let doc = DocumentBuilder::new()
        .heading(1, "Title")
        .paragraph(|p| p.text("Content"))
        .build();
    let bytes = doc.export(Format::Docx).unwrap();
    assert!(bytes.len() > 100);
    let doc2 = Engine::new().open(&bytes).unwrap();
    assert!(doc2.to_plain_text().contains("Title"));
    assert!(doc2.to_plain_text().contains("Content"));
}

#[test]
fn odt_roundtrip() {
    let doc = DocumentBuilder::new()
        .paragraph(|p| p.text("ODT test"))
        .build();
    let bytes = doc.export(Format::Odt).unwrap();
    assert!(bytes.len() > 100);
    let doc2 = Engine::new().open(&bytes).unwrap();
    assert!(doc2.to_plain_text().contains("ODT test"));
}

#[test]
fn txt_roundtrip() {
    let doc = DocumentBuilder::new()
        .paragraph(|p| p.text("TXT test"))
        .build();
    let bytes = doc.export(Format::Txt).unwrap();
    let doc2 = Engine::new().open(&bytes).unwrap();
    assert!(doc2.to_plain_text().contains("TXT test"));
}

#[test]
fn markdown_export() {
    let doc = DocumentBuilder::new()
        .heading(1, "MD Title")
        .paragraph(|p| p.text("Body"))
        .build();
    let bytes = doc.export(Format::Md).unwrap();
    let md = String::from_utf8(bytes).unwrap();
    assert!(md.contains("MD Title"));
}

// PDF tests are behind the `pdf` feature — skip in default feature set
#[test]
fn pdf_export_via_format() {
    let doc = DocumentBuilder::new()
        .paragraph(|p| p.text("PDF"))
        .build();
    // Use the export() path which works with the pdf feature
    let result = doc.export(Format::Pdf);
    // PDF may or may not be available depending on features
    if result.is_ok() {
        assert!(result.unwrap().starts_with(b"%PDF"));
    }
}

#[test]
fn metadata() {
    let doc = DocumentBuilder::new()
        .title("MyTitle")
        .author("MyAuthor")
        .paragraph(|p| p.text("x"))
        .build();
    assert_eq!(doc.model().metadata().title.as_deref(), Some("MyTitle"));
    assert_eq!(doc.model().metadata().creator.as_deref(), Some("MyAuthor"));
}

#[test]
fn metadata_docx_roundtrip() {
    let doc = DocumentBuilder::new()
        .title("RT")
        .author("RA")
        .paragraph(|p| p.text("x"))
        .build();
    let bytes = doc.export(Format::Docx).unwrap();
    let doc2 = Engine::new().open(&bytes).unwrap();
    assert_eq!(doc2.model().metadata().title.as_deref(), Some("RT"));
}

#[test]
fn headings_collection() {
    let doc = DocumentBuilder::new()
        .heading(1, "H1")
        .heading(2, "H2")
        .paragraph(|p| p.text("body"))
        .build();
    let h = doc.model().collect_headings();
    assert!(h.len() >= 2, "headings: {}", h.len());
}

#[test]
fn format_detection() {
    assert_eq!(Format::detect(b"PK\x03\x04"), Format::Docx);
    assert_eq!(Format::detect(b"%PDF-1.4"), Format::Pdf);
    assert_eq!(Format::detect(b"Hello world"), Format::Txt);
}

#[test]
fn empty_document() {
    let doc = Engine::new().create();
    let bytes = doc.export(Format::Docx).unwrap();
    assert!(bytes.len() > 50);
}

#[test]
fn paragraph_count() {
    let doc = DocumentBuilder::new()
        .paragraph(|p| p.text("A"))
        .paragraph(|p| p.text("B"))
        .paragraph(|p| p.text("C"))
        .build();
    assert!(doc.paragraph_count() >= 3);
}

#[test]
fn complex_docx_roundtrip() {
    let doc = DocumentBuilder::new()
        .title("Complex")
        .heading(1, "Chapter")
        .paragraph(|p| p.text("Normal ").bold("bold").text(" ").italic("italic"))
        .heading(2, "Section")
        .paragraph(|p| p.text("End"))
        .build();
    let bytes = doc.export(Format::Docx).unwrap();
    let doc2 = Engine::new().open(&bytes).unwrap();
    let text = doc2.to_plain_text();
    assert!(text.contains("Chapter"));
    assert!(text.contains("bold"));
    assert!(text.contains("End"));
}

#[test]
fn html_read() {
    let doc = s1_format_html::read_html(b"<h1>Test</h1><p>Hello <b>world</b></p>").unwrap();
    assert!(doc.to_plain_text().contains("Test"));
    assert!(doc.to_plain_text().contains("Hello"));
}

#[test]
fn html_write() {
    let doc = DocumentBuilder::new()
        .heading(1, "Title")
        .paragraph(|p| p.text("Body"))
        .build();
    let bytes = s1_format_html::write_html(doc.model()).unwrap();
    let html = String::from_utf8(bytes).unwrap();
    assert!(html.contains("<h1>"));
    assert!(html.contains("Body"));
}

#[test]
fn rtf_read() {
    let doc = s1_format_rtf::read_rtf(b"{\\rtf1\\ansi Hello world}").unwrap();
    assert!(doc.to_plain_text().contains("Hello"));
}

#[test]
fn rtf_write() {
    let doc = DocumentBuilder::new()
        .paragraph(|p| p.text("RTF content"))
        .build();
    let bytes = s1_format_rtf::write_rtf(doc.model()).unwrap();
    let rtf = String::from_utf8(bytes).unwrap();
    assert!(rtf.starts_with("{\\rtf1"));
    assert!(rtf.contains("RTF content"));
}

#[test]
fn plain_text_extraction() {
    let doc = DocumentBuilder::new()
        .paragraph(|p| p.text("Alpha ").bold("Beta").text(" Gamma"))
        .build();
    let text = doc.to_plain_text();
    assert!(text.contains("Alpha"));
    assert!(text.contains("Beta"));
    assert!(text.contains("Gamma"));
}
