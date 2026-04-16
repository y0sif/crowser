# Phase 3: CSS Tokenizer

## Command

```
crowser tokenize-css <file.css>
```

Reads a CSS file from disk and prints a JSON array of tokens to stdout.

## Output Format

A JSON array printed to stdout. Each element is a token object. The array preserves source order. Whitespace is **not** included as tokens — it is consumed and discarded.

```json
[
  { "type": "Ident", "value": "p" },
  { "type": "OpenBrace" },
  { "type": "Ident", "value": "color" },
  { "type": "Colon" },
  { "type": "Ident", "value": "red" },
  { "type": "Semicolon" },
  { "type": "CloseBrace" }
]
```

## Token Types

### Ident

An identifier: tag names, property names, keyword values, unit-less keywords.

```json
{ "type": "Ident", "value": "color" }
```

- `value` — the identifier text, **always lowercased**

### Hash

A `#` followed by an identifier. Used for ID selectors and hex colors.

```json
{ "type": "Hash", "value": "header" }
```

- `value` — the text after the `#`, **without** the `#` prefix. Original case is **preserved** (not lowercased). Hash tokens are used for both ID selectors and hex colors (e.g. `#FF0000`), and lowercasing would corrupt hex color values.

### Dot

The `.` character, used before class names in selectors.

```json
{ "type": "Dot" }
```

### String

A quoted string (single or double quotes).

```json
{ "type": "String", "value": "hello world" }
```

- `value` — the string content **without** the surrounding quotes. Content is preserved as-is (no case transformation).

### Number

A bare number without a unit or percent sign.

```json
{ "type": "Number", "value": "10" }
```

- `value` — the number as a string, preserving the original representation (e.g. `"0"`, `"10"`, `"3.14"`)

### Dimension

A number immediately followed by a unit identifier (no space between them).

```json
{ "type": "Dimension", "value": "10", "unit": "px" }
```

- `value` — the numeric part as a string
- `unit` — the unit identifier, **lowercased** (e.g. `"px"`, `"em"`, `"rem"`, `"vh"`)

### Percentage

A number immediately followed by `%`.

```json
{ "type": "Percentage", "value": "50" }
```

- `value` — the numeric part as a string, **without** the `%`

### Colon

The `:` character.

```json
{ "type": "Colon" }
```

### Semicolon

The `;` character.

```json
{ "type": "Semicolon" }
```

### Comma

The `,` character.

```json
{ "type": "Comma" }
```

### OpenBrace

The `{` character.

```json
{ "type": "OpenBrace" }
```

### CloseBrace

The `}` character.

```json
{ "type": "CloseBrace" }
```

### Star

The `*` character, used as the universal selector.

```json
{ "type": "Star" }
```

## Rules

1. **Whitespace is skipped.** Spaces, tabs, and newlines between tokens are consumed but do not produce tokens.
2. **Identifiers are lowercased.** This includes tag names, property names, keyword values, and unit names. Hash values are **not** lowercased (they preserve original case to support hex colors like `#FF0000`).
3. **Numbers vs Dimensions vs Percentages.** When a number is immediately followed (no whitespace) by an identifier, it is a Dimension. When followed by `%`, it is a Percentage. Otherwise it is a Number.
4. **Hash tokens.** `#` followed by alphanumeric characters or hyphens produces a Hash token. The `#` is consumed but not included in the value.
5. **Dot tokens.** A `.` character produces a Dot token. The class name that follows will be a separate Ident token.
6. **String tokens.** Strings start with `"` or `'` and end with the matching quote. The quotes are not included in the value.
7. **Comments.** CSS comments (`/* ... */`) are consumed and discarded. They do not produce tokens.
8. **Source order.** Tokens appear in the order they occur in the source CSS.

## Out of Scope

The following are **not** handled in this phase:

- **@-rules** (`@media`, `@import`, `@keyframes`, etc.)
- **Functions** — `rgb()`, `calc()`, `url()`, etc. Parentheses are not tokenized.
- **Unicode ranges** (`U+0000-00FF`)
- **Escape sequences** (`\00A0`, `\"`)
- **CSS variables** (`--custom-property`, `var(...)`)
- **Nested selectors**
- **Important flag** (`!important`)

## Exit Codes

- `0` — success, JSON printed to stdout
- `1` — parse error (malformed input beyond recovery), error message printed to stderr
- `2` — file not found, print `Error: file not found: <path>` to stderr
