# crowser

A tutorial project for building a browser engine from scratch in Rust.

## Project Type

This is NOT a typical code project. The `main` branch contains tutorial content (guides, specs, test fixtures, hints). User implementations live on separate branches.

## Structure

- `guide/` — Tutorial chapters (one per phase)
- `spec/` — Phase specs and black-box test fixtures
- `hints/` — Optional Rust implementation hints
- `tools/` — Test runner and utilities

## Content Conventions

- Specs define WHAT, not HOW. They are the source of truth for each phase.
- Guides teach concepts and approaches. They include pseudocode, not full solutions.
- Hints provide Rust-specific type suggestions and architecture tips. They are optional.
- Test fixtures: each test case is a directory with input files and expected output (JSON or description).

## CLI Interface

The user's engine binary exposes subcommands for each phase:
```
crowser tokenize <file.html>           # Phase 1
crowser parse <file.html>              # Phase 2
crowser tokenize-css <file.css>        # Phase 3
crowser parse-css <file.css>           # Phase 4
crowser style <file.html>              # Phase 5
crowser layout <file.html>             # Phase 6
crowser paint <file.html> -o out.png   # Phase 7
crowser open <file.html>               # Phase 8
```

## Running Tests

```bash
./tools/test-runner.sh <phase-number> <path-to-binary>
```

## Writing Style

- Second person ("you", "your") in guides
- Technical but clear, no jargon without explanation
- No em dashes
- Code blocks always have language specifiers
- Pseudocode in guides, not complete solutions
