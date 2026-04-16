# Phase 5: Style Resolution

## Command

```
crowser style <file.html>
```

Reads an HTML file from disk, extracts CSS from embedded `<style>` blocks, matches rules to DOM nodes, and prints a JSON styled tree to stdout.

## Input

An HTML file containing one or more `<style>` blocks with CSS rules. External stylesheets (`<link>` tags) are not supported. The CSS is extracted from all `<style>` elements in the document, concatenated in source order, and parsed into a single stylesheet.

## Output Format

A single JSON object representing the root of the styled tree, printed to stdout. The tree mirrors the DOM structure from Phase 2, but each node is wrapped with resolved styles.

### Element Node

```json
{
  "node": {
    "type": "Element",
    "tag": "div",
    "attributes": { "class": "main" }
  },
  "styles": {
    "display": "block",
    "color": "black",
    "background-color": "transparent"
  },
  "children": [...]
}
```

- `node` -- the DOM node, same schema as Phase 2 (`type`, `tag`, `attributes`), but **without** `children` (children are in the outer `children` array)
- `styles` -- an object mapping CSS property names to their resolved string values. Every supported property always appears with its resolved value (explicit or default).
- `children` -- an ordered array of styled child nodes

### Text Node

```json
{
  "node": {
    "type": "Text",
    "content": "hello"
  },
  "styles": {},
  "children": []
}
```

Text nodes always have `styles: {}` (empty object) and `children: []`.

### Whitespace-Only Text Nodes

Whitespace-only text nodes (nodes containing only spaces, tabs, or newlines) are **excluded** from the styled tree. Only text nodes with non-whitespace content are included.

### Comment Nodes

Comment nodes are **excluded** from the styled tree. They are dropped during style resolution.

### Style and Head Elements

The styled tree includes the **full document structure**. `<head>` elements appear in the tree with their resolved default styles and include their children. `<style>` elements appear in the tree with their resolved default styles, but their `children` array is always empty -- the CSS text content inside `<style>` elements is not included as text nodes (it is CSS source, not renderable text).

## Supported CSS Properties

The following properties are supported in this phase. Each has a default value that applies when no rule sets the property.

### display

- Values: `block`, `inline`, `none`
- Default by tag:
  - **Block elements** (default `"block"`): `div`, `p`, `h1`, `h2`, `h3`, `h4`, `h5`, `h6`, `section`, `article`, `header`, `footer`, `main`, `ul`, `ol`, `li`, `html`, `body`, `head`
  - **Inline elements** (default `"inline"`): `span`, `a`, `strong`, `em`, `b`, `i`, `u`
  - **All other elements**: default `"block"`
- Does **not** inherit

### color

- Values: named colors or hex (`#RGB`, `#RRGGBB`)
- Default: `"black"` at the root, **inherited** from parent otherwise
- **Inherits**

### background-color

- Values: named colors or hex (`#RGB`, `#RRGGBB`)
- Default: `"transparent"`
- Does **not** inherit

### width

- Values: `<N>px` or `"auto"`
- Default: `"auto"`
- Does **not** inherit

### height

- Values: `<N>px` or `"auto"`
- Default: `"auto"`
- Does **not** inherit

### margin, margin-top, margin-right, margin-bottom, margin-left

- Values: `<N>px`
- Default: `"0px"`
- Does **not** inherit
- The shorthand `margin` expands to the four longhand properties (see Shorthand Expansion below)

### padding, padding-top, padding-right, padding-bottom, padding-left

- Values: `<N>px`
- Default: `"0px"`
- Does **not** inherit
- The shorthand `padding` expands to the four longhand properties (see Shorthand Expansion below)

### border-width

- Values: `<N>px`
- Default: `"0px"`
- Does **not** inherit

### border-color

- Values: named colors or hex
- Default: `"black"`
- Does **not** inherit

### font-size

- Values: `<N>px`
- Default: `"16px"` at the root, **inherited** from parent otherwise
- **Inherits**

## Property Output

The `styles` object on each element node must include **all** supported properties with their resolved values. The resolved value is determined by:

1. The value set by the highest-specificity matching rule (or last in source order if specificities are equal)
2. If no rule sets the property and the property inherits, the parent's resolved value
3. If no rule sets the property and the property does not inherit, the default value

The properties must appear in the following order in the output object:

`display`, `color`, `background-color`, `width`, `height`, `margin-top`, `margin-right`, `margin-bottom`, `margin-left`, `padding-top`, `padding-right`, `padding-bottom`, `padding-left`, `border-width`, `border-color`, `font-size`

## Selectors

The following simple selectors are supported:

### Tag Selector

Matches elements by tag name.

```css
p { color: red; }
```

Specificity: **1**

### Class Selector

Matches elements with a matching `class` attribute. An element's `class` attribute is split on whitespace to get its class list.

```css
.highlight { background-color: yellow; }
```

Specificity: **10**

### ID Selector

Matches elements with a matching `id` attribute.

```css
#title { font-size: 32px; }
```

Specificity: **100**

### Universal Selector

Matches all elements.

```css
* { margin: 0px; }
```

Specificity: **0**

### Compound Selectors

A selector may combine a tag name, an ID, and one or more classes (e.g., `div.main#content`). The specificity of a compound selector is the sum of its parts.

```css
p.highlight { color: green; }   /* specificity: 1 + 10 = 11 */
```

A compound selector matches an element only if **all** parts match.

## Specificity Rules

1. **Inline styles** (not implemented in this phase): specificity 1000
2. **ID selector** (`#name`): specificity 100 per ID
3. **Class selector** (`.name`): specificity 10 per class
4. **Tag selector** (`tagname`): specificity 1 per tag
5. **Universal selector** (`*`): specificity 0

When two declarations set the same property on the same element:
- The declaration from the higher-specificity selector wins
- If specificities are equal, the **last** declaration in source order wins

## Inheritance

The following properties **inherit** from the parent element if no rule explicitly sets them on the current element:

- `color`
- `font-size`

All other properties use their default values when not explicitly set.

At the root element, inherited properties use their initial values (`"black"` for `color`, `"16px"` for `font-size`).

## Shorthand Expansion

### margin shorthand

The `margin` shorthand expands to `margin-top`, `margin-right`, `margin-bottom`, `margin-left`:

- **1 value:** `margin: 10px` -- all four sides are `10px`
- **2 values:** `margin: 10px 20px` -- top/bottom are `10px`, left/right are `20px`
- **3 values:** `margin: 10px 20px 30px` -- top is `10px`, left/right are `20px`, bottom is `30px`
- **4 values:** `margin: 10px 20px 30px 40px` -- top, right, bottom, left in that order

### padding shorthand

Same expansion rules as `margin`, applied to `padding-top`, `padding-right`, `padding-bottom`, `padding-left`.

### Shorthand specificity

Shorthand properties have the same specificity as the selector they appear in. When a shorthand and a longhand conflict, normal specificity and source-order rules apply. A longhand property that appears after a shorthand in the same rule overrides the shorthand's value for that side.

## Named Colors

The following named colors must be supported (mapped to their hex equivalents for storage, or stored as the name string -- either is acceptable as long as output is consistent):

- `black` -- #000000
- `white` -- #FFFFFF
- `red` -- #FF0000
- `green` -- #008000
- `blue` -- #0000FF
- `yellow` -- #FFFF00
- `orange` -- #FFA500
- `purple` -- #800080
- `gray` -- #808080
- `transparent`

Color values in the output should be the same string form as specified in the CSS. If the CSS says `color: red`, the output should say `"color": "red"`. If the CSS says `color: #FF0000`, the output should say `"color": "#FF0000"`.

## Relationship to Earlier Phases

This phase uses:
- Phase 1 & 2: HTML tokenization and parsing to build the DOM tree
- Phase 3 & 4: CSS tokenization and parsing to build the stylesheet

The implementation extracts text content from `<style>` elements, parses it as CSS (Phase 3 & 4), then walks the DOM tree (Phase 2) to match rules and resolve styles.

## Out of Scope

The following are **not** handled in this phase:

- **Pseudo-classes** (`:hover`, `:first-child`, etc.)
- **Pseudo-elements** (`::before`, `::after`)
- **Media queries** (`@media`)
- **!important** declarations
- **Combinators** (descendant ` `, child `>`, sibling `+`, `~`)
- **Attribute selectors** (`[type="text"]`)
- **Inline styles** (`style="..."` attribute)
- **External stylesheets** (`<link>` tags)
- **@import** rules
- **CSS variables** (`--custom-property`, `var(...)`)
- **calc() or other CSS functions**

## Exit Codes

- `0` -- success, JSON printed to stdout
- `1` -- parse error (malformed HTML or CSS), error message printed to stderr
- `2` -- file not found, print `Error: file not found: <path>` to stderr
