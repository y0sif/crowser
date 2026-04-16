# Spec System

This directory contains the specifications and test suites for **crowser**, organized by phase. Each phase builds on the previous one, guiding you from raw HTML tokenization all the way to rendering pixels in a window.

## Directory Structure

```
spec/
  cli-interface.md              # The CLI contract (start here)
  README.md                     # This file
  phase-01-html-tokenizer/
    SPEC.md                     # What to build and why
    schema.json                 # (optional) JSON Schema for output validation
    tests/
      basic-tag/
        input.html              # Input fed to crowser
        expected.json           # Expected stdout (JSON)
      self-closing/
        input.html
        expected.json
      ...
  phase-02-html-parser/
    SPEC.md
    tests/
      simple-tree/
        input.html
        expected.json
      ...
  ...
  phase-07-painting/
    SPEC.md
    tests/
      red-box/
        input.html
        expected.png            # Expected rendered output
      ...
  phase-08-the-window/
    SPEC.md                     # No automated tests (interactive)
```

## How Tests Work

Tests are **black-box**. They know nothing about your code's internal structure. The test runner invokes your compiled `crowser` binary with a subcommand, feeds it an input file, and compares the output against an expected result.

### The flow for each test case

1. The test runner reads the test directory to find the input file (`input.html`, `input.css`, etc.) and the expected output file (`expected.json`, `expected.png`).
2. It builds the command from the phase's subcommand and the input file path:
   ```
   ./target/debug/crowser tokenize tests/basic-tag/input.html
   ```
3. It captures stdout and the exit code.
4. It compares the actual output against the expected output.
5. It reports PASS or FAIL.

### JSON comparison

JSON output is compared **structurally**, not textually:

- Key order does not matter: `{"a":1,"b":2}` equals `{"b":2,"a":1}`
- Whitespace does not matter: pretty-printed and minified JSON are equivalent
- Types must match exactly: `"1"` (string) is not equal to `1` (number)
- Array order **does** matter: `[1,2]` is not equal to `[2,1]`

The test runner normalizes both the expected and actual JSON using `jq`, then diffs the results.

### PNG comparison

For Phase 7 (painting), the output is a PNG file. Currently, comparison is basic:

- The test runner checks that the output file was created and has non-zero size.
- Full visual comparison (perceptual diff) is a stretch goal. For now, review rendered output manually or do exact byte-level comparison against a reference image.

### Exit codes

Your binary must exit with specific codes:

| Code | Meaning |
|------|---------|
| `0`  | Success |
| `1`  | Parse error (malformed input) |
| `2`  | File not found |

The test runner checks exit codes when a test expects a specific one (e.g., a test for malformed input expects exit code 1).

## Writing Your Own Tests

Each test case is a directory. At minimum, it contains:

- An input file (`input.html` or `input.css`)
- An expected output file (`expected.json` or `expected.png`)

Optionally, a test directory can include:

- `description.txt` -- a one-line description of what the test covers
- `exit_code` -- a file containing the expected exit code (defaults to `0` if absent)

To add a test, create a new directory under the appropriate phase's `tests/` folder, drop in your input and expected output, and run the test runner.

## Running Tests

Use the test runner script:

```bash
# Run all Phase 1 tests against your debug build
./tools/test-runner.sh 1 ./target/debug/crowser

# Run all Phase 4 tests against a release build
./tools/test-runner.sh 4 ./target/release/crowser
```

See `./tools/test-runner.sh --help` for full usage.

## Phase Overview

| Phase | Subcommand      | Input    | Output        |
|-------|-----------------|----------|---------------|
| 1     | `tokenize`      | HTML     | JSON tokens   |
| 2     | `parse`         | HTML     | JSON DOM tree |
| 3     | `tokenize-css`  | CSS      | JSON tokens   |
| 4     | `parse-css`     | CSS      | JSON stylesheet |
| 5     | `style`         | HTML     | JSON styled tree |
| 6     | `layout`        | HTML     | JSON layout tree |
| 7     | `paint`         | HTML     | PNG file      |
| 8     | `open`          | HTML     | Window (interactive) |

Each phase's `SPEC.md` explains the theory, requirements, and edge cases in detail. The `cli-interface.md` file defines the exact JSON schemas and CLI contract for every subcommand.
