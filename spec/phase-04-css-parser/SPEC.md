# Phase 4: CSS Parser

## Command

```
crowser parse-css <file.css>
```

Reads a CSS file from disk and prints a JSON stylesheet to stdout.

## Output Format

A JSON object with a single `rules` array, printed to stdout.

```json
{
  "rules": [
    {
      "selectors": [
        { "tag": "p", "id": null, "classes": [] }
      ],
      "declarations": [
        { "property": "color", "value": "red" }
      ]
    }
  ]
}
```

## Structure

### Stylesheet

The top-level object.

```json
{
  "rules": [...]
}
```

- `rules` — an ordered array of rule objects, in source order

### Rule

A CSS rule consisting of selectors and declarations.

```json
{
  "selectors": [...],
  "declarations": [...]
}
```

- `selectors` — an ordered array of selector objects, in source order (left to right, as separated by commas)
- `declarations` — an ordered array of declaration objects, in source order

### Selector

Represents a simple selector. Each selector targets elements by tag name, ID, and/or classes.

```json
{ "tag": "div", "id": "main", "classes": ["container", "wide"] }
```

- `tag` — the tag name as a string, or `null` if no tag is specified. Lowercased. The universal selector `*` is represented as `"*"`.
- `id` — the ID as a string (without `#`), or `null` if no ID is specified. Lowercased.
- `classes` — an array of class names (without `.`), in source order. Lowercased. Empty array if no classes.

#### Selector Examples

| CSS | JSON |
|-----|------|
| `p` | `{ "tag": "p", "id": null, "classes": [] }` |
| `.main` | `{ "tag": null, "id": null, "classes": ["main"] }` |
| `#header` | `{ "tag": null, "id": "header", "classes": [] }` |
| `div.container` | `{ "tag": "div", "id": null, "classes": ["container"] }` |
| `div#main.wide` | `{ "tag": "div", "id": "main", "classes": ["wide"] }` |
| `*` | `{ "tag": "*", "id": null, "classes": [] }` |
| `.card.featured` | `{ "tag": null, "id": null, "classes": ["card", "featured"] }` |

### Declaration

A CSS property-value pair.

```json
{ "property": "margin", "value": "10px" }
```

- `property` — the property name, **lowercased**
- `value` — the value as a single string. Multiple parts are joined with spaces (e.g. `"2px solid black"`). The value is **not** parsed further — dimensions, colors, etc. remain as strings. Values are lowercased.

## Parsing Rules

1. **Source order is preserved.** Rules appear in the order they occur in the CSS. Selectors within a rule are in source order. Declarations within a rule are in source order.
2. **Comma-separated selectors.** `h1, h2, h3 { ... }` creates **one** rule with three selectors.
3. **Combined selectors.** `div.container#main` is a single selector with `tag: "div"`, `id: "main"`, `classes: ["container"]`. The parts can appear in any order after the tag name (e.g. `div#main.container` is also valid and means the same thing).
4. **Declaration values.** Everything after the `:` up to the `;` is the value, trimmed of leading/trailing whitespace. Multi-word values like `2px solid black` are a single string.
5. **Lowercasing.** Tag names, IDs, class names, property names, and values are all lowercased.

## Relationship to Phase 3

This phase consumes the same token stream that Phase 3 produces. The parser takes the flat array of CSS tokens and builds a stylesheet structure. Implementations may reuse their Phase 3 tokenizer internally, or re-implement tokenization — as long as the output matches this spec.

## Out of Scope

The following are **not** handled in this phase:

- **@-rules** (`@media`, `@import`, `@keyframes`, etc.)
- **Pseudo-classes** (`:hover`, `:first-child`, etc.)
- **Pseudo-elements** (`::before`, `::after`, etc.)
- **Combinator selectors** — descendant (space), child (`>`), sibling (`~`, `+`)
- **Attribute selectors** (`[type="text"]`)
- **Specificity calculation** — that is resolved in Phase 5
- **Shorthand property expansion** — `margin: 10px 20px` stays as-is
- **`!important`**
- **Comments** — consumed and discarded (same as Phase 3)

## Exit Codes

- `0` — success, JSON printed to stdout
- `1` — parse error (malformed input beyond recovery), error message printed to stderr
- `2` — file not found, print `Error: file not found: <path>` to stderr
