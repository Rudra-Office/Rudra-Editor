#!/usr/bin/env bash
set -euo pipefail

# ============================================================================
# s1engine Feature Test & Fidelity Analysis
# ============================================================================
#
# Runs all Rust workspace tests, builds WASM, executes round-trip fidelity
# tests, and generates a comprehensive report at docs/FEATURE_TEST_REPORT.md.
#
# Usage:
#   ./scripts/feature-test.sh              # Full suite
#   ./scripts/feature-test.sh --skip-wasm  # Skip WASM build (faster)
#   ./scripts/feature-test.sh --quick      # Only feature_coverage tests + report
#
# Exit codes:
#   0  All tests passed
#   1  One or more steps failed (report still generated)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
REPORT="$PROJECT_DIR/docs/FEATURE_TEST_REPORT.md"
SKIP_WASM=false
QUICK=false

for arg in "$@"; do
    case "$arg" in
        --skip-wasm) SKIP_WASM=true ;;
        --quick)     QUICK=true; SKIP_WASM=true ;;
        --help|-h)
            echo "Usage: $0 [--skip-wasm] [--quick] [--help]"
            echo "  --skip-wasm  Skip WASM build step"
            echo "  --quick      Only run feature_coverage tests and generate report"
            echo "  --help       Show this help"
            exit 0
            ;;
    esac
done

cd "$PROJECT_DIR"

# ---- Timestamps and git info -----------------------------------------------

RUN_DATE="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
COMMIT_HASH="$(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')"
COMMIT_MSG="$(git log -1 --pretty=%s 2>/dev/null || echo 'unknown')"
BRANCH="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo 'unknown')"

# Accumulate report sections and overall status
OVERALL_STATUS=0
declare -a SECTION_RESULTS=()

section_pass() { SECTION_RESULTS+=("PASS|$1"); }
section_fail() { SECTION_RESULTS+=("FAIL|$1"); OVERALL_STATUS=1; }
section_skip() { SECTION_RESULTS+=("SKIP|$1"); }

# ---- Helpers ----------------------------------------------------------------

BOLD='\033[1m'
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
RESET='\033[0m'

banner() { printf "\n${BOLD}${CYAN}=== %s ===${RESET}\n\n" "$1"; }
ok()     { printf "${GREEN}[PASS]${RESET} %s\n" "$1"; }
fail()   { printf "${RED}[FAIL]${RESET} %s\n" "$1"; }
skip()   { printf "${YELLOW}[SKIP]${RESET} %s\n" "$1"; }

# ============================================================================
# STEP 1: Workspace Rust Tests
# ============================================================================

CRATE_RESULTS=""

run_workspace_tests() {
    banner "Step 1: Rust Workspace Tests"

    # Collect per-crate results by running tests per workspace member
    # (excluding wasm/c FFI targets that need special setup)
    local CRATES=(
        s1-model s1-ops
        s1-format-docx s1-format-odt s1-format-pdf s1-format-txt s1-format-md
        s1-format-html s1-format-rtf s1-format-xlsx
        s1-convert s1-crdt s1-layout s1-text
        s1engine
    )

    local total_pass=0
    local total_fail=0
    local crate_lines=""

    for crate in "${CRATES[@]}"; do
        local output
        if output=$(cargo test -p "$crate" 2>&1); then
            # Parse pass/fail counts from cargo test output
            local result_line
            result_line=$(echo "$output" | grep -E '^test result:' | tail -1 || true)
            local passed=0 failed=0 ignored=0
            if [[ -n "$result_line" ]]; then
                passed=$(echo "$result_line" | grep -oE '[0-9]+ passed' | grep -oE '[0-9]+' || echo 0)
                failed=$(echo "$result_line" | grep -oE '[0-9]+ failed' | grep -oE '[0-9]+' || echo 0)
                ignored=$(echo "$result_line" | grep -oE '[0-9]+ ignored' | grep -oE '[0-9]+' || echo 0)
            fi
            total_pass=$((total_pass + passed))
            total_fail=$((total_fail + failed))
            if [[ "$failed" -gt 0 ]]; then
                fail "$crate: $passed passed, $failed FAILED, $ignored ignored"
                crate_lines+="| $crate | $passed | $failed | $ignored | FAIL |\n"
            else
                ok "$crate: $passed passed, $ignored ignored"
                crate_lines+="| $crate | $passed | $failed | $ignored | PASS |\n"
            fi
        else
            local result_line
            result_line=$(echo "$output" | grep -E '^test result:' | tail -1 || true)
            local passed=0 failed=0 ignored=0
            if [[ -n "$result_line" ]]; then
                passed=$(echo "$result_line" | grep -oE '[0-9]+ passed' | grep -oE '[0-9]+' || echo 0)
                failed=$(echo "$result_line" | grep -oE '[0-9]+ failed' | grep -oE '[0-9]+' || echo 0)
                ignored=$(echo "$result_line" | grep -oE '[0-9]+ ignored' | grep -oE '[0-9]+' || echo 0)
            fi
            total_pass=$((total_pass + passed))
            total_fail=$((total_fail + failed))
            fail "$crate: $passed passed, $failed FAILED, $ignored ignored"
            crate_lines+="| $crate | $passed | $failed | $ignored | FAIL |\n"
        fi
    done

    CRATE_RESULTS="$crate_lines"
    TOTAL_PASS=$total_pass
    TOTAL_FAIL=$total_fail

    printf "\n  Total: ${BOLD}%d passed${RESET}, ${BOLD}%d failed${RESET}\n" "$total_pass" "$total_fail"

    if [[ "$total_fail" -gt 0 ]]; then
        section_fail "Workspace Tests ($total_pass passed, $total_fail failed)"
    else
        section_pass "Workspace Tests ($total_pass passed)"
    fi
}

# ============================================================================
# STEP 2: Feature Coverage Integration Tests
# ============================================================================

FEATURE_COVERAGE_OUTPUT=""
FEATURE_COVERAGE_PASS=0
FEATURE_COVERAGE_FAIL=0

run_feature_coverage() {
    banner "Step 2: Feature Coverage Integration Tests"

    local output
    if output=$(cargo test -p s1engine --test feature_coverage 2>&1); then
        FEATURE_COVERAGE_OUTPUT="$output"
        local result_line
        result_line=$(echo "$output" | grep -E '^test result:' | tail -1 || true)
        FEATURE_COVERAGE_PASS=$(echo "$result_line" | grep -oE '[0-9]+ passed' | grep -oE '[0-9]+' || echo 0)
        FEATURE_COVERAGE_FAIL=$(echo "$result_line" | grep -oE '[0-9]+ failed' | grep -oE '[0-9]+' || echo 0)
        ok "feature_coverage: $FEATURE_COVERAGE_PASS passed"
        section_pass "Feature Coverage ($FEATURE_COVERAGE_PASS passed)"
    else
        FEATURE_COVERAGE_OUTPUT="$output"
        local result_line
        result_line=$(echo "$output" | grep -E '^test result:' | tail -1 || true)
        FEATURE_COVERAGE_PASS=$(echo "$result_line" | grep -oE '[0-9]+ passed' | grep -oE '[0-9]+' || echo 0)
        FEATURE_COVERAGE_FAIL=$(echo "$result_line" | grep -oE '[0-9]+ failed' | grep -oE '[0-9]+' || echo 0)
        fail "feature_coverage: $FEATURE_COVERAGE_PASS passed, $FEATURE_COVERAGE_FAIL FAILED"
        section_fail "Feature Coverage ($FEATURE_COVERAGE_PASS passed, $FEATURE_COVERAGE_FAIL failed)"
    fi

    # Extract individual test results
    FEATURE_TEST_LINES=$(echo "$FEATURE_COVERAGE_OUTPUT" | grep -E '^test t[0-9]' | sort || true)
}

# ============================================================================
# STEP 3: Round-Trip Fidelity Tests (subset of feature_coverage)
# ============================================================================

run_fidelity_tests() {
    banner "Step 3: Round-Trip Fidelity Tests"

    local fidelity_pass=0
    local fidelity_fail=0
    local fidelity_lines=""

    # These test names correspond to round-trip tests in feature_coverage.rs
    local -a FIDELITY_TESTS=(
        "t02_docx_roundtrip|DOCX"
        "t03_odt_roundtrip|ODT"
        "t04_txt_roundtrip|Plain Text"
        "t05_markdown_roundtrip|Markdown"
        "t19_metadata_docx_roundtrip|DOCX Metadata"
        "t26_empty_document_roundtrip|DOCX Empty"
        "t29_complex_docx_export|DOCX Complex"
        "t31_html_roundtrip_standalone|HTML"
        "t32_rtf_roundtrip_standalone|RTF"
    )

    for entry in "${FIDELITY_TESTS[@]}"; do
        local test_name="${entry%%|*}"
        local format_name="${entry##*|}"

        if echo "$FEATURE_TEST_LINES" | grep -q "$test_name.*ok"; then
            ok "Round-trip: $format_name"
            fidelity_lines+="| $format_name | PASS | Round-trip preserved |\n"
            fidelity_pass=$((fidelity_pass + 1))
        elif echo "$FEATURE_TEST_LINES" | grep -q "$test_name.*FAILED"; then
            fail "Round-trip: $format_name"
            fidelity_lines+="| $format_name | FAIL | See test output |\n"
            fidelity_fail=$((fidelity_fail + 1))
        else
            skip "Round-trip: $format_name (not executed)"
            fidelity_lines+="| $format_name | SKIP | Feature not enabled |\n"
        fi
    done

    # Also check PDF export (not a round-trip but verify header)
    if echo "$FEATURE_TEST_LINES" | grep -q "t06_pdf_export_valid_header.*ok"; then
        ok "Export: PDF (%%PDF- header verified)"
        fidelity_lines+="| PDF (export) | PASS | %%PDF- header verified |\n"
        fidelity_pass=$((fidelity_pass + 1))
    elif echo "$FEATURE_TEST_LINES" | grep -q "t06_pdf_export_valid_header.*FAILED"; then
        fail "Export: PDF"
        fidelity_lines+="| PDF (export) | FAIL | See test output |\n"
        fidelity_fail=$((fidelity_fail + 1))
    else
        skip "Export: PDF (pdf feature not enabled)"
        fidelity_lines+="| PDF (export) | SKIP | Feature not enabled |\n"
    fi

    FIDELITY_RESULTS="$fidelity_lines"
    FIDELITY_PASS=$fidelity_pass
    FIDELITY_FAIL=$fidelity_fail

    if [[ "$fidelity_fail" -gt 0 ]]; then
        section_fail "Fidelity ($fidelity_pass passed, $fidelity_fail failed)"
    else
        section_pass "Fidelity ($fidelity_pass passed)"
    fi
}

# ============================================================================
# STEP 4: WASM Build
# ============================================================================

WASM_STATUS="SKIP"
WASM_SIZE=""

run_wasm_build() {
    if [[ "$SKIP_WASM" == true ]]; then
        banner "Step 4: WASM Build (skipped)"
        skip "WASM build skipped (--skip-wasm)"
        section_skip "WASM Build"
        return
    fi

    banner "Step 4: WASM Build"

    if ! command -v wasm-pack &> /dev/null; then
        skip "wasm-pack not installed"
        WASM_STATUS="SKIP (wasm-pack not found)"
        section_skip "WASM Build (wasm-pack not found)"
        return
    fi

    local wasm_crate="$PROJECT_DIR/ffi/wasm"
    local out_dir="$PROJECT_DIR/demo/pkg"

    if wasm-pack build "$wasm_crate" --target web --release --out-dir "../../demo/pkg" 2>&1; then
        WASM_STATUS="PASS"
        if [[ -f "$out_dir/s1engine_wasm_bg.wasm" ]]; then
            WASM_SIZE=$(ls -lh "$out_dir/s1engine_wasm_bg.wasm" | awk '{print $5}')
            ok "WASM build succeeded (${WASM_SIZE})"
        else
            ok "WASM build succeeded"
        fi
        section_pass "WASM Build (${WASM_SIZE:-unknown size})"
    else
        WASM_STATUS="FAIL"
        fail "WASM build failed"
        section_fail "WASM Build"
    fi
}

# ============================================================================
# STEP 5: Code Quality Checks
# ============================================================================

CLIPPY_STATUS="SKIP"
FMT_STATUS="SKIP"

run_quality_checks() {
    if [[ "$QUICK" == true ]]; then
        banner "Step 5: Code Quality (skipped in --quick mode)"
        section_skip "Clippy"
        section_skip "Formatting"
        return
    fi

    banner "Step 5: Code Quality Checks"

    # Clippy
    if cargo clippy --workspace -- -D warnings 2>&1 | tail -5; then
        CLIPPY_STATUS="PASS"
        ok "Clippy: zero warnings"
        section_pass "Clippy"
    else
        CLIPPY_STATUS="FAIL"
        fail "Clippy: warnings or errors found"
        section_fail "Clippy"
    fi

    # Formatting
    if cargo fmt --all -- --check 2>&1; then
        FMT_STATUS="PASS"
        ok "Formatting: correct"
        section_pass "Formatting"
    else
        FMT_STATUS="FAIL"
        fail "Formatting: issues found"
        section_fail "Formatting"
    fi
}

# ============================================================================
# Generate Report
# ============================================================================

generate_report() {
    banner "Generating Report"

    mkdir -p "$(dirname "$REPORT")"

    # Build feature coverage checklist from test results
    local feature_checklist=""
    local -a FEATURE_CHECKS=(
        "t01_builder_creates_non_empty_document|Document Builder API"
        "t02_docx_roundtrip|DOCX Export/Import"
        "t03_odt_roundtrip|ODT Export/Import"
        "t04_txt_roundtrip|TXT Export/Import"
        "t05_markdown_roundtrip|Markdown Export/Import"
        "t06_pdf_export_valid_header|PDF Export"
        "t07_text_extraction|Text Extraction"
        "t08_paragraph_insert_via_operation|Paragraph Operations"
        "t09_table_creation|Table Creation"
        "t10_rich_table_cells|Rich Table Cells"
        "t11_style_system|Style System"
        "t12_undo_redo|Undo/Redo"
        "t13_transaction_batching|Transaction Batching"
        "t14_track_changes_detection|Track Changes Detection"
        "t15_accept_all_changes|Accept Track Changes"
        "t16_image_insertion|Image/Media Insertion"
        "t17_heading_collection|Heading Collection"
        "t18_metadata_get_set|Metadata Get/Set"
        "t19_metadata_docx_roundtrip|Metadata DOCX Round-Trip"
        "t20_format_detection|Format Detection"
        "t21_lists|Bullet/Numbered Lists"
        "t22_inline_formatting_varieties|Inline Formatting (bold/italic/underline/super/sub/color/font)"
        "t23_hyperlinks_and_bookmarks|Hyperlinks and Bookmarks"
        "t24_sections_headers_footers|Sections/Headers/Footers"
        "t25_table_of_contents|Table of Contents"
        "t26_empty_document_roundtrip|Empty Document Round-Trip"
        "t27_clear_history|Clear Undo History"
        "t28_insert_text_operation|Insert Text Operation"
        "t29_complex_docx_export|Complex DOCX Export"
        "t30_paragraph_ids|Paragraph ID Listing"
        "t31_html_roundtrip_standalone|HTML Round-Trip (standalone crate)"
        "t32_rtf_roundtrip_standalone|RTF Round-Trip (standalone crate)"
        "t33_media_store_dedup|Media Store Deduplication"
        "t34_undo_cap|Undo History Cap"
        "t35_reject_all_changes|Reject Track Changes"
    )

    for entry in "${FEATURE_CHECKS[@]}"; do
        local test_name="${entry%%|*}"
        local feature_name="${entry##*|}"

        if echo "$FEATURE_TEST_LINES" | grep -q "$test_name.*ok"; then
            feature_checklist+="- [x] $feature_name\n"
        elif echo "$FEATURE_TEST_LINES" | grep -q "$test_name.*FAILED"; then
            feature_checklist+="- [ ] $feature_name (FAILED)\n"
        else
            feature_checklist+="- [ ] $feature_name (not executed)\n"
        fi
    done

    # Overall status string
    local status_emoji
    if [[ "$OVERALL_STATUS" -eq 0 ]]; then
        status_emoji="ALL PASSED"
    else
        status_emoji="FAILURES DETECTED"
    fi

    cat > "$REPORT" << REPORT_EOF
# s1engine Feature Test Report

**Status:** $status_emoji
**Date:** $RUN_DATE
**Commit:** \`$COMMIT_HASH\` ($COMMIT_MSG)
**Branch:** $BRANCH

---

## Summary

| Step | Result |
|------|--------|
$(for r in "${SECTION_RESULTS[@]}"; do
    _status="${r%%|*}"
    _desc="${r##*|}"
    echo "| $_desc | $_status |"
done)

---

## 1. Rust Test Suite Results (per crate)

| Crate | Passed | Failed | Ignored | Status |
|-------|--------|--------|---------|--------|
$(printf '%b' "$CRATE_RESULTS")
| **Total** | **${TOTAL_PASS:-0}** | **${TOTAL_FAIL:-0}** | | |

---

## 2. Feature Coverage Tests

Test file: \`crates/s1engine/tests/feature_coverage.rs\`

- Passed: **$FEATURE_COVERAGE_PASS**
- Failed: **$FEATURE_COVERAGE_FAIL**

### Individual Results

| Test | Status |
|------|--------|
$(echo "$FEATURE_TEST_LINES" | while IFS= read -r line; do
    if [[ -z "$line" ]]; then continue; fi
    _name=$(echo "$line" | awk '{print $2}')
    _tstatus=$(echo "$line" | awk '{print $NF}')
    if [[ "$_tstatus" == "ok" ]]; then
        echo "| \`$_name\` | PASS |"
    else
        echo "| \`$_name\` | FAIL |"
    fi
done)

---

## 3. Round-Trip Fidelity Results

| Format | Status | Notes |
|--------|--------|-------|
$(printf '%b' "${FIDELITY_RESULTS:-}")

---

## 4. WASM Build

- **Status:** $WASM_STATUS
$(if [[ -n "$WASM_SIZE" ]]; then echo "- **Binary size:** $WASM_SIZE"; fi)

---

## 5. Code Quality

- **Clippy:** $CLIPPY_STATUS
- **Formatting:** $FMT_STATUS

---

## 6. Feature Coverage Checklist

$(printf '%b' "$feature_checklist")

---

*Report generated by \`scripts/feature-test.sh\`*
REPORT_EOF

    ok "Report written to docs/FEATURE_TEST_REPORT.md"
}

# ============================================================================
# Main
# ============================================================================

printf "${BOLD}s1engine Feature Test & Fidelity Analysis${RESET}\n"
printf "Date: %s  Commit: %s  Branch: %s\n" "$RUN_DATE" "$COMMIT_HASH" "$BRANCH"

if [[ "$QUICK" == true ]]; then
    printf "${YELLOW}Quick mode: running only feature_coverage tests${RESET}\n"
    # Minimal workspace test (just s1engine) for crate results
    CRATE_RESULTS="| s1engine | - | - | - | - |\n"
    TOTAL_PASS=0
    TOTAL_FAIL=0
    run_feature_coverage
    run_fidelity_tests
else
    run_workspace_tests
    run_feature_coverage
    run_fidelity_tests
    run_wasm_build
    run_quality_checks
fi

generate_report

# Final summary
printf "\n${BOLD}────────────────────────────────────────${RESET}\n"
if [[ "$OVERALL_STATUS" -eq 0 ]]; then
    printf "${GREEN}${BOLD}ALL STEPS PASSED${RESET}\n"
else
    printf "${RED}${BOLD}SOME STEPS FAILED — see report for details${RESET}\n"
fi
printf "Report: %s\n" "$REPORT"
printf "${BOLD}────────────────────────────────────────${RESET}\n"

exit $OVERALL_STATUS
