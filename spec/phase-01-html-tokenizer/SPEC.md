# Phase 1: HTML Tokenizer

## Command

```
crowser tokenize <file.html>
```

Reads an HTML file from disk and prints a JSON array of tokens to stdout.

## Output Format

A JSON array printed to stdout. Each element is a token object. The array preserves source order.

```json
[
  { "type": "StartTag", "name": "p", "attributes": {} },
  { "type": "Text", "content": "hello" },
  { "type": "EndTag", "name": "p" }
]
```

## Token Types

### StartTag

An opening HTML tag.

```json
{ "type": "StartTag", "name": "div", "attributes": { "class": "main", "id": "content" } }
```

- `name` — the tag name, **always lowercased**
- `attributes` — an object mapping attribute names to values. Attribute names are **always lowercased**. If no attributes, use `{}`

### EndTag

A closing HTML tag.

```json
{ "type": "EndTag", "name": "div" }
```

- `name` — the tag name, **always lowercased**

### SelfClosingTag

A tag that closes itself (e.g. `<br/>`, `<img src="x"/>`).

```json
{ "type": "SelfClosingTag", "name": "br", "attributes": {} }
```

- `name` — the tag name, **always lowercased**
- `attributes` — same rules as StartTag

### Text

Raw text content between tags.

```json
{ "type": "Text", "content": "hello world" }
```

- `content` — the text content exactly as it appears in source. Consecutive whitespace is **preserved as-is**. Empty text nodes (containing only whitespace) **are included** in the output.

### Comment

An HTML comment (`<!-- ... -->`).

```json
{ "type": "Comment", "content": "this is a comment" }
```

- `content` — the text inside the comment delimiters, with leading/trailing whitespace **preserved as-is**

## Rules

1. **Tag names are lowercased.** `<DIV>` produces `"name": "div"`.
2. **Attribute names are lowercased.** `Class="main"` produces `"class": "main"`.
3. **Attribute values are preserved as-is.** No case transformation on values.
4. **Boolean attributes** (attributes without a value, like `disabled`) have their value set to the **empty string** `""`.
5. **Whitespace text nodes are included.** If there is whitespace between tags (spaces, newlines, tabs), it becomes a Text token. Do not strip or skip whitespace-only text.
6. **Consecutive whitespace is preserved.** Do not collapse spaces, newlines, or tabs in text content.
7. **Self-closing tags** are identified by the trailing `/>`. Both `<br/>` and `<br />` (with a space) are self-closing.
8. **Comments** are delimited by `<!--` and `-->`. Everything between those delimiters is the content.
9. **Source order.** Tokens appear in the order they occur in the source HTML.

## Attribute Parsing

Attributes appear inside start tags and self-closing tags. Supported forms:

- `key="value"` — double-quoted value
- `key='value'` — single-quoted value
- `key` — boolean attribute (value is `""`)

Multiple attributes are separated by whitespace.

## Out of Scope

The following are **not** handled in this phase:

- **DOCTYPE declarations** — if present, skip them entirely (do not emit a token)
- **CDATA sections**
- **Processing instructions** (`<?xml ... ?>`)
- **Entity decoding** — `&amp;` is kept as the literal string `&amp;`, not decoded to `&`
- **Script and style content parsing** — content inside `<script>` and `<style>` is not special; tokenize normally
- **Malformed HTML recovery** — input is assumed to be well-formed

## Exit Codes

- `0` — success, JSON printed to stdout
- `1` — parse error (malformed input beyond recovery), error message printed to stderr
- `2` — file not found, print `Error: file not found: <path>` to stderr
