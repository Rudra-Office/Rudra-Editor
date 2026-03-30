# Canvas-First Editor Fidelity Validation Spec

**Status:** Draft validation spec  
**Last updated:** 2026-03-30  
**Applies to branch:** `feature/reimagine-port`

## Related Documents

- `CANVAS_EDITOR_HIGH_LEVEL_ARCHITECTURE.md`
- `CANVAS_EDITOR_LOW_LEVEL_DESIGN.md`
- `CANVAS_EDITOR_ELEMENTS_SPEC.md`
- `CANVAS_EDITOR_IMPLEMENTATION_ROADMAP.md`
- `CANVAS_EDITOR_WASM_API_CONTRACT.md`
- `fidelity/OVERVIEW.md`

## Purpose

This document defines how to prove whether the canvas-first editor increases fidelity.

The key distinction is:

- **format fidelity**: how much DOCX/ODT structure and semantics the engine reads, preserves, edits, and writes
- **layout fidelity**: whether pagination, line breaks, object placement, and page geometry match engine truth and reference output
- **render fidelity**: whether what the user sees in the editor matches the intended page output
- **interaction fidelity**: whether caret movement, selection geometry, hit-testing, and editing behavior match the laid-out document

Canvas-first should improve layout, render, and interaction fidelity if implemented correctly.
Canvas-first does **not** automatically improve format fidelity.

## Validation Question

The migration is successful only if the answer to both questions is yes:

1. Does the canvas-first editor match the Rust layout engine more closely than the DOM editor?
2. Does the canvas-first editor avoid regressing current document-format fidelity?

## What We Expect To Improve

Expected improvements from canvas-first architecture:

- page-break fidelity
- margin and guide fidelity
- table border fidelity
- image and shape placement fidelity
- zoom fidelity
- header/footer and footnote placement fidelity
- selection painting fidelity
- caret painting fidelity
- click and drag hit-test fidelity

Expected non-automatic improvements:

- DOCX import fidelity
- ODT import fidelity
- DOCX export fidelity
- ODT export fidelity
- unsupported semantic feature preservation

Those remain work in the format crates and model/layout engine, not in the canvas shell.

## Validation Baselines

Every fidelity check should compare at least two of these surfaces:

### Baseline A: Legacy DOM editor

Current browser path based on:

- `ffi/wasm/src/lib.rs` HTML-oriented rendering APIs
- `editor/src/render.js`
- `editor/src/input.js`
- `.page-content` and `contenteditable` ownership

### Baseline B: Canvas-first editor

New browser path based on:

- scene APIs from WASM
- canvas page rendering
- hidden input bridge
- geometry-driven selection and caret

### Baseline C: Engine reference output

Reference outputs owned by Rust:

- layout scene / layout JSON
- PDF export output
- page map, run positions, object bounds, and selection/caret geometry APIs

### Baseline D: Source-application reference

When available, compare against source app output for authored corpus documents:

- Microsoft Word PDF print/export
- LibreOffice PDF export
- OpenOffice PDF export when relevant

The primary truth for editor validation is Baseline C. Baseline D is a stronger product-fidelity check, but it depends on the source application.

## Validation Principle

The browser must be judged against Rust-owned layout truth, not against its own DOM or canvas implementation details.

That means all page/render comparisons should resolve to engine geometry where possible.

## Fidelity Dimensions

### 1. Format Fidelity

Questions:

- was the source document parsed correctly?
- were semantics preserved through edit/save?
- are unsupported features preserved as raw XML or lost?

Primary metrics:

- feature support matrix status
- round-trip preservation rate
- import/export diff counts
- unsupported-feature loss count

Owner:

- `s1-format-docx`
- `s1-format-odt`
- `s1-model`

Canvas migration impact:

- indirect only

### 2. Layout Fidelity

Questions:

- do page breaks match the engine?
- do line breaks match the engine?
- do headers/footers/footnotes land in the correct regions?
- do tables, images, and floating objects use correct geometry?

Primary metrics:

- page count match
- page break position match
- line count per paragraph
- run bounding-box deltas
- table border segment deltas
- image/shape anchor rect deltas
- header/footer/content rect match

Owner:

- `s1-layout`
- `s1-text`
- WASM scene/geometry APIs

Canvas migration impact:

- direct and high

### 3. Render Fidelity

Questions:

- does the editor look like the engine output at all zoom levels?
- do borders, backgrounds, markers, and object layers paint consistently?
- are glyph positions stable under zoom and device pixel ratio changes?

Primary metrics:

- visual diff against reference rasterization
- per-page pixel diff thresholds
- zoom consistency checks at multiple scales
- layer-order correctness checks

Owner:

- canvas renderer
- scene serialization layer
- font loading path

Canvas migration impact:

- direct and high

### 4. Interaction Fidelity

Questions:

- does click map to the right model position?
- does caret geometry match the intended insertion point?
- do drag selection rectangles follow laid-out text correctly?
- does keyboard navigation respect page and line boundaries?

Primary metrics:

- hit-test accuracy rate
- caret rect delta from engine rect
- selection rect overlap with engine rects
- keyboard navigation conformance cases
- IME caret-anchor correctness

Owner:

- geometry APIs
- hidden input bridge
- selection and navigation controllers

Canvas migration impact:

- direct and high

## Test Corpus

Use a fixed corpus grouped by risk level.

### Tier 1: Core pagination corpus

Documents covering:

- plain paragraphs
- headings
- mixed fonts and sizes
- lists
- page breaks
- headers and footers
- footnotes and endnotes

### Tier 2: Structured-layout corpus

Documents covering:

- tables with merged cells
- complex borders and shading
- nested lists
- multi-section documents
- columns
- bookmarks and hyperlinks

### Tier 3: Visual-object corpus

Documents covering:

- inline images
- floating images
- wrapped images
- shapes and text boxes when supported
- tracked changes and comment anchors

### Tier 4: Stress corpus

Documents covering:

- large documents with hundreds of pages
- multilingual/CJK/RTL content
- font fallback cases
- malformed but accepted DOCX/ODT files
- high-zoom and low-zoom view scenarios

Each corpus item should have:

- source document
- canonical engine output snapshot
- source-application reference PDF when available
- expected known limitations note

## Measurement Methods

### A. Page-map comparison

Compare:

- page count
- page dimensions
- content rects
- header/footer rects
- footnote area boundaries

Acceptance direction:

- canvas must match engine exactly
- DOM baseline may be recorded for comparison but is not the pass target

### B. Geometry comparison

Compare engine and browser for:

- text run bounds
- line baselines
- table border segments
- image and shape bounds
- caret rects
- selection rects

Metrics:

- max delta in points
- mean delta in points
- percentage within tolerance band

### C. Visual raster comparison

Rasterize reference outputs and compare against browser rendering captures.

Recommended reference order:

1. engine PDF export rendered to bitmap
2. source-application PDF export rendered to bitmap
3. browser canvas screenshot
4. browser DOM screenshot for baseline only

Metrics:

- per-page pixel diff percentage
- masked diff percentage excluding UI chrome
- edge-alignment diff around borders and glyph runs

### D. Interaction scenario tests

Run scripted scenarios for:

- click to caret
- shift-arrow selection
- drag selection across lines and pages
- home/end and word navigation
- header/footer edit entry
- table cell selection
- IME composition anchor behavior

Metrics:

- correct final position/range
- rect agreement with engine geometry
- no DOM-range fallback usage in canvas mode

## Tolerances

These are starting thresholds and should be tightened as the implementation stabilizes.

### Hard-match requirements

These should be exact or treated as failures:

- page count
- page order
- section order
- header/footer presence by page
- dirty-page reporting after edits

### Geometry tolerances

Initial acceptable tolerances:

- text/caret/selection rect delta: `<= 1.0pt`
- image and shape bounds delta: `<= 1.5pt`
- table border segment delta: `<= 1.0pt`
- page content rect delta: `<= 0.25pt`

### Visual tolerances

Initial acceptable thresholds:

- non-text decorative diff per page: `<= 0.5%`
- text-region raster diff should trend toward engine glyph geometry, not browser DOM text metrics
- no visible border drift or page-break drift between 100%, 125%, and 200% zoom

These thresholds are product gates, not excuses. If a document class repeatedly needs relaxed thresholds, the architecture or renderer is wrong.

## Phase Gates

### Gate 1: Read-only page fidelity

Required before Phase 2 is considered stable:

- page count matches engine
- page rects and content rects match engine
- page background and page chrome are stable under zoom
- no DOM `.page-content` needed for visible page display

### Gate 2: Geometry fidelity

Required before single-caret editing is accepted:

- click hit-test accuracy is stable on Tier 1 and Tier 2 corpus
- caret rect API agrees with rendered caret placement
- selection rect API matches painted highlights

### Gate 3: Typing and IME fidelity

Required before canvas mode handles primary editing:

- typing changes only dirty pages reported by engine
- IME candidate anchoring follows caret geometry correctly
- keyboard navigation is model-based and page-aware

### Gate 4: Structured object fidelity

Required before claiming print-editor parity:

- tables, images, headers/footers, and footnotes render from scene geometry
- table borders do not drift with zoom
- image/object selection uses geometry, not DOM boxes

### Gate 5: Full parity gate

Required before DOM editing ownership can be retired:

- no critical regression in format fidelity metrics
- layout/render/interaction fidelity is better than or equal to DOM baseline across the approved corpus
- accessibility and clipboard remain acceptable

## Required Instrumentation

The canvas migration should expose instrumentation for validation:

- document revision
- layout revision
- visible page range
- page scene debug ids
- hit-test trace data
- caret and selection geometry dumps
- optional page-scene JSON snapshot export for failing cases

Without this instrumentation, regressions will be hard to localize.

## Reporting Format

Each validation run should report:

- corpus version
- code revision / branch
- engine revision if tracked separately
- browser and platform
- font set used
- per-document pass/fail
- aggregate metrics by fidelity dimension
- top regressions with screenshots or geometry diffs

## Pass Criteria

The migration can claim higher fidelity only if all are true:

1. canvas beats or matches DOM baseline on layout fidelity across the approved corpus
2. canvas beats or matches DOM baseline on render fidelity across the approved corpus
3. canvas beats or matches DOM baseline on interaction fidelity across the approved corpus
4. format fidelity does not regress because of browser/editor integration changes
5. failing cases are limited to explicitly documented unsupported features

## Non-Goals

This spec does not claim that canvas-first will:

- solve unsupported DOCX/ODT features by itself
- fix missing model attributes by itself
- replace format-fidelity audits in `docs/fidelity/`
- remove the need for source-application comparison on high-risk documents

## Bottom Line

If implemented with full scene geometry, correct fonts, and Rust-owned hit-testing, the canvas-first editor should produce higher page/render/interaction fidelity than the current DOM-first editor.

This spec exists to prove that with measurements, not assumptions.
