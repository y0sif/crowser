# Phase 6: Layout

## Command

```
crowser layout <file.html>
```

Reads an HTML file from disk, parses HTML and CSS, resolves styles, computes layout, and prints a JSON layout tree to stdout.

**Optional flags:**

```
crowser layout <file.html> --width <pixels> --height <pixels>
```

- `--width` -- viewport width in pixels (default: `800`)
- `--height` -- viewport height in pixels (default: `600`)

## Input

An HTML file containing one or more `<style>` blocks with CSS rules. Same input format as Phase 5.

## Output Format

A single JSON object representing the root of the layout tree, printed to stdout.

### Layout Box Node

```json
{
  "type": "Block",
  "tag": "div",
  "box": {
    "x": 0.0,
    "y": 0.0,
    "width": 800.0,
    "height": 100.0
  },
  "padding": { "top": 0.0, "right": 0.0, "bottom": 0.0, "left": 0.0 },
  "border": { "top": 0.0, "right": 0.0, "bottom": 0.0, "left": 0.0 },
  "margin": { "top": 0.0, "right": 0.0, "bottom": 0.0, "left": 0.0 },
  "children": [...]
}
```

- `type` -- one of `"Block"`, `"Text"`, or `"Anonymous"`
- `tag` -- the HTML tag name (present for Block nodes, absent for Text/Anonymous)
- `box` -- the **content area** rectangle: `x`, `y`, `width`, `height` (all floating-point numbers with one decimal place)
- `padding` -- the padding widths on each side (floating-point)
- `border` -- the border widths on each side (floating-point)
- `margin` -- the margin widths on each side (floating-point)
- `children` -- an ordered array of child layout boxes

### Text Node

```json
{
  "type": "Text",
  "content": "hello world",
  "box": {
    "x": 0.0,
    "y": 0.0,
    "width": 800.0,
    "height": 16.0
  }
}
```

- `type` -- always `"Text"`
- `content` -- the text string
- `box` -- the content area rectangle
- Text nodes have no `padding`, `border`, `margin`, or `children` fields
- Text width equals the parent's content width
- Text height equals the resolved `font-size` value (simplified -- real text layout is out of scope)

## Box Model

The CSS box model defines four areas around an element's content:

```
+--------------------------------------------------+
|                    margin                         |
|  +--------------------------------------------+  |
|  |                  border                     |  |
|  |  +--------------------------------------+  |  |
|  |  |                padding               |  |  |
|  |  |  +--------------------------------+  |  |  |
|  |  |  |           content              |  |  |  |
|  |  |  |     (box.width x box.height)   |  |  |  |
|  |  |  +--------------------------------+  |  |  |
|  |  +--------------------------------------+  |  |
|  +--------------------------------------------+  |
+--------------------------------------------------+
```

- `box.x` and `box.y` describe the **top-left corner of the content area**
- `box.width` and `box.height` describe the **content area dimensions**
- `padding`, `border`, and `margin` are additional space outside the content area
- The **total occupied width** of a box is: `margin-left + border-left + padding-left + width + padding-right + border-right + margin-right`
- The **total occupied height** of a box is: `margin-top + border-top + padding-top + height + padding-bottom + border-bottom + margin-bottom`

## Layout Algorithm

This phase implements **block layout only**. All block-level elements are laid out vertically, top to bottom.

### 1. Width Calculation

For each block box:

1. If the element has an explicit `width` set in CSS (e.g., `width: 200px`), use that value as `box.width`.
2. Otherwise, `box.width` = containing block's content width - own `margin-left` - own `border-left` - own `padding-left` - own `padding-right` - own `border-right` - own `margin-right`.

The **containing block** is the parent layout box. For the root element, the containing block is the viewport (`800.0` wide by default).

### 2. Position Calculation

For each block box:

1. `box.x` = containing block's content area x + own `margin-left` + own `border-left` + own `padding-left`

   Wait -- correction. `box.x` is the x coordinate of the content area's top-left corner:
   - `box.x` = parent content x + own `margin-left` + own `border-left` + own `padding-left`

2. `box.y` = starting y position + own `margin-top` + own `border-top` + own `padding-top`

   Where **starting y position** is:
   - For the first child: parent content area's y position
   - For subsequent children: the bottom edge of the previous sibling's total box (previous sibling's `box.y` - own `padding-top` - own `border-top` - own `margin-top` + previous sibling's total height)

   More precisely:
   - `cursor_y` starts at parent's content area y
   - For each child: `cursor_y` += child's `margin-top` + child's `border-top` + child's `padding-top` gives child's `box.y`
   - After laying out the child: `cursor_y` = child's `box.y` + child's `box.height` + child's `padding-bottom` + child's `border-bottom` + child's `margin-bottom`

### 3. Children Layout

Children are laid out top-to-bottom inside the parent's content area, in document order.

### 4. Height Calculation

For each block box:

1. If the element has an explicit `height` set in CSS (e.g., `height: 100px`), use that value as `box.height`.
2. Otherwise, `box.height` = the total vertical space occupied by all children. This is the distance from the top of the content area to the bottom of the last child's total box (including the last child's `padding-bottom`, `border-bottom`, and `margin-bottom`).
3. If there are no children, `box.height` = `0.0`.

### 5. Display None

Elements with `display: none` are **excluded** from the layout tree entirely. They produce no layout box and take up no space.

### 6. Root Box

The root element's layout box has:
- `box.x` = own `margin-left` + own `border-left` + own `padding-left`
- `box.y` = own `margin-top` + own `border-top` + own `padding-top`
- `box.width` = viewport width - own `margin-left` - own `border-left` - own `padding-left` - own `padding-right` - own `border-right` - own `margin-right`
- `box.height` = calculated from children (or explicit if set)

## Parsing CSS Values

Layout-relevant CSS values are parsed as follows:

- `"200px"` -- extract the numeric part as a float: `200.0`
- `"auto"` -- use the auto-sizing algorithm described above
- `"0px"` or unset margin/padding/border -- `0.0`

## Float Precision

All numeric values in the output use **one decimal place** (e.g., `800.0`, `22.0`, `0.0`). This applies to `box`, `padding`, `border`, and `margin` fields.

## Excluded Nodes

The following are excluded from the layout tree:

- Elements with `display: none`
- Comment nodes (already excluded in Phase 5)
- `<style>` elements (they have no visual representation)
- `<head>` elements and their children

Only elements within `<body>` (or the full tree if no `<body>`) produce layout boxes. If the input has `<html>` and `<body>` tags, the root layout box corresponds to the `<body>` element. If the input has no `<body>`, the outermost element is the root layout box.

**Simplified rule:** Walk the styled tree. Skip `<html>`, `<head>`, and `<style>` nodes -- start layout from `<body>` (or the first renderable element). The root layout box corresponds to the first renderable element.

## Relationship to Earlier Phases

This phase uses:
- Phase 1-4: HTML and CSS tokenization and parsing
- Phase 5: Style resolution (to know each element's `display`, `width`, `height`, `margin-*`, `padding-*`, `border-width`, `font-size`)

## Out of Scope

The following are **not** handled in this phase:

- **Inline layout** (horizontal flow of inline elements and text)
- **Floats**
- **Flexbox** and **Grid**
- **Margin collapsing** (adjacent vertical margins do not collapse; both are fully applied)
- **Percentage widths** or percentage values for any property
- **Auto margins for centering**
- **Min/max width/height**
- **Overflow handling**
- **Scrolling**

## Exit Codes

- `0` -- success, JSON printed to stdout
- `1` -- parse error, error message printed to stderr
- `2` -- file not found, print `Error: file not found: <path>` to stderr
