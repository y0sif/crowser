<div align="center">

# crowser

**From `<html>` to pixels -- build a browser engine from scratch in Rust.**

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](#license)
[![Phases](https://img.shields.io/badge/phases-8_core-CE422B.svg)](#the-journey)
[![Language](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)

</div>

---

You use a browser every day. You've probably never built one. **crowser** walks you through constructing a real browser engine from the ground up: HTML tokenizer, CSS parser, style resolution, layout, painting, and finally a windowed renderer that puts pixels on screen. Not a toy. Not a simplified mockup. A working engine you understand because you wrote every line. CodeCrafters covers git, Redis, shells, and HTTP servers, but there's nothing like this for browser engines. crowser fills that gap.

## What You'll Build

By the end of the eight core phases, you'll have a standalone binary that reads an HTML file (with embedded CSS), computes styles and layout, and renders the result in an actual window on your screen. Along the way, each phase produces its own inspectable output (JSON, PNG), so you can see exactly what your engine is doing at every stage.

## The Journey

| Phase | Name | What You'll Build | Output |
|:-----:|------|-------------------|--------|
| 1 | **HTML Tokenizer** | A lexer that turns raw HTML into a stream of tokens (start tags, end tags, text, attributes) | JSON token array |
| 2 | **HTML Parser** | A tree builder that assembles tokens into a DOM tree | JSON DOM tree |
| 3 | **CSS Tokenizer** | A lexer for CSS (selectors, properties, values, delimiters) | JSON token array |
| 4 | **CSS Parser** | A parser that produces a structured stylesheet from CSS tokens | JSON stylesheet |
| 5 | **Style Resolution** | The engine that matches CSS rules to DOM nodes and computes final styles | JSON styled tree |
| 6 | **Layout** | A layout engine that computes positions and sizes using the box model | JSON layout tree |
| 7 | **Painting** | A renderer that walks the layout tree and produces actual pixels | PNG image |
| 8 | **The Window** | The final piece: display your rendered page in a real OS window | A running window |

### Stretch Goals

Once the core is solid, keep going:

- **Networking** -- fetch pages over HTTP
- **Text rendering** -- proper font shaping and glyph rendering
- **Scrolling** -- handle documents taller than the viewport
- **Interactivity** -- click events, links, basic input

## How It Works

crowser is **spec-driven**. Each phase comes with a spec that defines what your engine must do, expressed as black-box tests: feed your binary an input file, compare the output against the expected result. You pass the tests, you've built it correctly.

You own the design. The specs tell you *what*, not *how*. If you want more structure, each phase includes optional **guided hints** that suggest architecture patterns and data structures. Use them or ignore them.

```text
spec says: crowser tokenize hello.html -> expected_tokens.json
you write: the code that makes it happen
tests run: your output vs. expected output
```

This means you can approach each phase your own way. Minimal and scrappy, or clean and over-engineered. As long as the output matches, you're good.

## Getting Started

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (stable, latest)
- `jq` (for JSON comparison in the test runner)
- A terminal
- Curiosity about how browsers actually work

### Quick Start

```bash
# Clone the tutorial repo
git clone https://github.com/y0sif/crowser.git
cd crowser

# Create your implementation branch
git checkout -b my-implementation

# Initialize your Rust project
cargo init --name crowser

# Start reading
cat guide/README.md
```

Start with Phase 1. Read the guide (`guide/01-html-tokenizer.md`), read the spec (`spec/phase-01-html-tokenizer/SPEC.md`), look at the test cases, and start writing code.

```bash
# Build your engine
cargo build

# Run your engine against a test case
./target/debug/crowser tokenize spec/phase-01-html-tokenizer/tests/01-simple-element/input.html

# Run all tests for a phase
./tools/test-runner.sh 1 ./target/debug/crowser
```

## Repo Structure

The `main` branch is the tutorial. Your implementation lives on your own branch.

```text
crowser/
├── guide/                              # Tutorial chapters
│   ├── README.md                       # Overview and how to use
│   ├── 00-introduction.md              # Setup and orientation
│   ├── 01-html-tokenizer.md            # Phase 1 guide
│   ├── 02-html-parser.md               # Phase 2 guide
│   ├── ...                             # Phases 3-8
│   └── stretch/                        # Stretch goal guides
├── spec/                               # Phase specs and test fixtures
│   ├── cli-interface.md                # Full CLI contract and JSON schemas
│   ├── phase-01-html-tokenizer/
│   │   ├── SPEC.md                     # What this phase must do
│   │   └── tests/                      # Black-box test cases
│   │       ├── 01-simple-element/
│   │       │   ├── input.html          # Test input
│   │       │   └── expected.json       # Expected output
│   │       └── ...
│   ├── phase-02-html-parser/
│   └── ...                             # Phases 3-8
├── hints/                              # Optional implementation hints
│   ├── phase-01/hints.md               # Suggested Rust types and tips
│   ├── phase-02/hints.md
│   └── ...                             # Phases 3-8
├── tools/
│   └── test-runner.sh                  # Black-box test runner
├── .github/                            # Issue templates, CI
├── README.md
├── CONTRIBUTING.md
└── LICENSE
```

## Running Tests

Tests are black-box: the test runner invokes your `crowser` binary with input files and compares output against expected results. JSON comparison is structural (key order and whitespace don't matter).

```bash
# Run all tests for a specific phase
./tools/test-runner.sh 1 ./target/debug/crowser

# Test output shows PASS/FAIL for each case with diffs on failure
```

To debug manually:

```bash
# See your output
./target/debug/crowser tokenize spec/phase-01-html-tokenizer/tests/01-simple-element/input.html

# Compare against expected
diff <(./target/debug/crowser tokenize spec/phase-01-html-tokenizer/tests/01-simple-element/input.html | jq -S .) \
     <(jq -S . spec/phase-01-html-tokenizer/tests/01-simple-element/expected.json)
```

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on adding test cases, improving specs, or fixing bugs.

## License

MIT. See [LICENSE](LICENSE) for details.
