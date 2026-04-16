# Phase 2: HTML Parser

## Command

```
crowser parse <file.html>
```

Reads an HTML file from disk and prints a JSON DOM tree to stdout.

## Output Format

A single JSON object representing the root of the DOM tree, printed to stdout.

If the HTML has exactly one root element, that element is the root of the output:

```json
{
  "type": "Element",
  "tag": "div",
  "attributes": {},
  "children": [
    { "type": "Text", "content": "hello" }
  ]
}
```

If the HTML has multiple root-level nodes (elements, text, or comments), wrap them in a synthetic Document node:

```json
{
  "type": "Document",
  "children": [
    { "type": "Element", "tag": "h1", "attributes": {}, "children": [...] },
    { "type": "Element", "tag": "p", "attributes": {}, "children": [...] }
  ]
}
```

## Node Types

### Element

An HTML element with a tag name, attributes, and children.

```json
{
  "type": "Element",
  "tag": "div",
  "attributes": { "class": "main", "id": "content" },
  "children": [...]
}
```

- `tag` — the tag name, **always lowercased**
- `attributes` — an object mapping attribute names to values. Same rules as Phase 1 (lowercased names, preserved values, boolean attributes are `""`)
- `children` — an ordered array of child nodes (Element, Text, or Comment). If no children, use `[]`

### Text

A text node.

```json
{ "type": "Text", "content": "hello world" }
```

- `content` — the text content, with whitespace **preserved as-is**

### Comment

A comment node.

```json
{ "type": "Comment", "content": " a comment " }
```

- `content` — the comment text, with whitespace **preserved as-is**

### Document

A synthetic root node used when the HTML has multiple root-level nodes.

```json
{
  "type": "Document",
  "children": [...]
}
```

- `children` — an ordered array of the root-level nodes

## Tree Construction Rules

1. **StartTag** opens a new Element node and pushes it onto the stack. Subsequent nodes become children of this element.
2. **EndTag** closes the current element by popping it from the stack. The tag name must match the currently open element.
3. **SelfClosingTag** creates an Element node with no children and adds it as a child of the current element (does not push onto the stack).
4. **Text** creates a Text node and adds it as a child of the current element.
5. **Comment** creates a Comment node and adds it as a child of the current element.
6. **Single root detection.** After processing all tokens, if there is exactly one root-level node, output that node directly. If there are multiple root-level nodes (or zero), wrap them in a Document node.
7. **Whitespace text nodes are included.** Same as Phase 1 — whitespace-only text nodes are part of the tree.

## Relationship to Phase 1

This phase consumes the same token stream that Phase 1 produces. The parser takes the flat array of tokens and builds a tree. Implementations may reuse their Phase 1 tokenizer internally, or re-implement tokenization — as long as the output matches this spec.

## Out of Scope

The following are **not** handled in this phase:

- **Implicit tag closing** — `<p>one<p>two` is not valid input for this phase; the parser does not auto-close tags
- **Error recovery** — input is assumed to be well-formed (every StartTag has a matching EndTag, nesting is correct)
- **Void elements auto-closing** — `<br>` without a slash is treated as a StartTag, not auto-closed. Use `<br/>` for self-closing. Input is assumed to use explicit self-closing syntax
- **DOCTYPE handling** — DOCTYPE declarations are skipped (same as Phase 1)
- **Entity decoding** — same as Phase 1, entities are literal text

## Exit Codes

- `0` — success, JSON printed to stdout
- `1` — file not found or read error (print error message to stderr)
