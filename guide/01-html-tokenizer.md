# Phase 1: HTML Tokenizer

## What You'll Learn

- What tokenization is and why it is the first step in any language pipeline
- How to model an HTML tokenizer as a state machine
- The token types that represent HTML's lexical structure
- How to handle attributes, self-closing tags, comments, and whitespace
- How to serialize your token stream as JSON for testing

## Why Tokenize?

You cannot parse structure directly from raw text. Consider this HTML:

```html
<div class="main">Hello, <em>world</em>!</div>
```

As a string, this is 47 characters. As a human, you can see the structure:
there is a `div` element with a class attribute, some text, an `em` element
with its own text, more text, and then both elements close. But a program
looking at the raw characters sees just a sequence of bytes. It does not know
where tag names start and end, which `>` closes which `<`, or where attribute
values live.

Tokenization solves this. It scans the character stream and breaks it into
*tokens* -- meaningful chunks that the parser can then arrange into a tree. Each
token has a type (start tag, end tag, text, etc.) and carries the relevant data
(the tag name, the attribute map, the text content).

After tokenization, the downstream parser never has to think about angle
brackets, quotes, or whitespace between attributes. It works with clean,
typed, structured objects.

## The Token Types

Your tokenizer will produce five types of tokens:

- **StartTag** -- an opening tag like `<div>` or `<p class="intro">`. Carries
  the tag name and a map of attributes.
- **EndTag** -- a closing tag like `</div>`. Carries just the tag name.
- **SelfClosingTag** -- a tag that opens and closes itself like `<br/>` or
  `<img src="photo.jpg"/>`. Carries the tag name and attributes, like StartTag.
- **Text** -- raw text content between tags. Carries the text string.
- **Comment** -- an HTML comment like `<!-- TODO: fix this -->`. Carries the
  comment text.

These five types are sufficient to represent any well-formed HTML document at
the lexical level.

## The State Machine Approach

An HTML tokenizer is a state machine. You maintain a "current state" and read
the input one character at a time. Each character either adds to a buffer,
triggers a state transition, or causes a token to be emitted.

Here are the core states and what triggers transitions between them:

```
          '<'                    '/'
  Data ──────> TagOpen ──────────────> EndTagName
   ^             |                         |
   |             | letter                  | '>'
   |             v                         v
   |          TagName ──> InsideTag    (emit EndTag,
   |             |           |          return to Data)
   |             | '>'       |
   |             v           |
   |        (emit StartTag,  |
   |         return to Data) |
   |                         |
   |    attr name / '='      |
   |         ┌───────────┐   |
   |         v           |   |
   |    AttributeName ───┘   |
   |         |               |
   |         | '=' + quote   |
   |         v               |
   |    AttributeValue       |
   |         |               |
   |         | closing quote |
   |         v               |
   |      InsideTag ─────────┘
   |         |
   |         | '>'  -> emit StartTag, return to Data
   |         | '/>' -> emit SelfClosingTag, return to Data
   |
   └──── '>' or '/>' (emit token, return here)
```

And separately:

```
  TagOpen + '!' + '-' + '-' ──> Comment state
  Comment state ──> read until '-->' ──> emit Comment, return to Data
```

Let's walk through how this processes `<p class="intro">Hello</p>`:

| Characters Read   | State            | Action                                 |
|-------------------|------------------|----------------------------------------|
| `<`               | Data -> TagOpen  | Start of a tag                         |
| `p`               | TagOpen -> TagName | Start reading tag name                |
| ` `               | TagName -> InsideTag | Tag name is "p", space means attributes follow |
| `c`, `l`, `a`, `s`, `s` | InsideTag -> AttributeName | Reading attribute name |
| `=`               | AttributeName    | Attribute name is "class", expect value |
| `"`               | -> AttributeValue | Start of quoted value                  |
| `i`, `n`, `t`, `r`, `o` | AttributeValue | Reading value characters     |
| `"`               | AttributeValue -> InsideTag | Value is "intro", attribute complete |
| `>`               | InsideTag -> Data | Emit StartTag("p", {class: "intro"})   |
| `H`, `e`, `l`, `l`, `o` | Data     | Reading text content                   |
| `<`               | Data -> TagOpen  | Emit Text("Hello"), start of next tag  |
| `/`               | TagOpen -> EndTagName | This is a closing tag              |
| `p`               | EndTagName       | Reading end tag name                   |
| `>`               | EndTagName -> Data | Emit EndTag("p")                      |

The output is three tokens:

```json
[
  { "type": "StartTag", "name": "p", "attributes": { "class": "intro" } },
  { "type": "Text", "content": "Hello" },
  { "type": "EndTag", "name": "p" }
]
```

## Implementing the State Machine

In Rust, the state machine maps naturally to an enum:

```rust
enum State {
    Data,
    TagOpen,
    TagName,
    EndTagName,
    InsideTag,
    AttributeName,
    AttributeValue,
    Comment,
    // ... you may need a few more
}
```

Your main loop looks something like this (conceptually):

```rust
let mut state = State::Data;
let mut pos = 0;

while pos < input.len() {
    let ch = input[pos];
    match state {
        State::Data => {
            if ch == '<' {
                // flush any accumulated text as a Text token
                state = State::TagOpen;
            } else {
                // accumulate character into text buffer
            }
        }
        State::TagOpen => {
            if ch == '/' {
                state = State::EndTagName;
            } else if ch == '!' {
                // might be a comment, check next chars
            } else {
                // start of tag name
                state = State::TagName;
            }
        }
        // ... handle each state
    }
    pos += 1;
}
```

Keep buffers for the current tag name, current attribute name, current
attribute value, and current text content. When you transition between states,
you finalize the buffer into a token or move data from one buffer to another.

## Attributes

Attributes appear inside start tags and self-closing tags. The spec supports
three forms:

- **Double-quoted:** `class="main"`
- **Single-quoted:** `class='main'`
- **Boolean (no value):** `disabled`

When you are in the `InsideTag` state and see a non-whitespace, non-`>`
character, you are starting an attribute name. Read characters until you hit
`=`, `>`, `/`, or whitespace.

If you hit `=`, the next character should be a quote (`"` or `'`). Read
characters until you see the matching closing quote. That is the attribute
value.

If you hit whitespace, `>`, or `/` without seeing `=`, you have a boolean
attribute. Its value is the empty string `""`.

Store attributes in a map (e.g., `HashMap<String, String>` or a
`Vec<(String, String)>` that you convert later). Attribute names are
lowercased. Attribute values are preserved as-is.

## Self-Closing Tags

A self-closing tag ends with `/>` instead of `>`. When you are in the
`InsideTag` state (or at the end of a tag name with no attributes) and see `/`
followed by `>`, emit a SelfClosingTag instead of a StartTag.

Watch out for the two forms:

- `<br/>` -- the slash comes right after the tag name
- `<br />` -- there is a space before the slash

Both produce the same token. Your state machine handles this naturally if the
`InsideTag` state treats `/` as a potential self-closing indicator.

## Comments

Comments start with `<!--` and end with `-->`. When you are in the `TagOpen`
state and see `!`, check if the next two characters are `--`. If so, enter a
comment state and read everything until you find `-->`.

The content of the comment is everything between `<!--` and `-->`, with
whitespace preserved as-is. So `<!-- hello -->` produces a comment with content
`" hello "` (note the spaces).

## Whitespace Handling

This is important and easy to get wrong: **whitespace text nodes are
preserved**. If there is a newline and some spaces between `</p>` and `<p>`,
that whitespace becomes a Text token. Do not skip it. Do not collapse it. Emit
it exactly as it appears in the source.

```html
<div>
  <p>one</p>
  <p>two</p>
</div>
```

This produces Text tokens for `"\n  "` between the tags. They matter for
correct parsing later.

## Lowercasing

Tag names and attribute names are always lowercased in the output. If the input
says `<DIV Class="Main">`, the token has `name: "div"` and the attribute key is
`"class"`. The attribute *value* `"Main"` is not lowercased -- values are
preserved exactly.

## Edge Cases to Watch For

- **Empty attributes map:** `<p>` has no attributes. Output `"attributes": {}`,
  not a missing key.
- **Multiple attributes:** `<div id="x" class="y">` -- make sure your parser
  handles the whitespace between attributes and does not accidentally merge them.
- **DOCTYPE declarations:** `<!DOCTYPE html>` starts with `<!` but is not a
  comment. Skip it entirely -- do not emit a token for it.
- **Text at the end of input:** if the input ends with text (no closing tag),
  still emit the text token.
- **Empty text:** whitespace between tags that consists of only spaces or
  newlines is still a valid Text token.

## Testing Your Implementation

The spec at `spec/phase-01-html-tokenizer/SPEC.md` defines the exact output
format, all token types, and the full set of rules. Read it carefully -- it is
the contract your implementation must satisfy.

The test suite has cases covering simple elements, nested elements, attributes,
self-closing tags, whitespace, comments, mixed content, boolean attributes, and
deeply nested structures. Run them with:

```
cargo build
./tools/test-runner.sh 1 ./target/debug/crowser
```

The test runner compares your JSON output structurally against expected output,
so formatting and key order do not matter. What matters is that every token is
present, in the right order, with the right type and data.

Stuck? Check `hints/phase-01/` for Rust-specific guidance on structuring your
tokenizer.

## The CLI Subcommand

Your tokenizer is invoked as:

```
crowser tokenize <file.html>
```

It reads the file, tokenizes it, and prints the JSON array of tokens to stdout.
If the file does not exist, print an error to stderr and exit with code 2. If
tokenization succeeds, exit with code 0.

See `spec/cli-interface.md` for the full CLI contract.

## What You'll Have at the End

A function that takes an HTML string and returns a `Vec` of tokens. A CLI
subcommand that reads a file, calls that function, serializes the tokens to
JSON, and prints the result. And a passing test suite that proves your
tokenizer handles real HTML correctly.

This is the foundation. Every phase that follows builds on the token stream you
produce here.

## What's Next

Phase 2: the HTML parser. You will take the flat list of tokens your tokenizer
produces and assemble them into a tree -- the DOM. The parser is where `<div>`,
Text, `</div>` becomes a tree node with a child.

Continue to [02-html-parser.md](02-html-parser.md).
