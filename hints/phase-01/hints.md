# Phase 1 Hints: HTML Tokenizer

## Suggested Types

```rust
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum Token {
    StartTag {
        name: String,
        attributes: HashMap<String, String>,
    },
    EndTag {
        name: String,
    },
    SelfClosingTag {
        name: String,
        attributes: HashMap<String, String>,
    },
    Text {
        content: String,
    },
    Comment {
        content: String,
    },
}
```

The `#[serde(tag = "type")]` attribute produces the internally tagged representation that the spec requires. When you serialize a `Token::StartTag { name: "div", attributes: {} }`, serde outputs `{"type":"StartTag","name":"div","attributes":{}}`.

## Architecture

A `Tokenizer` struct that holds the input and a cursor position:

```rust
pub struct Tokenizer {
    input: Vec<char>,
    pos: usize,
}

impl Tokenizer {
    pub fn new(input: &str) -> Self { ... }
    pub fn tokenize(&mut self) -> Vec<Token> { ... }
    fn next_token(&mut self) -> Option<Token> { ... }

    // Internal helpers
    fn peek(&self) -> Option<char> { ... }
    fn advance(&mut self) -> Option<char> { ... }
    fn starts_with(&self, s: &str) -> bool { ... }
    fn read_while(&mut self, pred: impl Fn(char) -> bool) -> String { ... }
}
```

The main loop in `tokenize()` calls `next_token()` repeatedly until it returns `None` (end of input). Each call to `next_token()` inspects the current character and decides what kind of token to read:

- `<` at the current position? It is a tag or comment. Check the next characters.
- `<!` followed by `--`? Read a comment.
- `</`? Read an end tag.
- `<` followed by a letter? Read a start tag (which might turn out to be self-closing).
- Anything else? Read text until you hit `<` or end of input.

## Rust Tips

**Use `Vec<char>` for the input.** Working with `&str` and byte indices is error-prone when you need to peek ahead, back up, or index by character position. Converting to `Vec<char>` upfront makes the tokenizer much simpler. The performance cost is negligible for HTML files.

**`read_while` is your workhorse.** A helper that advances the cursor while a predicate holds and returns the collected string covers most of the tokenizer's work: reading tag names, attribute names, attribute values, text content, and comment bodies.

**Lowercase tag and attribute names at read time.** Call `.to_ascii_lowercase()` on the collected string immediately after reading it. Do not store mixed case and lowercase later -- you will forget somewhere.

**Boolean attributes.** When parsing attributes, if you see a character that is not `=` after the attribute name (either whitespace followed by another attribute name, or `>`, or `/`), the attribute has no value. Store it with the value `""` (empty string).

**Attribute values can be quoted or unquoted in real HTML, but the spec only requires double-quoted and single-quoted forms, plus boolean attributes.** Keep it simple: after `=`, check if the next character is `"` or `'`. If so, read until the matching quote. If not, it is a boolean attribute (unusual, but handle it gracefully).

## Common Pitfalls

**Forgetting to handle `/>` for self-closing tags.** When reading a start tag, after parsing attributes, check if the next character is `/` before `>`. If it is, the token is `SelfClosingTag`, not `StartTag`. Watch for the space variant too: `<br />` has whitespace before the `/`.

**Whitespace-only text nodes.** The spec explicitly requires that whitespace between tags (newlines, indentation) becomes a `Text` token. Do not skip them. If you see `<div>\n  <p>`, the `\n  ` between `</div>` and `<p>` is a `Text` token.

**DOCTYPE declarations.** If the input starts with `<!DOCTYPE` (case-insensitive), skip everything until the closing `>`. Do not emit a token for it.

**Comment edge cases.** The comment delimiter is `<!--` to start and `-->` to end. The content between them is preserved exactly, including leading and trailing spaces. Make sure your reader consumes the full `-->` (three characters), not just `->`.

**Empty attributes map.** Tags with no attributes must serialize as `"attributes": {}`, not be omitted from the JSON. The `HashMap<String, String>` handles this naturally with serde, but double-check that your output matches.
