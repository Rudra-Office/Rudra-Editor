# Dependencies

All external dependencies used by s1engine, organized by crate and phase.

## Dependency Policy

1. **Minimize dependencies** — every dependency is a maintenance burden and attack surface
2. **Prefer well-maintained crates** — active maintainers, recent releases, good issue response
3. **Prefer pure Rust** — C/C++ FFI only in `s1-text` crate for text processing
4. **Pin major versions** — use `>=x.y, <x+1` ranges in Cargo.toml
5. **Audit security** — run `cargo audit` in CI

---

## Core Crates (Phase 1)

### s1-model
**Zero external dependencies.** Pure Rust data structures.

This is intentional and must never change. The document model is the foundation — it must be portable, compilable anywhere, and have zero supply-chain risk.

### s1-ops
| Dependency | Version | Purpose | License |
|---|---|---|---|
| `s1-model` | workspace | Core types | — |

No external dependencies. Pure logic operating on `s1-model` types.

### s1-format-txt
| Dependency | Version | Purpose | License |
|---|---|---|---|
| `s1-model` | workspace | Core types | — |
| `encoding_rs` | ^0.8 | Detect/convert text encodings (UTF-8, UTF-16, Latin-1, etc.) | MIT/Apache-2.0 |

### s1-format-docx
| Dependency | Version | Purpose | License |
|---|---|---|---|
| `s1-model` | workspace | Core types | — |
| `quick-xml` | ^0.36 | Fast, streaming XML parser and writer | MIT |
| `zip` | ^2.0 | ZIP archive reading and writing (DOCX/ODT are ZIP files) | MIT |
| `base64` | ^0.22 | Base64 encoding for embedded content | MIT/Apache-2.0 |

**Why quick-xml?**
- Fastest Rust XML parser (streaming, zero-copy where possible)
- No C dependencies (unlike libxml2 bindings)
- Well-maintained, widely used in Rust ecosystem
- Supports both reading and writing

**Why zip (crate)?**
- Pure Rust ZIP implementation
- Supports deflate compression (required by OOXML/ODF)
- Read and write support

### s1-format-odt
| Dependency | Version | Purpose | License |
|---|---|---|---|
| `s1-model` | workspace | Core types | — |
| `quick-xml` | ^0.36 | XML parser/writer (shared with DOCX) | MIT |
| `zip` | ^2.0 | ZIP archive handling (shared with DOCX) | MIT |

### s1engine (facade)
| Dependency | Version | Purpose | License |
|---|---|---|---|
| `s1-model` | workspace | Core types | — |
| `s1-ops` | workspace | Operations | — |
| `s1-format-*` | workspace | Format support (feature-gated) | — |
| `s1-layout` | workspace | Layout engine (feature-gated) | — |
| `s1-convert` | workspace | Conversion (feature-gated) | — |
| `thiserror` | ^2.0 | Ergonomic error type derivation | MIT/Apache-2.0 |

### Shared Dev Dependencies (all crates)
| Dependency | Version | Purpose | License |
|---|---|---|---|
| `proptest` | ^1.0 | Property-based testing | MIT/Apache-2.0 |
| `pretty_assertions` | ^1.0 | Better test failure diffs | MIT/Apache-2.0 |

---

## Rich Document Crates (Phase 2)

No new external dependencies in Phase 2. All format features (tables, images, lists) use the same `quick-xml` and `zip` crates.

### Benchmarking
| Dependency | Version | Purpose | License |
|---|---|---|---|
| `criterion` | ^0.5 | Statistical benchmarking framework | MIT/Apache-2.0 |

---

## Layout & Export Crates (Phase 3)

### s1-text (Text Processing — C/C++ FFI Layer)
| Dependency | Version | Purpose | License |
|---|---|---|---|
| `s1-model` | workspace | Core types | — |
| `harfbuzz-rs` | ^2.0 | Rust bindings to HarfBuzz text shaping engine | MIT |
| `freetype-rs` | ^0.36 | Rust bindings to FreeType font library | MIT |
| `fontdb` | ^0.22 | System font discovery and indexing | MIT |
| `unicode-bidi` | ^0.3 | Unicode Bidirectional Algorithm (UAX #9) | MIT/Apache-2.0 |
| `unicode-linebreak` | ^0.1 | Unicode Line Breaking Algorithm (UAX #14) | MIT |

**Why HarfBuzz (via harfbuzz-rs)?**
- Industry-standard text shaping engine (used by Chrome, Firefox, Android, LibreOffice)
- Handles complex scripts: Arabic, Devanagari, Thai, CJK
- OpenType feature support (ligatures, kerning, contextual alternates)
- 15+ years of development, battle-tested
- No viable pure-Rust alternative exists for full Unicode shaping

**Why FreeType (via freetype-rs)?**
- Industry-standard font engine (reads TrueType, OpenType, Type1, etc.)
- Glyph rasterization, font metrics, glyph outlines
- Used by virtually every non-Windows text rendering stack
- No viable pure-Rust alternative for full font format support

**Why fontdb?**
- Pure Rust system font discovery
- Indexes fonts by family, style, weight
- Cross-platform (macOS, Linux, Windows)
- Lightweight, no C dependencies

**Why unicode-bidi + unicode-linebreak instead of ICU?**
- Pure Rust — no C/C++ dependency
- Focused: only the Unicode algorithms we need
- Much smaller than full ICU
- If we later need full ICU: consider `icu4x` (pure Rust, by Unicode Consortium)

### s1-layout
| Dependency | Version | Purpose | License |
|---|---|---|---|
| `s1-model` | workspace | Core types | — |
| `s1-text` | workspace | Text shaping and font metrics | — |

No additional external dependencies. Layout algorithms are implemented in pure Rust.

### s1-format-pdf
| Dependency | Version | Purpose | License |
|---|---|---|---|
| `s1-model` | workspace | Core types | — |
| `s1-layout` | workspace | Layout tree (PDF renders from layout) | — |
| `pdf-writer` | ^0.12 | Low-level PDF file generation | MIT/Apache-2.0 |
| `subsetter` | ^0.2 | Font subsetting (embed only used glyphs) | MIT/Apache-2.0 |
| `image` | ^0.25 | Image decoding (PNG, JPEG, etc.) | MIT/Apache-2.0 |

**Why pdf-writer?**
- Pure Rust, no C dependencies
- Low-level control over PDF output
- Small, focused crate (not a monolithic PDF toolkit)
- Same authors as Typst — proven in production

**Why subsetter?**
- Companion to `pdf-writer` — same authors
- Subsets TrueType/OpenType fonts to include only used glyphs
- Critical for reasonable PDF file sizes

### s1-convert
| Dependency | Version | Purpose | License |
|---|---|---|---|
| `s1-model` | workspace | Core types | — |
| `s1-format-docx` | workspace | DOCX codec | — |
| `s1-format-odt` | workspace | ODT codec | — |
| `cfb` | ^0.10 | OLE2 Compound File Binary reader (for DOC files) | MIT |

**Why cfb?**
- Reads Microsoft's OLE2 container format (used by .doc, .xls, .ppt)
- Pure Rust
- Required as first step to reading legacy DOC files

---

## FFI & WASM Crates (Phase 5)

### ffi/c
| Dependency | Version | Purpose | License |
|---|---|---|---|
| `s1engine` | workspace | Engine facade | — |
| `cbindgen` | ^0.27 | Auto-generate C headers from Rust code | MPL-2.0 |

### ffi/wasm
| Dependency | Version | Purpose | License |
|---|---|---|---|
| `s1engine` | workspace | Engine facade | — |
| `wasm-bindgen` | ^0.2 | Rust ↔ JavaScript FFI for WASM | MIT/Apache-2.0 |
| `js-sys` | ^0.3 | JavaScript standard library bindings | MIT/Apache-2.0 |
| `web-sys` | ^0.3 | Web API bindings (optional, for browser features) | MIT/Apache-2.0 |
| `serde-wasm-bindgen` | ^0.6 | Serialize Rust types for JS consumption | MIT |

---

## C/C++ System Libraries

These are linked at build time via `-sys` crates or system package managers.

| Library | Version | Linked By | Install (macOS) | Install (Ubuntu) |
|---|---|---|---|---|
| HarfBuzz | >= 8.0 | `harfbuzz-rs` | `brew install harfbuzz` | `apt install libharfbuzz-dev` |
| FreeType | >= 2.13 | `freetype-rs` | `brew install freetype` | `apt install libfreetype-dev` |

**Note**: Both `harfbuzz-rs` and `freetype-rs` can optionally vendor (compile from source) their C libraries. This is recommended for reproducible builds and CI.

```toml
# Cargo.toml — vendor C libraries instead of using system ones
harfbuzz-rs = { version = "^2.0", features = ["bundled"] }
freetype-rs = { version = "^0.36", features = ["bundled"] }
```

---

## Dependency Graph (Visualization)

```
s1engine
├── s1-model              (0 external deps)
├── s1-ops                (0 external deps)
├── s1-format-docx        (quick-xml, zip, base64)
├── s1-format-odt         (quick-xml, zip)
├── s1-format-txt         (encoding_rs)
├── s1-format-pdf         (pdf-writer, subsetter, image)
├── s1-layout             (0 external deps)
├── s1-convert            (cfb)
├── s1-text               (harfbuzz-rs, freetype-rs, fontdb, unicode-bidi, unicode-linebreak)
├── thiserror
└── [dev] proptest, criterion, pretty_assertions
```

**Total external runtime dependencies**: ~15 crates (not counting transitive deps)
**C/C++ libraries**: 2 (HarfBuzz, FreeType) — isolated in `s1-text` only

---

## License Compatibility

All dependencies use MIT, Apache-2.0, or MIT/Apache-2.0 dual license, which are compatible with s1engine's MIT/Apache-2.0 dual license.

The only exception is `cbindgen` (MPL-2.0), which is a build tool only — it is not linked into the final binary.

---

## Alternatives Considered

| Chosen | Alternative | Why We Chose This |
|---|---|---|
| `quick-xml` | `roxmltree`, `xmltree` | `quick-xml` supports both read and write; streaming API for large files |
| `zip` | `rc-zip` | `zip` is more mature and widely used |
| `pdf-writer` | `printpdf`, `lopdf` | `pdf-writer` is lower-level, smaller, proven by Typst |
| `harfbuzz-rs` | `rustybuzz` | `rustybuzz` is pure Rust but may lag behind HarfBuzz in complex script support |
| `freetype-rs` | `ttf-parser` | `ttf-parser` is pure Rust read-only; FreeType offers rasterization too |
| `unicode-bidi` | ICU / `icu4x` | Smaller, focused; full ICU is overkill for just BiDi |
| `fontdb` | `font-kit` | `fontdb` is lighter, no C deps, does what we need |

**Watch list**: `rustybuzz` (pure Rust HarfBuzz port) and `icu4x` (pure Rust Unicode). If these mature sufficiently, we can drop all C/C++ dependencies and become 100% pure Rust.
