# Contributing to crowser

Thanks for your interest in making crowser better. This project is a community-driven tutorial, and contributions are welcome.

## What You Can Contribute

### Tutorial Content
- Fix typos, unclear explanations, or incorrect information in guides
- Improve ASCII diagrams or add new ones
- Add more context or examples to existing phases

### Test Fixtures
- Add new test cases for edge cases not currently covered
- Fix incorrect expected output in existing tests
- Add test cases for stretch goal phases

### Specs
- Clarify ambiguous spec language
- Add edge case definitions
- Propose spec changes (open an issue first to discuss)

### Hints
- Add hints for additional languages (future goal)
- Improve Rust-specific hints with better patterns or alternatives
- Fix outdated crate recommendations

### Tooling
- Improve the test runner
- Add support for new output comparison methods
- Build additional developer tools

## How to Contribute

1. Fork the repository
2. Create a branch: `git checkout -b feat/your-change`
3. Make your changes
4. Test that any modified test fixtures have valid JSON: `jq . expected.json`
5. Submit a pull request

## Guidelines

- Keep guides educational, not prescriptive. Teach concepts, not copy-paste solutions.
- Specs must be precise and unambiguous. If a spec can be interpreted two ways, it needs clarification.
- Test fixtures should be minimal. Each test case should test ONE concept.
- Expected output must be valid JSON (for JSON-based phases).
- Do not include complete implementations in guides or hints. Pseudocode and type suggestions are fine.

## Commit Messages

Use conventional commits:
- `docs:` for guide/spec/hint content changes
- `test:` for new or modified test fixtures
- `fix:` for corrections to existing content
- `feat:` for new phases, tools, or features
- `chore:` for tooling, CI, or repo maintenance

## Code of Conduct

Be respectful and constructive. This is an educational project. Questions are welcome, condescension is not.
