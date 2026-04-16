#!/usr/bin/env bash
#
# test-runner.sh -- Black-box test runner for crowser
#
# Runs all test cases for a given phase against the crowser binary.
# Each test case is a directory containing input files and expected output.
# JSON output is compared structurally using jq. PNG output is checked
# for existence and non-zero size.
#
# Usage:
#   ./tools/test-runner.sh <phase> <binary-path>
#   ./tools/test-runner.sh 1 ./target/debug/crowser
#   ./tools/test-runner.sh --help
#
# Requirements:
#   - bash 4+
#   - jq (for JSON comparison)
#
# Exit codes:
#   0 - All tests passed
#   1 - One or more tests failed
#   2 - Usage error or missing dependencies

set -euo pipefail

# ---------------------------------------------------------------------------
# Colors
# ---------------------------------------------------------------------------

if [ -t 1 ]; then
    GREEN='\033[0;32m'
    RED='\033[0;31m'
    YELLOW='\033[0;33m'
    BOLD='\033[1m'
    DIM='\033[2m'
    RESET='\033[0m'
else
    GREEN=''
    RED=''
    YELLOW=''
    BOLD=''
    DIM=''
    RESET=''
fi

# ---------------------------------------------------------------------------
# Help
# ---------------------------------------------------------------------------

usage() {
    cat <<'USAGE'
Usage: test-runner.sh <phase> <binary-path>

Arguments:
  phase          Phase number (1-8)
  binary-path    Path to the compiled crowser binary

Options:
  --help, -h     Show this help message

Examples:
  ./tools/test-runner.sh 1 ./target/debug/crowser
  ./tools/test-runner.sh 4 ./target/release/crowser

Phases:
  1  HTML tokenizer    (tokenize)
  2  HTML parser       (parse)
  3  CSS tokenizer     (tokenize-css)
  4  CSS parser        (parse-css)
  5  Style resolution  (style)
  6  Layout            (layout)
  7  Painting          (paint)
  8  The window        (open) -- no automated tests
USAGE
}

# ---------------------------------------------------------------------------
# Phase -> subcommand / directory mapping
# ---------------------------------------------------------------------------

phase_subcommand() {
    case "$1" in
        1) echo "tokenize" ;;
        2) echo "parse" ;;
        3) echo "tokenize-css" ;;
        4) echo "parse-css" ;;
        5) echo "style" ;;
        6) echo "layout" ;;
        7) echo "paint" ;;
        8) echo "open" ;;
        *) echo "" ;;
    esac
}

phase_directory() {
    case "$1" in
        1) echo "phase-01-html-tokenizer" ;;
        2) echo "phase-02-html-parser" ;;
        3) echo "phase-03-css-tokenizer" ;;
        4) echo "phase-04-css-parser" ;;
        5) echo "phase-05-style-resolution" ;;
        6) echo "phase-06-layout" ;;
        7) echo "phase-07-painting" ;;
        8) echo "phase-08-the-window" ;;
        *) echo "" ;;
    esac
}

# Determine the input file extension for the phase.
phase_input_ext() {
    case "$1" in
        3|4) echo "css" ;;
        *)   echo "html" ;;
    esac
}

# ---------------------------------------------------------------------------
# Argument parsing
# ---------------------------------------------------------------------------

if [ $# -eq 0 ] || [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    usage
    exit 0
fi

if [ $# -lt 2 ]; then
    echo -e "${RED}Error: expected 2 arguments, got $#${RESET}" >&2
    echo "Run with --help for usage." >&2
    exit 2
fi

PHASE="$1"
BINARY="$2"

# Validate phase number.
if ! [[ "$PHASE" =~ ^[1-8]$ ]]; then
    echo -e "${RED}Error: phase must be a number between 1 and 8, got '$PHASE'${RESET}" >&2
    exit 2
fi

# Phase 8 has no automated tests.
if [ "$PHASE" = "8" ]; then
    echo -e "${YELLOW}Phase 8 (the window) has no automated tests. Verify visually.${RESET}"
    exit 0
fi

# Validate binary exists and is executable.
if [ ! -f "$BINARY" ]; then
    echo -e "${RED}Error: binary not found at '$BINARY'${RESET}" >&2
    exit 2
fi

if [ ! -x "$BINARY" ]; then
    echo -e "${RED}Error: '$BINARY' is not executable${RESET}" >&2
    exit 2
fi

# Make binary path absolute so it works regardless of cwd.
BINARY="$(cd "$(dirname "$BINARY")" && pwd)/$(basename "$BINARY")"

# Check for jq (required for JSON comparison).
if ! command -v jq &>/dev/null; then
    echo -e "${RED}Error: jq is required but not found. Install it with your package manager.${RESET}" >&2
    exit 2
fi

# ---------------------------------------------------------------------------
# Locate the spec directory
# ---------------------------------------------------------------------------

# Find the repo root. The script lives in tools/ at the repo root.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

PHASE_DIR_NAME="$(phase_directory "$PHASE")"
TESTS_DIR="$REPO_ROOT/spec/$PHASE_DIR_NAME/tests"

if [ ! -d "$TESTS_DIR" ]; then
    echo -e "${YELLOW}No tests directory found at $TESTS_DIR${RESET}"
    echo -e "${YELLOW}Create test cases under spec/$PHASE_DIR_NAME/tests/ to get started.${RESET}"
    exit 0
fi

# Count test cases (each subdirectory is a test case).
TEST_CASES=()
for dir in "$TESTS_DIR"/*/; do
    [ -d "$dir" ] && TEST_CASES+=("$dir")
done

if [ ${#TEST_CASES[@]} -eq 0 ]; then
    echo -e "${YELLOW}No test cases found in $TESTS_DIR${RESET}"
    exit 0
fi

# ---------------------------------------------------------------------------
# Utilities
# ---------------------------------------------------------------------------

SUBCOMMAND="$(phase_subcommand "$PHASE")"
INPUT_EXT="$(phase_input_ext "$PHASE")"

# Compare two JSON files structurally using jq.
# Returns 0 if equivalent, 1 if different.
json_compare() {
    local actual_file="$1"
    local expected_file="$2"

    # Normalize both: sort keys, consistent formatting.
    local actual_normalized expected_normalized
    actual_normalized="$(jq -S '.' "$actual_file" 2>/dev/null)" || return 1
    expected_normalized="$(jq -S '.' "$expected_file" 2>/dev/null)" || return 1

    if [ "$actual_normalized" = "$expected_normalized" ]; then
        return 0
    else
        return 1
    fi
}

# ---------------------------------------------------------------------------
# Run tests
# ---------------------------------------------------------------------------

PASSED=0
FAILED=0
SKIPPED=0

echo ""
echo -e "${BOLD}crowser test runner${RESET}"
echo -e "${DIM}Phase $PHASE: $SUBCOMMAND${RESET}"
echo -e "${DIM}Binary: $BINARY${RESET}"
echo -e "${DIM}Tests:  $TESTS_DIR${RESET}"
echo ""

for test_dir in "${TEST_CASES[@]}"; do
    test_name="$(basename "$test_dir")"

    # Find the input file.
    input_file="$test_dir/input.$INPUT_EXT"
    if [ ! -f "$input_file" ]; then
        echo -e "  ${YELLOW}SKIP${RESET}  $test_name  ${DIM}(no input.$INPUT_EXT found)${RESET}"
        SKIPPED=$((SKIPPED + 1))
        continue
    fi

    # Determine expected exit code (default: 0).
    expected_exit=0
    if [ -f "$test_dir/exit_code" ]; then
        expected_exit="$(cat "$test_dir/exit_code" | tr -d '[:space:]')"
    fi

    # --- Phase 7 (paint): PNG output ---
    if [ "$PHASE" = "7" ]; then
        expected_png="$test_dir/expected.png"

        # Create a temp file for the actual output.
        actual_png="$(mktemp /tmp/crowser-test-XXXXXX.png)"
        trap "rm -f '$actual_png'" EXIT

        # Run the binary.
        actual_exit=0
        "$BINARY" paint "$input_file" -o "$actual_png" 2>/dev/null || actual_exit=$?

        # Check exit code.
        if [ "$actual_exit" -ne "$expected_exit" ]; then
            echo -e "  ${RED}FAIL${RESET}  $test_name  ${DIM}(exit code: expected $expected_exit, got $actual_exit)${RESET}"
            FAILED=$((FAILED + 1))
            rm -f "$actual_png"
            continue
        fi

        # For success cases, check that the PNG was created with non-zero size.
        if [ "$expected_exit" -eq 0 ]; then
            if [ ! -s "$actual_png" ]; then
                echo -e "  ${RED}FAIL${RESET}  $test_name  ${DIM}(output PNG is missing or empty)${RESET}"
                FAILED=$((FAILED + 1))
                rm -f "$actual_png"
                continue
            fi

            # If there is a reference PNG, note that visual comparison is manual.
            if [ -f "$expected_png" ]; then
                echo -e "  ${GREEN}PASS${RESET}  $test_name  ${DIM}(PNG created; visual diff is manual)${RESET}"
            else
                echo -e "  ${GREEN}PASS${RESET}  $test_name  ${DIM}(PNG created; no reference image)${RESET}"
            fi
        else
            echo -e "  ${GREEN}PASS${RESET}  $test_name  ${DIM}(expected error, exit code $actual_exit)${RESET}"
        fi

        PASSED=$((PASSED + 1))
        rm -f "$actual_png"
        continue
    fi

    # --- Phases 1-6: JSON output ---
    expected_json="$test_dir/expected.json"
    if [ ! -f "$expected_json" ] && [ "$expected_exit" -eq 0 ]; then
        echo -e "  ${YELLOW}SKIP${RESET}  $test_name  ${DIM}(no expected.json found)${RESET}"
        SKIPPED=$((SKIPPED + 1))
        continue
    fi

    # Run the binary and capture stdout.
    actual_json="$(mktemp /tmp/crowser-test-XXXXXX.json)"
    actual_exit=0
    "$BINARY" "$SUBCOMMAND" "$input_file" > "$actual_json" 2>/dev/null || actual_exit=$?

    # Check exit code.
    if [ "$actual_exit" -ne "$expected_exit" ]; then
        echo -e "  ${RED}FAIL${RESET}  $test_name  ${DIM}(exit code: expected $expected_exit, got $actual_exit)${RESET}"
        FAILED=$((FAILED + 1))
        rm -f "$actual_json"
        continue
    fi

    # If the expected exit code is non-zero, we only check the exit code (not output).
    if [ "$expected_exit" -ne 0 ]; then
        echo -e "  ${GREEN}PASS${RESET}  $test_name  ${DIM}(expected error, exit code $actual_exit)${RESET}"
        PASSED=$((PASSED + 1))
        rm -f "$actual_json"
        continue
    fi

    # Compare JSON output structurally.
    if json_compare "$actual_json" "$expected_json"; then
        echo -e "  ${GREEN}PASS${RESET}  $test_name"
        PASSED=$((PASSED + 1))
    else
        echo -e "  ${RED}FAIL${RESET}  $test_name  ${DIM}(JSON output mismatch)${RESET}"

        # Show a brief diff for debugging.
        echo -e "${DIM}"
        diff --color=auto \
            <(jq -S '.' "$expected_json" 2>/dev/null || cat "$expected_json") \
            <(jq -S '.' "$actual_json" 2>/dev/null || cat "$actual_json") \
            | head -20 || true
        echo -e "${RESET}"

        FAILED=$((FAILED + 1))
    fi

    rm -f "$actual_json"
done

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------

TOTAL=$((PASSED + FAILED + SKIPPED))

echo ""
echo -e "${BOLD}Results:${RESET}"
echo -e "  ${GREEN}$PASSED passed${RESET}"
if [ "$FAILED" -gt 0 ]; then
    echo -e "  ${RED}$FAILED failed${RESET}"
fi
if [ "$SKIPPED" -gt 0 ]; then
    echo -e "  ${YELLOW}$SKIPPED skipped${RESET}"
fi
echo -e "  $TOTAL total"
echo ""

if [ "$FAILED" -gt 0 ]; then
    exit 1
fi

exit 0
