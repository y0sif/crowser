# Phase 7: Painting

## Command

```
crowser paint <file.html> -o <output.png>
```

Reads an HTML file from disk, parses HTML and CSS, resolves styles, computes layout, paints the result to a pixel buffer, and writes a PNG file to the specified output path.

The `-o` flag is **required**. If omitted, print usage information to stderr and exit with code `1`.

## Input

An HTML file containing one or more `<style>` blocks with CSS rules. Same input format as Phase 5 and 6.

## Output

A PNG image file at the path specified by `-o`. The image dimensions match the layout viewport (800x600 by default).

**Optional flags:**

```
crowser paint <file.html> -o out.png --width <pixels> --height <pixels>
```

- `--width` -- canvas/viewport width in pixels (default: `800`)
- `--height` -- canvas/viewport height in pixels (default: `600`)

## Canvas

- Default size: **800x600 pixels**
- Default background: **white** (`#FFFFFF`)
- Pixel format: RGBA (8 bits per channel)
- The entire canvas is filled with white before any painting begins

## Painting Algorithm

The layout tree (from Phase 6) is traversed in **tree order** (pre-order: parent before children). For each layout box, the following paint operations are performed in order:

### 1. Paint Background

Fill the **padding box** (content area + padding) with the element's `background-color`.

- The padding box rectangle is:
  - x: `box.x - padding.left`
  - y: `box.y - padding.top`
  - width: `padding.left + box.width + padding.right`
  - height: `padding.top + box.height + padding.bottom`
- If `background-color` is `"transparent"`, skip this step (do not paint anything)

### 2. Paint Borders

Paint four rectangles for the border, one per side, using `border-color` and `border-width` from the resolved styles.

- **Top border:** a rectangle from the top-left of the border box to the top-right, with height = `border.top`
- **Bottom border:** a rectangle along the bottom edge, with height = `border.bottom`
- **Left border:** a rectangle along the left edge, with width = `border.left`
- **Right border:** a rectangle along the right edge, with width = `border.right`

The **border box** rectangle is:
- x: `box.x - padding.left - border.left`
- y: `box.y - padding.top - border.top`
- width: `border.left + padding.left + box.width + padding.right + border.right`
- height: `border.top + padding.top + box.height + padding.bottom + border.bottom`

If `border-width` resolves to `"0px"`, skip border painting.

### 3. Paint Text

For Text layout nodes, render the text string at the box position.

- Position: `(box.x, box.y)`
- Color: inherited `color` from the parent element
- Font size: inherited `font-size` from the parent element
- Font: use a **monospace** font. Each character occupies a fixed width. A reasonable approximation is `character_width = font_size * 0.6` (e.g., for 16px font, each character is ~9.6px wide).
- Text is rendered left-to-right, one line, no wrapping (wrapping is out of scope)
- If the text extends beyond the canvas, it is clipped at the canvas edge

### Paint Order

Because the tree is traversed in pre-order (parent first, then children), children are painted **on top of** their parents. This gives the correct visual stacking for nested elements -- a child's background will cover the parent's background in the child's area.

## Color Resolution

Colors from CSS values are resolved to RGBA values for painting.

### Named Colors

| Name          | Hex       | RGB             |
|---------------|-----------|-----------------|
| `black`       | `#000000` | (0, 0, 0)      |
| `white`       | `#FFFFFF` | (255, 255, 255) |
| `red`         | `#FF0000` | (255, 0, 0)    |
| `green`       | `#008000` | (0, 128, 0)    |
| `blue`        | `#0000FF` | (0, 0, 255)    |
| `yellow`      | `#FFFF00` | (255, 255, 0)  |
| `orange`      | `#FFA500` | (255, 165, 0)  |
| `purple`      | `#800080` | (128, 0, 128)  |
| `gray`        | `#808080` | (128, 128, 128) |
| `transparent` | --        | (0, 0, 0, 0)   |

### Hex Colors

- `#RGB` -- each hex digit is doubled: `#F00` becomes `#FF0000`
- `#RRGGBB` -- used directly

All named colors and hex colors are fully opaque (alpha = 255), except `transparent` which has alpha = 0.

## Testing Approach

Visual output is difficult to test automatically with exact pixel comparison (font rendering, anti-aliasing, and platform differences make exact matches unreliable). Instead, each test fixture provides:

- `input.html` -- the HTML file to paint
- `expected-description.txt` -- a human-readable description of what the output image should look like

Tests can be verified by:
1. **Visual inspection** -- run `crowser paint input.html -o output.png` and check the image
2. **Approximate pixel sampling** -- a test script can check that specific pixel coordinates have approximately the expected color (with tolerance for anti-aliasing)
3. **Reference image comparison** -- if a reference PNG is provided, compare with a tolerance threshold (e.g., RMSE < 5)

## Relationship to Earlier Phases

This phase uses:
- Phase 1-5: HTML/CSS parsing and style resolution
- Phase 6: Layout (provides the box positions and dimensions for painting)

The painting phase does **not** modify the layout tree. It only reads the layout boxes and their associated styles to produce pixels.

## Out of Scope

The following are **not** handled in this phase:

- **Anti-aliasing** (nice to have but not required)
- **Transparency / alpha blending** beyond the `transparent` keyword
- **Gradients** (`linear-gradient`, `radial-gradient`)
- **Images** (`<img>`, `background-image`)
- **Box shadows** (`box-shadow`)
- **Border radius** (`border-radius`)
- **Text decoration** (`text-decoration: underline`, etc.)
- **Text wrapping** (text is single-line, clipped at canvas edge)
- **Font selection** (always use a default monospace font)
- **Sub-pixel rendering**

## Exit Codes

- `0` -- success, PNG written to the output path
- `1` -- error (missing `-o` flag, unwritable output path, parse error), error message printed to stderr
- `2` -- file not found, print `Error: file not found: <path>` to stderr
