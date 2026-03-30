# Fidelity Validation Assets

This directory holds the validation scaffolding for the canvas-first editor migration.

## Files

- `corpus_manifest.json`: canonical list of validation cases, tiers, expected artifacts, and gate coverage
- `corpus/`: source documents or references for approved fidelity cases
- `artifacts/`: generated reference and candidate outputs such as layout JSON, page-map JSON, screenshots, and reports

## Seeded Cases

The first Tier 1 corpus cases can be generated from the repo with:

```bash
./scripts/generate-fidelity-artifacts.sh
```

That currently produces:

- `tier1_basic_paragraphs`
- `tier1_headers_footers`

DOM baseline artifacts for generated cases can then be captured with:

```bash
./scripts/capture-dom-fidelity-baselines.sh
```

That writes scene-style DOM geometry JSON to the `dom_baseline_layout_json` paths in the manifest.

## Intended Workflow

1. add or update a corpus case in `corpus_manifest.json`
2. generate engine reference artifacts for that case
3. capture DOM baseline artifacts for that case
4. generate canvas candidate artifacts
5. compare them with `scripts/compare-canvas-fidelity.py`
6. attach the report to the relevant phase gate or change review
