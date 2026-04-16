# Phase 3: CSS Tokenizer

## What You'll Learn

- Why CSS needs its own tokenizer, separate from the HTML tokenizer
- The token types that represent CSS's lexical structure
- How to distinguish numbers, dimensions, and percentages
- How to handle identifiers, hash tokens, dots, strings, and comments
- The differences between tokenizing CSS and tokenizing HTML

## Why a Separate Tokenizer?

HTML and CSS are different languages with different grammars. In HTML, the
fundamental structure is angle brackets and tag names. In CSS, the fundamental
structure is selectors, braces, properties, colons, and values. The characters
`{}:;,.#*` are all meaningful punctuation in CSS. Numbers can carry units
(`10px`, `50%`). Identifiers can contain hyphens (`background-color`,
`font-size`).

You could try to extend your HTML tokenizer to handle CSS, but you would end up
with an awkward hybrid that does neither job well. Better to build a clean CSS
tokenizer that understands CSS's rules from the ground up. The technique is the
same -- read characters, emit tokens -- but the rules are different.

## CSS Token Types

Your CSS tokenizer produces these token types:

**Ident** -- an identifier. This covers tag names in selectors (`p`, `div`),
property names (`color`, `font-size`), and keyword values (`red`, `block`,
`solid`). Identifiers can contain letters, digits, hyphens, and underscores.
They cannot start with a digit (that would be a number).

```json
{ "type": "Ident", "value": "font-size" }
```

**Hash** -- the `#` character followed by identifier characters. Used for ID
selectors (`#header`) and hex colors (`#ff0000`). The `#` is consumed but not
included in the value.

```json
{ "type": "Hash", "value": "header" }
```

**Dot** -- the `.` character by itself. In CSS, the dot is a prefix for class
selectors. The class name that follows becomes a separate Ident token.

```json
{ "type": "Dot" }
```

**Number** -- a bare number with no unit. Just digits, optionally with a
decimal point.

```json
{ "type": "Number", "value": "10" }
```

**Dimension** -- a number immediately followed by a unit identifier, with no
whitespace between them. The tokenizer must detect this: `10px` is one
Dimension token, not a Number followed by an Ident.

```json
{ "type": "Dimension", "value": "10", "unit": "px" }
```

**Percentage** -- a number immediately followed by `%`, with no whitespace.

```json
{ "type": "Percentage", "value": "50" }
```

**String** -- a quoted string, using either single or double quotes. The quotes
are consumed but not included in the value.

```json
{ "type": "String", "value": "hello world" }
```

**Punctuation tokens** -- these have no additional data:

| Token        | Character |
|--------------|-----------|
| `Colon`      | `:`       |
| `Semicolon`  | `;`       |
| `Comma`      | `,`       |
| `OpenBrace`  | `{`       |
| `CloseBrace` | `}`       |
| `Star`       | `*`       |

## The Tokenization Approach

Like the HTML tokenizer, this is a character-by-character scan. But CSS is
simpler in some ways -- there is no nesting of delimiters at the token level
(braces nest at the parser level, not the tokenizer level). The main challenge
is correctly classifying what you are reading.

Here is the decision tree for each character:

```
Is it whitespace?
  -> Skip it. CSS whitespace does not produce tokens.

Is it a letter, underscore, or hyphen (and not following a digit)?
  -> Start reading an identifier.
     Consume letters, digits, hyphens, underscores.
     Emit an Ident token.

Is it a digit?
  -> Start reading a number.
     Consume digits and at most one decimal point.
     Then check what follows immediately (no whitespace):
       - A letter? -> It is a Dimension. Read the unit identifier.
       - A '%'?    -> It is a Percentage.
       - Anything else? -> It is a Number.

Is it '#'?
  -> Read the next characters as identifier characters.
     Emit a Hash token with the value (without the #).

Is it '.'?
  -> Emit a Dot token.
     (The class name will be the next Ident token.)

Is it '"' or "'"?
  -> Read characters until the matching closing quote.
     Emit a String token with the content between the quotes.

Is it '/'?
  -> Check if the next character is '*'.
     If yes: this is a comment. Read until '*/'. Discard entirely.
     If no: this is an unexpected character. Skip it.

Is it one of { } : ; , *?
  -> Emit the corresponding punctuation token.
```

## Walking Through an Example

Let's tokenize this CSS:

```css
div.main {
  color: red;
  margin: 10px;
  width: 50%;
}
```

| Characters      | Token Emitted                          |
|-----------------|----------------------------------------|
| `div`           | Ident("div")                           |
| `.`             | Dot                                    |
| `main`          | Ident("main")                          |
| ` ` `{`         | (whitespace skipped) OpenBrace         |
| `color`         | Ident("color")                         |
| `:`             | Colon                                  |
| `red`           | Ident("red")                           |
| `;`             | Semicolon                              |
| `margin`        | Ident("margin")                        |
| `:`             | Colon                                  |
| `10px`          | Dimension("10", "px")                  |
| `;`             | Semicolon                              |
| `width`         | Ident("width")                         |
| `:`             | Colon                                  |
| `50%`           | Percentage("50")                       |
| `;`             | Semicolon                              |
| `}`             | CloseBrace                             |

Notice how `10px` becomes a single Dimension token, not `Number("10")` +
`Ident("px")`. And `50%` becomes a single Percentage token, not `Number("50")`
\+ something. This distinction is critical -- the parser in Phase 4 depends on
getting these as single tokens.

## Identifiers

CSS identifiers are more permissive than you might expect. They can contain:

- Letters (a-z, A-Z)
- Digits (0-9) -- but not as the first character
- Hyphens (`-`)
- Underscores (`_`)

This means `background-color` is a *single* Ident token, not three tokens. And
`-webkit-transform` is also a single Ident token (starts with a hyphen, which
is allowed).

All identifier values are lowercased in the output. `Color` becomes `"color"`.
`DIV` becomes `"div"`.

## Numbers, Dimensions, and Percentages

The key rule: after reading a number, look at what comes *immediately* next
(no whitespace).

- If the next character starts an identifier (a letter or hyphen), read the
  identifier and emit a Dimension token with the number and unit.
- If the next character is `%`, consume it and emit a Percentage token.
- Otherwise, emit a plain Number token.

This means:

| Input      | Token                        |
|------------|------------------------------|
| `10px`     | Dimension("10", "px")        |
| `2.5em`    | Dimension("2.5", "em")       |
| `100%`     | Percentage("100")            |
| `0`        | Number("0")                  |
| `10 px`    | Number("10"), then Ident("px") |

That last case is important: the space between `10` and `px` means they are
separate tokens. The "immediately followed" rule is about adjacency in the
source text.

Numbers can include a decimal point: `3.14`, `0.5`, `.5`. They can also be
negative: `-10px` is a Dimension with value `"-10"`. Handle the leading `-` by
checking if a hyphen is followed by a digit -- if so, it is part of a number,
not the start of an identifier.

## Hash Tokens

When you see `#`, read the following identifier characters and emit a Hash
token. The value does not include the `#` itself.

```
#header   ->  Hash("header")
#ff0000   ->  Hash("ff0000")
#main-nav ->  Hash("main-nav")
```

Hash values are lowercased, just like identifiers.

Both ID selectors and hex colors use the same token type. The CSS parser in
Phase 4 will determine from context whether a Hash is an ID selector or a
color value. The tokenizer does not need to care.

## Strings

Strings start with `"` or `'` and end with the matching quote. The content
between the quotes is the value. Do not transform the content -- preserve it
as-is (no lowercasing for string values).

```
"hello world"  ->  String("hello world")
'serif'        ->  String("serif")
```

## Comments

CSS comments start with `/*` and end with `*/`. When you see `/` followed by
`*`, read and discard everything until you find `*/`. Comments do not produce
tokens -- they are simply consumed.

```css
/* This is a comment */
p { color: red; /* inline comment */ }
```

The comment is invisible to the token stream.

## Whitespace

Whitespace (spaces, tabs, newlines) is consumed and discarded. It does not
produce tokens. This is different from the HTML tokenizer, where whitespace
text nodes are preserved.

The parser in Phase 4 benefits from this: it sees a clean stream of meaningful
tokens with no whitespace noise to filter.

## Edge Cases to Watch For

- **Hyphenated identifiers:** `font-size`, `background-color`, `border-left` --
  these are each a single Ident token. Do not split on hyphens.

- **Negative numbers:** `-10px` should produce Dimension("-10", "px"). The
  tricky part is distinguishing a negative number from a hyphen that starts an
  identifier. If `-` is followed by a digit, treat it as a number. If followed
  by a letter, treat it as an identifier.

- **Dot ambiguity:** The `.` in `.classname` is a Dot token followed by an
  Ident token. The `.` in `0.5` is part of a number. Distinguish by context:
  if `.` follows a digit or is followed by a digit in a number-reading context,
  it is part of the number. Otherwise, it is a Dot token.

- **Hash ambiguity:** `#header` and `#ff0000` both produce Hash tokens. The
  tokenizer does not distinguish between ID selectors and hex colors.

- **Empty input:** produces an empty token array `[]`.

- **Star:** the `*` character produces a Star token. It is the universal
  selector in CSS.

## Testing

See `spec/phase-03-css-tokenizer/SPEC.md` for the exact output format and
rules. The test suite covers basic rules, selectors, various value types,
comments, and combinations.

```
cargo build
./tools/test-runner.sh 3 ./target/debug/crowser
```

Stuck? Check `hints/phase-03/` for Rust-specific guidance.

## The CLI Subcommand

```
crowser tokenize-css <file.css>
```

Reads a CSS file, tokenizes it, and prints the JSON array to stdout. Same
exit code conventions as previous phases.

## What You'll Have at the End

A CSS tokenizer function and a new CLI subcommand. You now have tokenizers for
both languages your browser engine will process. The next step is to parse the
CSS token stream into a structured stylesheet -- the CSS equivalent of what
Phase 2 did for HTML.

## What's Next

Phase 4: the CSS parser. You will take the flat stream of CSS tokens and build
a structured stylesheet: a list of rules, each with selectors and declarations.
This is where `.card.featured { color: red; margin: 10px; }` becomes data
your style resolver can work with.

Continue to [04-css-parser.md](04-css-parser.md).
