# Phase 4 Hints: CSS Parser

## Suggested Types

```rust
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Selector {
    pub tag: Option<String>,
    pub id: Option<String>,
    pub classes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Declaration {
    pub property: String,
    pub value: String,
}
```

Note: the universal selector `*` is represented as `tag: Some("*")`, not `tag: None`. Check the spec's selector examples table carefully.

## Architecture

A `CssParser` struct that walks through CSS tokens:

```rust
pub struct CssParser {
    tokens: Vec<CssToken>,
    pos: usize,
}

impl CssParser {
    pub fn new(tokens: Vec<CssToken>) -> Self { ... }
    pub fn parse_stylesheet(&mut self) -> Stylesheet { ... }
    fn parse_rule(&mut self) -> Option<Rule> { ... }
    fn parse_selectors(&mut self) -> Vec<Selector> { ... }
    fn parse_selector(&mut self) -> Selector { ... }
    fn parse_declarations(&mut self) -> Vec<Declaration> { ... }
    fn parse_declaration(&mut self) -> Option<Declaration> { ... }

    // Helpers
    fn peek(&self) -> Option<&CssToken> { ... }
    fn advance(&mut self) -> Option<&CssToken> { ... }
    fn expect(&mut self, expected: /* ... */) -> Result<(), Error> { ... }
}
```

The parsing flow:

1. `parse_stylesheet()` -- loop calling `parse_rule()` until end of tokens.
2. `parse_rule()` -- call `parse_selectors()`, expect `OpenBrace`, call `parse_declarations()`, expect `CloseBrace`.
3. `parse_selectors()` -- call `parse_selector()` in a loop, separated by `Comma` tokens.
4. `parse_selector()` -- build up a `Selector` by consuming tokens until you see `Comma`, `OpenBrace`, or end of tokens.
5. `parse_declarations()` -- call `parse_declaration()` in a loop until `CloseBrace`.
6. `parse_declaration()` -- read property (Ident), expect Colon, read value tokens until Semicolon or CloseBrace.

## Rust Tips

**Selector parsing.** A single selector like `div.main#content.wide` is a stream of tokens: `Ident("div")`, `Dot`, `Ident("main")`, `Hash("content")`, `Dot`, `Ident("wide")`. Parse them in a loop:

```
Start with tag = None, id = None, classes = [].
Loop:
  - Ident token (and tag is None and no Dot/Hash preceded it): set tag.
  - Dot token: expect the next token to be Ident, push to classes.
  - Hash token: set id from its value.
  - Star token: set tag to Some("*").
  - Anything else: stop (you have finished this selector).
```

Be careful with the first token. If the first token is `Dot` or `Hash`, there is no tag name. If it is `Ident`, it is the tag name. If it is `Star`, it is the universal selector.

**Declaration value reconstruction.** The value of a declaration is everything between `:` and `;`, joined as a single string. You may encounter multiple tokens: `Dimension { value: "10", unit: "px" }` should become the string `"10px"`. `Ident("solid")` becomes `"solid"`. A value like `2px solid black` is three tokens that you join with spaces: `"2px solid black"`.

Write a helper that converts a `CssToken` back to its string form:

```rust
fn token_to_string(token: &CssToken) -> String {
    match token {
        CssToken::Ident { value } => value.clone(),
        CssToken::Hash { value } => format!("#{}", value),
        CssToken::Number { value } => value.clone(),
        CssToken::Dimension { value, unit } => format!("{}{}", value, unit),
        CssToken::Percentage { value } => format!("{}%", value),
        CssToken::String { value } => format!("\"{}\"", value),
        // ... other variants
    }
}
```

Then collect value tokens and join with `" "`.

**`Option<String>` and serde.** By default, serde serializes `None` as `null` in JSON, which is what the spec requires for `tag` and `id` when they are not set. You do not need any special serde attributes for this.

**Lowercasing.** The spec requires property names and values to be lowercased. If your tokenizer already lowercases identifiers, property names are handled. For values, apply `.to_ascii_lowercase()` to the reconstructed value string.

## Common Pitfalls

**Universal selector representation.** The spec says `*` maps to `{ "tag": "*", "id": null, "classes": [] }`. Not `tag: null`. If you set `tag` to `None` for `*`, your tests will fail.

**Missing semicolons.** The last declaration before a `}` might not have a semicolon. When reading value tokens, stop at both `Semicolon` and `CloseBrace`. If you stopped at `CloseBrace`, do not consume it (the outer `parse_declarations` loop needs to see it).

**Empty rules.** A rule with no declarations (`p { }`) is valid. Your `parse_declarations` should handle seeing `CloseBrace` immediately and return an empty `Vec`.

**Comma handling in selectors.** When you see a `Comma` token while parsing selectors, finish the current selector, start a new one. Make sure you consume the `Comma` and skip any whitespace (already handled at the tokenizer level since whitespace is not emitted).

**Multi-word values.** Values like `2px solid black` are three tokens. Values like `10px 20px` (a shorthand margin) are two tokens. Collect all tokens up to the semicolon and join them. Do not stop at the first token after the colon.
