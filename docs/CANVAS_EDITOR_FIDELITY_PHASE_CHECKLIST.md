# Canvas-First Editor Fidelity Phase Checklist

**Status:** Draft phase checklist  
**Last updated:** 2026-03-30  
**Applies to branch:** `feature/reimagine-port`

## Related Documents

- `CANVAS_EDITOR_FIDELITY_VALIDATION_SPEC.md`
- `CANVAS_EDITOR_IMPLEMENTATION_ROADMAP.md`
- `CANVAS_EDITOR_ELEMENTS_SPEC.md`
- `CANVAS_EDITOR_WASM_API_CONTRACT.md`

## Purpose

This checklist converts the canvas editor fidelity validation spec into release gates that can be used during implementation.

The rule is simple:

> No canvas phase is complete until its required fidelity evidence exists.

## Global Exit Rules

These apply to every phase.

- engine geometry remains the source of truth
- corpus cases used for the phase are listed in the corpus manifest
- failing cases are documented explicitly, not hand-waved
- validation output is attached for the branch or PR
- the canvas path does not quietly fall back to DOM ownership for the feature under test

## Phase 0: Contract Freeze and Instrumentation

Required evidence:

- scene, geometry, and edit APIs documented
- corpus manifest created and versioned
- comparison tooling exists and runs from CLI
- document revision and layout revision are observable in debug output

Checklist:

- [ ] WASM contract is approved
- [ ] fidelity corpus manifest exists
- [ ] geometry diff tool exists
- [ ] validation report format is defined
- [ ] debug instrumentation requirements are accepted

## Phase 1: Read-Only Page Fidelity

Required corpus tiers:

- Tier 1 core pagination corpus
- Tier 2 structured-layout corpus for page counts and content rects

Required evidence:

- page count matches engine
- page size and content rects match engine
- zoom does not introduce page-break drift
- visible pages render without `.page-content`

Checklist:

- [ ] page-map comparison passes on required corpus
- [ ] content rect delta stays within spec tolerance
- [ ] page chrome is stable at 100%, 125%, and 200% zoom
- [ ] canvas path renders visible pages without DOM page content ownership

## Phase 2: Geometry and Hit-Testing Fidelity

Required corpus tiers:

- Tier 1 core pagination corpus
- Tier 2 structured-layout corpus

Required evidence:

- click-to-position is correct
- caret rects match engine geometry
- selection rects match engine geometry
- no DOM range is required for position resolution in canvas mode

Checklist:

- [ ] hit-test scenarios pass on required corpus
- [ ] caret rect delta stays within spec tolerance
- [ ] selection rect delta stays within spec tolerance
- [ ] keyboard navigation is model-based in canvas mode

## Phase 3: Typing and IME Fidelity

Required corpus tiers:

- Tier 1 core pagination corpus
- Tier 4 stress corpus for IME and multilingual cases

Required evidence:

- single-caret typing updates only dirty pages returned by engine
- IME candidate anchoring follows caret geometry
- cursor painting matches canonical selection state

Checklist:

- [ ] typing scenarios pass on required corpus
- [ ] IME scenarios pass on approved browser matrix
- [ ] dirty-page repaint behavior is validated
- [ ] canvas caret remains aligned after edits and zoom changes

## Phase 4: Selection, Clipboard, and Search Fidelity

Required corpus tiers:

- Tier 1 core pagination corpus
- Tier 2 structured-layout corpus
- Tier 4 stress corpus for large selections

Required evidence:

- drag selection works across lines and pages
- copy/cut/paste uses model ranges
- find highlights match engine ranges

Checklist:

- [ ] multi-line selection paint matches engine rects
- [ ] cross-page selection scenarios pass
- [ ] clipboard round-trip checks pass for approved cases
- [ ] search highlight scenarios pass

## Phase 5: Common Object and Print-Editor Fidelity

Required corpus tiers:

- Tier 2 structured-layout corpus
- Tier 3 visual-object corpus

Required evidence:

- lists, headers/footers, links, images, footnotes, rulers, and guides render from scene geometry
- object selection does not depend on DOM boxes
- print-editor zoom feel is stable

Checklist:

- [ ] list marker placement matches engine layout
- [ ] header/footer placement matches engine layout
- [ ] image bounds stay within tolerance
- [ ] footnote placement matches engine layout
- [ ] rulers and guides align with page metrics

## Phase 6: Table and Review Fidelity

Required corpus tiers:

- Tier 2 structured-layout corpus
- Tier 3 visual-object corpus
- Tier 4 stress corpus where applicable

Required evidence:

- table borders render deterministically
- table navigation and selection are geometry-driven
- comment anchors and review markers use scene geometry
- spellcheck underlines do not rely on visible DOM text

Checklist:

- [ ] table border segment deltas stay within tolerance
- [ ] table cell hit-testing passes
- [ ] comment anchor geometry matches engine output
- [ ] track changes markers paint in correct page positions
- [ ] spellcheck overlays attach to geometry, not DOM text boxes

## Phase 7: Full Parity and DOM Retirement

Required corpus tiers:

- all approved tiers

Required evidence:

- canvas matches or exceeds DOM baseline on approved corpus
- no critical format-fidelity regression
- accessibility and clipboard remain acceptable

Checklist:

- [ ] full corpus run is attached
- [ ] canvas meets or beats DOM baseline on layout fidelity
- [ ] canvas meets or beats DOM baseline on render fidelity
- [ ] canvas meets or beats DOM baseline on interaction fidelity
- [ ] format fidelity has no critical regression
- [ ] accessibility and clipboard sign-off are complete

## Reporting Template

Every phase sign-off should capture:

- corpus manifest version
- branch and commit
- browser/platform/font set
- passing cases
- failing cases
- known exceptions
- screenshots or geometry diffs for top regressions

## Operational Rule

If a phase fails fidelity gates, implementation continues only for diagnosis or rollback, not for pretending the phase is complete.
