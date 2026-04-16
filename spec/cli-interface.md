# CLI Interface Specification

This is the authoritative contract for the `crowser` binary. Every subcommand, flag, output format, and exit code is defined here. If your implementation matches this spec, the test suite will pass.

## General Rules

- All JSON output is written to **stdout**.
- All error messages are written to **stderr**.
- Output encoding is always **UTF-8**.
- JSON output may be pretty-printed or minified -- the test runner accepts both.
- For phases 5-7, CSS is embedded in the HTML file inside `<style>` blocks. There is no separate CSS file argument.

## Exit Codes

Every subcommand follows the same exit code convention:

| Code | Meaning |
|------|---------|
| `0`  | Success. Output written to stdout (or file for `paint`). |
| `1`  | Parse error. The input was malformed beyond recovery. A human-readable error message must be printed to stderr. |
| `2`  | File not found. The specified input file does not exist. Print `Error: file not found: <path>` to stderr. |

## Subcommands

---

### `crowser tokenize <file.html>`

**Phase 1 -- HTML Tokenization**

Tokenizes an HTML file and outputs a JSON array of tokens to stdout.

**Usage:**
```
crowser tokenize path/to/file.html
```

**Output schema:**

A JSON array of token objects. Each token has a `"type"` field and type-specific fields:

| Token type       | Fields                        | Description |
|------------------|-------------------------------|-------------|
| `StartTag`       | `name`, `attributes`          | An opening tag like `<div class="x">` |
| `EndTag`         | `name`                        | A closing tag like `</div>` |
| `SelfClosingTag` | `name`, `attributes`          | A self-closing tag like `<br />` |
| `Text`           | `content`                     | Raw text between tags |
| `Comment`        | `content`                     | An HTML comment `<!-- ... -->` |

- `name` is a string (lowercase tag name).
- `attributes` is an object mapping attribute names (strings) to attribute values (strings). Empty if no attributes. Attribute names are lowercase.
- `content` is a string.

**Example:**

Input (`input.html`):
```html
<div class="main" id="app">
  Hello, <strong>world</strong>!
  <!-- greeting -->
  <br />
</div>
```

Output:
```json
[
  { "type": "StartTag", "name": "div", "attributes": { "class": "main", "id": "app" } },
  { "type": "Text", "content": "\n  Hello, " },
  { "type": "StartTag", "name": "strong", "attributes": {} },
  { "type": "Text", "content": "world" },
  { "type": "EndTag", "name": "strong" },
  { "type": "Text", "content": "!\n  " },
  { "type": "Comment", "content": " greeting " },
  { "type": "Text", "content": "\n  " },
  { "type": "SelfClosingTag", "name": "br", "attributes": {} },
  { "type": "Text", "content": "\n" },
  { "type": "EndTag", "name": "div" }
]
```

**Behavior with malformed input:**

- Unclosed tags: tokenize what you can. If the tokenizer cannot make progress, exit with code `1`.
- Missing closing `>`: emit a parse error (exit `1`).
- The tokenizer does **not** need to validate nesting -- that is the parser's job (Phase 2).

---

### `crowser parse <file.html>`

**Phase 2 -- HTML Parsing (DOM Tree Construction)**

Parses an HTML file into a DOM tree and outputs it as JSON to stdout.

**Usage:**
```
crowser parse path/to/file.html
```

**Output schema:**

A tree of node objects. Each node has a `"type"` field:

| Node type  | Fields                           | Description |
|------------|----------------------------------|-------------|
| `Element`  | `tag`, `attributes`, `children`  | An HTML element |
| `Text`     | `content`                        | A text node |
| `Comment`  | `content`                        | A comment node |

- `tag` is a string (lowercase).
- `attributes` is an object (same as tokenizer).
- `children` is an array of node objects.
- `content` is a string.

The root node is the outermost element. If the input has no root element, wrap everything in a synthetic `Document` node (type `"Document"`, with a `children` array).

**Example:**

Input:
```html
<html>
  <body>
    <p>Hello <em>world</em></p>
  </body>
</html>
```

Output:
```json
{
  "type": "Element",
  "tag": "html",
  "attributes": {},
  "children": [
    { "type": "Text", "content": "\n  " },
    {
      "type": "Element",
      "tag": "body",
      "attributes": {},
      "children": [
        { "type": "Text", "content": "\n    " },
        {
          "type": "Element",
          "tag": "p",
          "attributes": {},
          "children": [
            { "type": "Text", "content": "Hello " },
            {
              "type": "Element",
              "tag": "em",
              "attributes": {},
              "children": [
                { "type": "Text", "content": "world" }
              ]
            }
          ]
        },
        { "type": "Text", "content": "\n  " }
      ]
    },
    { "type": "Text", "content": "\n" }
  ]
}
```

**Behavior with malformed input:**

- Unclosed tags: auto-close them at the end of their parent or at end-of-file.
- Extra closing tags with no matching opener: ignore them.
- Completely unparseable input: exit with code `1`.

---

### `crowser tokenize-css <file.css>`

**Phase 3 -- CSS Tokenization**

Tokenizes a CSS file and outputs a JSON array of tokens to stdout.

**Usage:**
```
crowser tokenize-css path/to/file.css
```

**Output schema:**

A JSON array of token objects. Each token has a `"type"` field and type-specific fields:

| Token type    | Fields  | Example input | Description |
|---------------|---------|---------------|-------------|
| `Ident`       | `value` | `color`       | An identifier |
| `Hash`        | `value` | `#main`       | A hash (ID selector or hex color). Value does **not** include the `#`. |
| `Dot`         | --      | `.`           | A period (class selector prefix) |
| `String`      | `value` | `"hello"`     | A quoted string. Value does **not** include the quotes. |
| `Number`      | `value` | `42`          | A bare number (no unit). Value is a **string** (e.g. `"42"`). |
| `Dimension`   | `value`, `unit` | `10px`  | A number with a unit. `value` is a **string** (e.g. `"10"`), `unit` is a string. |
| `Percentage`  | `value` | `50%`         | A number followed by `%`. `value` is a **string** (e.g. `"50"`). |
| `Colon`       | --      | `:`           | |
| `Semicolon`   | --      | `;`           | |
| `Comma`       | --      | `,`           | |
| `OpenBrace`   | --      | `{`           | |
| `CloseBrace`  | --      | `}`           | |
| `Star`        | --      | `*`           | The universal selector |

Whitespace and CSS comments are **not** emitted as tokens -- they are consumed silently.

> **Note:** Parentheses (`(`, `)`) are **not** tokenized. Functions like `rgb()`, `calc()`, and `url()` are out of scope for Phase 3.

**Example:**

Input (`input.css`):
```css
p {
  color: red;
  font-size: 16px;
  width: 50%;
}
```

Output:
```json
[
  { "type": "Ident", "value": "p" },
  { "type": "OpenBrace" },
  { "type": "Ident", "value": "color" },
  { "type": "Colon" },
  { "type": "Ident", "value": "red" },
  { "type": "Semicolon" },
  { "type": "Ident", "value": "font-size" },
  { "type": "Colon" },
  { "type": "Dimension", "value": "16", "unit": "px" },
  { "type": "Semicolon" },
  { "type": "Ident", "value": "width" },
  { "type": "Colon" },
  { "type": "Percentage", "value": "50" },
  { "type": "Semicolon" },
  { "type": "CloseBrace" }
]
```

**Behavior with malformed input:**

- Unterminated strings: exit with code `1`.
- Unknown characters: skip them and continue tokenizing.

---

### `crowser parse-css <file.css>`

**Phase 4 -- CSS Parsing**

Parses a CSS file into a stylesheet and outputs it as JSON to stdout.

**Usage:**
```
crowser parse-css path/to/file.css
```

**Output schema:**

A stylesheet object containing an array of rules:

```json
{
  "rules": [ <rule>, ... ]
}
```

Each **rule** has:
- `selectors`: array of selector objects
- `declarations`: array of declaration objects

Each **selector** has:
- `tag`: string or `null` (tag name selector)
- `id`: string or `null` (ID selector)
- `classes`: array of strings (class selectors)

Each **declaration** has:
- `property`: string (property name, lowercase)
- `value`: string (property value as a string, preserving units, e.g. `"16px"`, `"50%"`, `"red"`)

**Example:**

Input:
```css
h1, .title {
  color: red;
  margin: 10px;
}

#main p {
  font-size: 16px;
}
```

Output:
```json
{
  "rules": [
    {
      "selectors": [
        { "tag": "h1", "id": null, "classes": [] },
        { "tag": null, "id": null, "classes": ["title"] }
      ],
      "declarations": [
        { "property": "color", "value": "red" },
        { "property": "margin", "value": "10px" }
      ]
    },
    {
      "selectors": [
        { "tag": "p", "id": "main", "classes": [] }
      ],
      "declarations": [
        { "property": "font-size", "value": "16px" }
      ]
    }
  ]
}
```

> **Note on compound selectors:** `#main p` is a descendant selector. For Phase 4, treat it as a simple selector matching the *last* component -- `p` with context `#main`. Specifically: `tag` = `"p"`, `id` = `"main"`. Full descendant/combinator support is a stretch goal.

**Behavior with malformed input:**

- Missing semicolons: recover by looking for the next `;` or `}`.
- Missing closing brace: close the rule at end-of-file.
- Completely empty input: output `{ "rules": [] }` with exit code `0`.

---

### `crowser style <file.html>`

**Phase 5 -- Style Resolution**

Parses an HTML file (with embedded `<style>` blocks), resolves CSS styles onto the DOM tree, and outputs the styled tree as JSON to stdout.

**Usage:**
```
crowser style path/to/file.html
```

CSS is extracted from `<style>` elements within the HTML. External stylesheets are not supported.

**Output schema:**

A tree of styled nodes. Each node has:
- `node`: the DOM node (same schema as Phase 2, but without `children`)
- `styles`: an object mapping CSS property names to their resolved values (strings)
- `children`: array of styled child nodes

Text nodes have `styles: {}` (empty object).

```json
{
  "node": {
    "type": "Element",
    "tag": "<tagname>",
    "attributes": {}
  },
  "styles": {
    "<property>": "<value>",
    ...
  },
  "children": [ ... ]
}
```

**Example:**

Input:
```html
<html>
<head>
  <style>
    div { display: block; color: blue; }
    .highlight { background-color: yellow; }
  </style>
</head>
<body>
  <div class="highlight">Hello</div>
</body>
</html>
```

Output (simplified -- only the body subtree shown for clarity):
```json
{
  "node": { "type": "Element", "tag": "html", "attributes": {} },
  "styles": {},
  "children": [
    {
      "node": { "type": "Element", "tag": "head", "attributes": {} },
      "styles": {},
      "children": []
    },
    {
      "node": { "type": "Element", "tag": "body", "attributes": {} },
      "styles": {},
      "children": [
        {
          "node": { "type": "Element", "tag": "div", "attributes": { "class": "highlight" } },
          "styles": {
            "display": "block",
            "color": "blue",
            "background-color": "yellow"
          },
          "children": [
            {
              "node": { "type": "Text", "content": "Hello" },
              "styles": {},
              "children": []
            }
          ]
        }
      ]
    }
  ]
}
```

**Style resolution rules:**

1. Only include properties that are explicitly set by a matching rule. Do not include defaults.
2. When multiple rules match the same element and set the same property, the last rule in source order wins (no specificity calculation required for the base implementation).
3. The full document structure is included. `<head>` elements appear in the tree and include their children. `<style>` elements appear in the tree but with `children: []` (their CSS text content is not included as text nodes). Whitespace-only text nodes are excluded from the styled tree.

---

### `crowser layout <file.html>`

**Phase 6 -- Layout**

Parses, styles, and lays out an HTML file, outputting the layout tree as JSON to stdout.

**Usage:**
```
crowser layout path/to/file.html
```

The default viewport is **800x600** pixels.

**Output schema:**

A tree of layout boxes. Each box has:
- `type`: `"Block"` or `"Inline"` or `"Anonymous"`
- `box`: the content box dimensions `{ "x", "y", "width", "height" }` (all floats)
- `padding`: `{ "top", "right", "bottom", "left" }` (all floats)
- `border`: `{ "top", "right", "bottom", "left" }` (all floats)
- `margin`: `{ "top", "right", "bottom", "left" }` (all floats)
- `children`: array of child layout boxes

```json
{
  "type": "Block",
  "box": {
    "x": 0.0,
    "y": 0.0,
    "width": 800.0,
    "height": 0.0
  },
  "padding": { "top": 0.0, "right": 0.0, "bottom": 0.0, "left": 0.0 },
  "border": { "top": 0.0, "right": 0.0, "bottom": 0.0, "left": 0.0 },
  "margin": { "top": 0.0, "right": 0.0, "bottom": 0.0, "left": 0.0 },
  "children": []
}
```

**Example:**

Input:
```html
<html>
<head>
  <style>
    .box {
      display: block;
      width: 200px;
      height: 100px;
      padding: 10px;
      border-width: 2px;
      margin: 20px;
    }
  </style>
</head>
<body>
  <div class="box"></div>
</body>
</html>
```

Output (the `.box` div's layout node):
```json
{
  "type": "Block",
  "box": {
    "x": 22.0,
    "y": 22.0,
    "width": 200.0,
    "height": 100.0
  },
  "padding": { "top": 10.0, "right": 10.0, "bottom": 10.0, "left": 10.0 },
  "border": { "top": 2.0, "right": 2.0, "bottom": 2.0, "left": 2.0 },
  "margin": { "top": 20.0, "right": 20.0, "bottom": 20.0, "left": 20.0 },
  "children": []
}
```

> Note: `x` = margin-left + border-left + padding-left of ancestors, plus this node's own margin-left + border-left. The `box` coordinates describe the **content area**.

**Layout rules (simplified):**

1. Block boxes stack vertically. Each block takes the full width of its container (minus padding, border, margin) unless an explicit `width` is set.
2. Heights are determined by content. An explicit `height` overrides this.
3. The box model follows CSS: content + padding + border + margin.
4. For the base implementation, only `display: block` and `display: none` are required.

---

### `crowser paint <file.html> -o <out.png>`

**Phase 7 -- Painting**

Renders an HTML file to a PNG image.

**Usage:**
```
crowser paint path/to/file.html -o output.png
```

The `-o` flag is **required**. If omitted, print usage info to stderr and exit with code `1`.

**Output:**

A PNG file at the specified path. The image dimensions match the viewport (800x600 by default).

**What to paint:**

1. Background colors (`background-color` property).
2. Borders (solid borders only, using `border-color` or inherited color).
3. Text (stretch goal -- start with just colored rectangles).

**Exit codes:**

Same as other subcommands. Additionally, if the output path is not writable, exit with code `1` and print an error to stderr.

---

### `crowser open <file.html>`

**Phase 8 -- The Window**

Opens an HTML file in a GUI window.

**Usage:**
```
crowser open path/to/file.html
```

This subcommand opens an OS window, renders the page, and blocks until the window is closed. There are no automated tests for this phase -- it is verified by visual inspection.

---

### `crowser <file.html>`

**Default subcommand -- alias for `open`**

If the first argument is a file path (not a known subcommand), treat it as `crowser open <file.html>`.

```
crowser page.html
# equivalent to:
crowser open page.html
```

---

## Global Flags

These flags are accepted by all subcommands:

| Flag               | Description |
|--------------------|-------------|
| `--help`, `-h`     | Print usage information and exit with code `0`. |
| `--version`, `-V`  | Print the version string and exit with code `0`. |

---

## Summary Table

| Subcommand      | Phase | Input | Output | Stdin | Stdout | File flag |
|-----------------|-------|-------|--------|-------|--------|-----------|
| `tokenize`      | 1     | HTML  | JSON   | No    | Yes    | No        |
| `parse`         | 2     | HTML  | JSON   | No    | Yes    | No        |
| `tokenize-css`  | 3     | CSS   | JSON   | No    | Yes    | No        |
| `parse-css`     | 4     | CSS   | JSON   | No    | Yes    | No        |
| `style`         | 5     | HTML  | JSON   | No    | Yes    | No        |
| `layout`        | 6     | HTML  | JSON   | No    | Yes    | No        |
| `paint`         | 7     | HTML  | PNG    | No    | No     | `-o`      |
| `open`          | 8     | HTML  | Window | No    | No     | No        |
