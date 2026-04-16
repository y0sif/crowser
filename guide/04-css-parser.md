# Phase 4: CSS Parser

## What You'll Learn

- The structure of a CSS stylesheet: rules, selectors, and declarations
- How to parse simple and combined selectors from a token stream
- How to parse property-value declarations
- How to handle comma-separated selector lists
- How declaration values are reconstructed from tokens into strings

## CSS Structure

A CSS stylesheet is a list of *rules*. Each rule has two parts:

1. **Selectors** -- which elements the rule applies to
2. **Declarations** -- what styles to apply to those elements

```css
h1, .title {
  color: red;
  margin: 10px;
}
```

This is one rule with two selectors (`h1` and `.title`) and two declarations
(`color: red` and `margin: 10px`). Your parser's job is to turn the flat token
stream from Phase 3 into this structured representation.

The output is a JSON object with a `rules` array:

```json
{
  "rules": [
    {
      "selectors": [ ... ],
      "declarations": [ ... ]
    }
  ]
}
```

## Parsing the Top Level

At the top level, you are reading a sequence of rules. Each rule looks like:

```
<selectors> { <declarations> }
```

So your top-level parsing loop is:

1. Read tokens until you see an `OpenBrace`. Everything before the brace is
   selectors.
2. Read tokens until you see a `CloseBrace`. Everything between the braces is
   declarations.
3. Repeat until you run out of tokens.

In pseudocode:

```rust
fn parse_stylesheet(tokens: &[Token]) -> Stylesheet {
    let mut rules = Vec::new();
    let mut pos = 0;

    while pos < tokens.len() {
        // Parse selectors (up to OpenBrace)
        let selectors = parse_selectors(tokens, &mut pos);
        // Expect and consume OpenBrace
        expect(tokens, &mut pos, Token::OpenBrace);
        // Parse declarations (up to CloseBrace)
        let declarations = parse_declarations(tokens, &mut pos);
        // Expect and consume CloseBrace
        expect(tokens, &mut pos, Token::CloseBrace);

        rules.push(Rule { selectors, declarations });
    }

    Stylesheet { rules }
}
```

## Parsing Selectors

Selectors appear before the `{`. They can be comma-separated:

```css
h1, h2, .title { ... }
```

This means: parse one selector, and if you see a `Comma`, parse another.
Repeat until you reach the `OpenBrace`.

Each individual selector is a *simple selector* that targets elements by some
combination of tag name, ID, and classes:

| CSS               | tag     | id       | classes              |
|--------------------|---------|----------|----------------------|
| `p`               | `"p"`   | `null`   | `[]`                 |
| `.main`           | `null`  | `null`   | `["main"]`           |
| `#header`         | `null`  | `"header"` | `[]`               |
| `div.container`   | `"div"` | `null`   | `["container"]`      |
| `div#main.wide`   | `"div"` | `"main"` | `["wide"]`           |
| `*`               | `"*"`   | `null`   | `[]`                 |
| `.card.featured`  | `null`  | `null`   | `["card", "featured"]` |

A selector is represented as:

```json
{ "tag": "div", "id": "main", "classes": ["container"] }
```

Fields that are not specified are `null` (for tag and id) or `[]` (for classes).

### Parsing a Single Selector

Look at the tokens and build up the selector piece by piece:

1. If the current token is an **Ident**, it is the tag name. Consume it.
2. If the current token is a **Star**, it is the universal selector. Set tag to
   `"*"`. Consume it.
3. Now, in a loop, check for ID and class parts:
   - If the current token is a **Hash**, it is an ID. Set the selector's id.
     Consume it.
   - If the current token is a **Dot**, the next token should be an **Ident**
     -- that is a class name. Consume both and add the class to the list.
   - If the current token is anything else (Comma, OpenBrace, etc.), the
     selector is complete.

This loop handles combined selectors like `div.card.featured#hero`, which
produces:

```json
{ "tag": "div", "id": "hero", "classes": ["card", "featured"] }
```

The parts after the tag name can appear in any order. `div#main.wide` and
`div.wide#main` mean the same thing.

### Walking Through `h1, .title`

Starting from these tokens:

```
Ident("h1"), Comma, Dot, Ident("title"), OpenBrace
```

**Parse first selector:**
- Token is Ident("h1") -> tag = "h1"
- Next token is Comma -> selector is done
- Result: `{ "tag": "h1", "id": null, "classes": [] }`

**Consume the Comma.**

**Parse second selector:**
- Token is Dot -> next token is Ident("title") -> class = "title"
- Next token is OpenBrace -> selector is done
- Result: `{ "tag": null, "id": null, "classes": ["title"] }`

The selectors array is:
```json
[
  { "tag": "h1", "id": null, "classes": [] },
  { "tag": null, "id": null, "classes": ["title"] }
]
```

## Parsing Declarations

Declarations appear between `{` and `}`. Each declaration has the form:

```
property: value;
```

In terms of tokens: an **Ident** (property name), a **Colon**, one or more
value tokens, and a **Semicolon**.

### The Property Name

The first token in a declaration is always an Ident -- the property name.
Property names are lowercased (your tokenizer already handles this).

### The Value

The value is everything between the `:` and the `;`. This is where it gets
interesting, because values can be made up of multiple tokens:

| CSS Declaration              | Value Tokens                         | Value String       |
|------------------------------|--------------------------------------|--------------------|
| `color: red;`               | Ident("red")                         | `"red"`            |
| `margin: 10px;`             | Dimension("10", "px")                | `"10px"`           |
| `width: 50%;`               | Percentage("50")                     | `"50%"`            |
| `border: 2px solid black;`  | Dimension("2","px"), Ident("solid"), Ident("black") | `"2px solid black"` |
| `padding: 10px 20px;`       | Dimension("10","px"), Dimension("20","px") | `"10px 20px"` |

The parser does not interpret these values further. It collects the value
tokens and reconstructs them into a single string:

- **Ident** -> its value (e.g., `"red"`)
- **Number** -> its value (e.g., `"10"`)
- **Dimension** -> value + unit (e.g., `"10px"`)
- **Percentage** -> value + "%" (e.g., `"50%"`)
- **Hash** -> "#" + value (e.g., `"#ff0000"`)
- **String** -> the string content

If there are multiple value tokens, join them with spaces. The result is a
single string like `"2px solid black"`.

Values are lowercased in the output. This is handled naturally since your
tokenizer lowercases identifiers and the spec requires lowercase values.

### Walking Through a Declaration Block

Tokens between `{` and `}`:

```
Ident("color"), Colon, Ident("red"), Semicolon,
Ident("margin"), Colon, Dimension("10", "px"), Semicolon
```

**Parse first declaration:**
- Ident("color") -> property = "color"
- Colon -> consume
- Ident("red") -> value tokens = ["red"]
- Semicolon -> declaration complete
- Result: `{ "property": "color", "value": "red" }`

**Parse second declaration:**
- Ident("margin") -> property = "margin"
- Colon -> consume
- Dimension("10", "px") -> value tokens = ["10px"]
- Semicolon -> declaration complete
- Result: `{ "property": "margin", "value": "10px" }`

## Full Example Walkthrough

Let's parse this complete stylesheet:

```css
div.main {
  color: red;
  margin: 10px;
}
```

**Phase 3 produces these tokens:**
```
Ident("div"), Dot, Ident("main"), OpenBrace,
Ident("color"), Colon, Ident("red"), Semicolon,
Ident("margin"), Colon, Dimension("10", "px"), Semicolon,
CloseBrace
```

**Parse selectors (up to OpenBrace):**
- Ident("div") -> tag = "div"
- Dot, Ident("main") -> class = "main"
- OpenBrace -> selectors done
- Result: `[{ "tag": "div", "id": null, "classes": ["main"] }]`

**Consume OpenBrace.**

**Parse declarations (up to CloseBrace):**
- color: red -> `{ "property": "color", "value": "red" }`
- margin: 10px -> `{ "property": "margin", "value": "10px" }`

**Consume CloseBrace.**

**Final output:**

```json
{
  "rules": [
    {
      "selectors": [
        { "tag": "div", "id": null, "classes": ["main"] }
      ],
      "declarations": [
        { "property": "color", "value": "red" },
        { "property": "margin", "value": "10px" }
      ]
    }
  ]
}
```

## Implementation Notes

### Token Consumption Pattern

You will find yourself writing a lot of "peek at the current token, decide what
to do, consume it, advance." A common pattern in Rust is to track a position
index and pass it by mutable reference:

```rust
fn peek(tokens: &[Token], pos: usize) -> Option<&Token> {
    tokens.get(pos)
}

fn consume(tokens: &[Token], pos: &mut usize) -> Option<&Token> {
    let token = tokens.get(*pos);
    *pos += 1;
    token
}
```

Or you can use an iterator with `Peekable`. Choose whatever feels clean to you.

### Stopping Conditions

When parsing selectors, you stop at `OpenBrace`. When parsing declarations, you
stop at `CloseBrace`. When parsing a single declaration's value, you stop at
`Semicolon` or `CloseBrace` (the last declaration in a block might not have a
trailing semicolon, though the test inputs typically include one).

### Multiple Classes

`.card.featured` is two Dot-Ident pairs in a row. Your selector parser's loop
handles this naturally: after consuming `.card`, it sees another Dot and
consumes `.featured`. Both class names end up in the classes array.

### The Universal Selector

`*` is a Star token. When you see it at the start of a selector (or as the
entire selector), set `tag` to the string `"*"`.

```css
* { margin: 0; }
```

Produces:

```json
{
  "selectors": [{ "tag": "*", "id": null, "classes": [] }],
  "declarations": [{ "property": "margin", "value": "0" }]
}
```

## Edge Cases

- **Empty stylesheet:** `""` produces `{ "rules": [] }`.
- **Multiple rules:** parse them in order. Each rule is independent.
- **Selector with only an ID:** `#sidebar { ... }` -> `{ "tag": null, "id": "sidebar", "classes": [] }`.
- **Selector with only classes:** `.card.wide { ... }` -> `{ "tag": null, "id": null, "classes": ["card", "wide"] }`.
- **Multi-word values:** `border: 2px solid black` -> value is `"2px solid black"`.
- **Hash in values:** `color: #ff0000` -> the Hash token becomes `"#ff0000"` in
  the value string (note: the `#` is prepended back when reconstructing the
  string).

## Testing

See `spec/phase-04-css-parser/SPEC.md` for the exact output format, selector
representation, and declaration rules.

```
cargo build
./tools/test-runner.sh 4 ./target/debug/crowser
```

Stuck? Check `hints/phase-04/` for implementation guidance.

## The CLI Subcommand

```
crowser parse-css <file.css>
```

Reads a CSS file, tokenizes it (reusing your Phase 3 tokenizer), parses the
tokens into a stylesheet, and prints the JSON to stdout.

## What You'll Have at the End

A CSS parser that produces structured stylesheets from raw CSS text. Combined
with your HTML parser from Phase 2, you now have both halves of the input
pipeline complete. You can turn HTML into a DOM tree and CSS into a stylesheet.

The next phase brings these together: matching CSS rules to DOM elements and
computing which styles apply to each node. That is where your browser engine
starts to feel real.

## What's Next

Phase 5: style resolution. This is where the DOM tree and the stylesheet meet.
For every element in the DOM, you will determine which CSS rules match it and
what the final computed styles are. The output is a *styled tree* -- each node
carrying its resolved CSS properties.

The guide for Phase 5 is not yet available. Check the spec at
`spec/phase-05-style-resolution/SPEC.md` and the CLI contract at
`spec/cli-interface.md` to work ahead.
