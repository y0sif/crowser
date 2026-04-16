# Phase 3 Hints: CSS Tokenizer

## Suggested Types

```rust
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum CssToken {
    Ident { value: String },
    Hash { value: String },
    Dot,
    String { value: String },
    Number { value: String },
    Dimension { value: String, unit: String },
    Percentage { value: String },
    Colon,
    Semicolon,
    Comma,
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    Star,
}
```

Note that `Number`, `Dimension`, and `Percentage` store their `value` as a `String`. The spec output preserves the original numeric representation (e.g., `"10"`, `"3.14"`). Storing as `String` avoids floating-point formatting surprises.

## Architecture

A `CssTokenizer` struct that mirrors your HTML tokenizer's shape:

```rust
pub struct CssTokenizer {
    input: Vec<char>,
    pos: usize,
}

impl CssTokenizer {
    pub fn new(input: &str) -> Self { ... }
    pub fn tokenize(&mut self) -> Vec<CssToken> { ... }
    fn next_token(&mut self) -> Option<CssToken> { ... }

    // Helpers
    fn skip_whitespace(&mut self) { ... }
    fn skip_comment(&mut self) { ... }
    fn read_ident(&mut self) -> String { ... }
    fn read_number(&mut self) -> String { ... }
    fn read_string(&mut self, quote: char) -> String { ... }
}
```

The main loop: skip whitespace and comments, then dispatch on the current character:

- `{`, `}`, `(`, `)`, `:`, `;`, `,` -- single-character tokens
- `*` -- Star token
- `.` -- Dot token
- `#` -- consume the `#`, read an identifier, produce a Hash token
- `"` or `'` -- read a string
- Digit or `-` followed by a digit -- read a number, then look ahead for a unit or `%`
- Letter or `-` (not followed by a digit) -- read an identifier

## Rust Tips

**Identifier characters.** CSS identifiers can contain letters, digits, hyphens, and underscores. They start with a letter, hyphen, or underscore. A simple predicate works:

```rust
fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '-' || c == '_'
}

fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '_'
}
```

**Number then what?** After reading a numeric string, peek at the next character:
- If it is `%`, consume it and produce `Percentage`.
- If it is a letter (start of a unit identifier), read the unit and produce `Dimension`.
- Otherwise, produce `Number`.

This three-way branch is the trickiest part of the CSS tokenizer. Get it right and everything else is straightforward.

**Lowercase everything.** The spec requires identifiers, hash values, and units to be lowercased. Apply `.to_ascii_lowercase()` right after reading, just like in the HTML tokenizer.

**CSS comments.** They start with `/*` and end with `*/`. In your whitespace-skipping loop, check for `/*` and consume everything up to and including `*/`. Do not emit a token. A common pattern is to loop `skip_whitespace` and `skip_comment` together until neither makes progress.

**Serde and unit-only variants.** Variants like `Dot`, `Colon`, and `Star` have no fields. With `#[serde(tag = "type")]`, they serialize as `{"type":"Dot"}`. That matches the spec.

## Common Pitfalls

**Hyphens in identifiers.** `background-color` is a single identifier containing a hyphen. Your `read_ident` must accept hyphens in the middle. But be careful: a bare `-` followed by a digit (like `-10px`) is a negative number, not an identifier. Check what follows the hyphen.

**Negative numbers.** If you see `-` and the next character is a digit, read it as a number (include the `-` in the numeric string). If `-` is followed by a letter, it is the start of an identifier (like `-webkit-transform`).

**Hash vs. standalone `#`.** The `#` character is always followed by an identifier in valid CSS (`#main`, `#ff0000`). Read the identifier after consuming `#` and produce a Hash token. The value does **not** include the `#`.

**Whitespace as a separator, not a token.** Unlike the HTML tokenizer which emits whitespace Text tokens, the CSS tokenizer silently discards whitespace. Make sure your main loop calls `skip_whitespace()` before dispatching. If after skipping whitespace you are at end of input, stop.

**String edge case.** The spec says unterminated strings cause exit code 1. Check that you hit the matching close quote before end of input.

**OpenParen and CloseParen.** The spec lists these token types even though CSS functions are out of scope. Include them for completeness -- they are simple single-character dispatches. You will need them if you later handle `rgb()`, `calc()`, etc.
